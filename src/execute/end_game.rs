use crate::constants::{
  GELOTTO_ADDR, GELOTTO_ANNUAL_PRIZE_ADDR, GELOTTO_NFT_SERIES_1_REWARDS_ADDR, GELOTTO_NFT_SERIES_2_REWARDS_ADDR,
  GELOTTO_OWNER_REWARDS_ADDR,
};
use crate::error::ContractError;
use crate::msg::WinnerSelection;
use crate::random;
use crate::random::pcg64_from_game_seed;
use crate::state::{
  Game, GameStatus, Player, Winner, GAME, INDEX_2_ADDR, INDICES, ORDERS, PLAYERS, PREV_HEIGHT, WINNERS,
};
use cosmwasm_std::{
  attr, to_binary, Addr, BankMsg, BlockInfo, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Storage, Uint128,
  WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use std::collections::HashSet;

pub fn execute_end_game(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  lucky_phrase: &Option<String>,
) -> Result<Response, ContractError> {
  let mut game: Game = GAME.load(deps.storage)?;

  authorize_and_validate(&game, &env)?;

  let orders = ORDERS.load(deps.storage)?;

  // Mark this end_game request as suspect if it's on the same block as the
  // latest buy_tickets execution. we can't allow this because an adversary
  // could employ a brute force attack to manipulate the PRNG seed in
  // buy_tickets, such that end_game always results in their wallet being drawn
  // as a winner.
  let mut is_suspect = false;
  if orders.len() > 1 {
    let has_same_block_height = PREV_HEIGHT.load(deps.storage)? == env.block.height;
    if has_same_block_height {
      is_suspect = true;
    }
  }

  update_game(deps.storage, &mut game, &info.sender, &env.block, lucky_phrase)?;

  // get total prize balance
  let jackpot: Coin = match game.cw20_token_address {
    Some(..) => Coin {
      amount: Uint128::from(game.ticket_count) * game.ticket_price,
      denom: game.denom.clone(),
    },
    None => deps
      .querier
      .query_balance(env.contract.address.clone(), game.denom.clone())?,
  };

  // if we only have one player, just refund that player and skip the whole
  // winner selection process.
  if game.player_count == 1 {
    if let Some(ticket_order) = ORDERS.load(deps.storage)?.get(0) {
      let player: Player = PLAYERS.load(deps.storage, ticket_order.owner.clone())?;
      WINNERS.save(
        deps.storage,
        0,
        &Winner {
          address: ticket_order.owner.clone(),
          ticket_count: player.ticket_count,
          claim_amount: jackpot.amount,
          position: 0,
          has_claimed: true,
        },
      )?;
      // transfer IBC or CW20 asset back to sole player
      match game.cw20_token_address {
        // transfer CW20 asset
        Some(cw20_token_address) => {
          let transfer = Cw20ExecuteMsg::Transfer {
            recipient: info.sender.clone().into(),
            amount: jackpot.amount.clone().into(),
          };
          let execute_msg = WasmMsg::Execute {
            contract_addr: cw20_token_address.clone().into(),
            msg: to_binary(&transfer)?,
            funds: vec![],
          };
          return Ok(Response::new().add_message(execute_msg).add_attributes(vec![
            attr("end_game", jackpot.amount.clone()),
            attr("to", info.sender.clone()),
            attr("winner_count", "1"),
          ]));
        },
        None => {
          // transfer IBC asset
          return Ok(
            Response::new()
              .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: ticket_order.owner.clone().into(),
                amount: vec![jackpot],
              }))
              .add_attributes(vec![attr("action", "end_game"), attr("winner_count", "1")]),
          );
        },
      }
    } else {
      return Ok(Response::new().add_attributes(vec![attr("action", "end_game"), attr("winner_count", "0")]));
    }
  } else {
    let game_admin = game.owner.as_str();
    let pct_gelotto: Uint128 = Uint128::from(25u128);
    let pct_gelotto_annual_grand_prize: Uint128 = Uint128::from(35u128);
    let pct_nft_series_1_rewards: Uint128 = Uint128::from(10u128);
    let pct_nft_series_2_rewards: Uint128 = Uint128::from(10u128);
    let pct_admin: Uint128 = Uint128::from(10u128);
    let pct_owner_rewards: Uint128 = Uint128::from(10u128);
    let pct_total_royalties: Uint128 = pct_gelotto
      + pct_gelotto_annual_grand_prize
      + pct_nft_series_1_rewards
      + pct_nft_series_2_rewards
      + pct_admin
      + pct_owner_rewards;

    // total amount split among winning wallets. this should equal 90% of the
    // total amount owned by the contract.
    let winnings = jackpot
      .amount
      .multiply_ratio(Uint128::from(1000u128) - pct_total_royalties, Uint128::from(1000u128));

    // find N winners and store in state
    let n_winners = select_winners(&info.sender, deps.storage, &game, winnings, is_suspect)?;

    let royalties = vec![
      (GELOTTO_ADDR, pct_gelotto),
      (GELOTTO_ANNUAL_PRIZE_ADDR, pct_gelotto_annual_grand_prize),
      (GELOTTO_NFT_SERIES_1_REWARDS_ADDR, pct_nft_series_1_rewards),
      (GELOTTO_NFT_SERIES_2_REWARDS_ADDR, pct_nft_series_2_rewards),
      (GELOTTO_OWNER_REWARDS_ADDR, pct_owner_rewards),
      (game_admin, pct_admin),
    ];

    // build response with royalty send msgs
    let response = match game.cw20_token_address {
      Some(cw20_token_address) => {
        let mut wasm_transfer_msgs: Vec<WasmMsg> = Vec::with_capacity(4);
        for (recipient, pct) in royalties.iter() {
          wasm_transfer_msgs.push(WasmMsg::Execute {
            contract_addr: cw20_token_address.clone().into(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
              recipient: recipient.clone().into(),
              amount: jackpot.amount.multiply_ratio(*pct, Uint128::from(1000u128)).into(),
            })?,
            funds: vec![],
          });
        }
        Response::new().add_messages(wasm_transfer_msgs).add_attributes(vec![
          attr("action", "end_game"),
          attr("winner_count", n_winners.to_string()),
        ])
      },
      None => {
        let mut cosmos_send_msgs: Vec<CosmosMsg> = Vec::with_capacity(4);
        for (recipient, pct) in royalties.iter() {
          cosmos_send_msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.clone().into(),
            amount: vec![Coin::new(
              jackpot.amount.multiply_ratio(*pct, Uint128::from(1000u128)).into(),
              game.denom.clone(),
            )],
          }));
        }
        Response::new().add_messages(cosmos_send_msgs).add_attributes(vec![
          attr("action", "end_game"),
          attr("winner_count", n_winners.to_string()),
        ])
      },
    };

    Ok(response)
  }
}

/// Is the game in a valid state to be ended?
fn authorize_and_validate(
  game: &Game,
  env: &Env,
) -> Result<(), ContractError> {
  if game.status != GameStatus::ACTIVE {
    return Err(ContractError::NotActive {});
  }
  if game.player_count == 0 {
    return Err(ContractError::NoWinners {});
  }
  // check if funding level is reached if applicable
  if let Some(funding_threshold) = game.funding_threshold {
    if Uint128::from(game.ticket_count) * game.ticket_price < funding_threshold {
      return Err(ContractError::UnderFundingThreshold { funding_threshold });
    }
  }
  // check if game end time is reached if applicable
  if let Some(ends_after) = game.ends_after {
    if env.block.time <= ends_after {
      return Err(ContractError::NotAuthorized {});
    }
  }
  Ok(())
}

/// Update the game's state, effectively "ending" it.
fn update_game(
  storage: &mut dyn Storage,
  game: &mut Game,
  sender: &Addr,
  block: &BlockInfo,
  lucky_phrase: &Option<String>,
) -> Result<(), ContractError> {
  game.status = GameStatus::ENDED;
  game.seed = random::seed::finalize(game, sender, block.height, lucky_phrase);
  game.ended_at = Some(block.time.clone());
  game.ended_by = Some(sender.clone());
  GAME.save(storage, &game)?;
  Ok(())
}

/// select the winners using game's seed
fn select_winners(
  sender: &Addr,
  storage: &mut dyn Storage,
  game: &Game,
  total_reward: Uint128,
  is_suspect: bool,
) -> Result<u32, ContractError> {
  let (n_winners, pct_split) = match game.selection.clone() {
    WinnerSelection::Fixed {
      winner_count,
      max_winner_count,
      pct_split,
    } => {
      let mut n_winners = std::cmp::min(game.player_count, winner_count as u32);
      if let Some(n_max) = max_winner_count {
        if n_max > 0 {
          n_winners = std::cmp::min(n_max, n_winners);
        }
      }
      (n_winners, pct_split.clone())
    },
    WinnerSelection::Percent { pct_player_count } => {
      let n_winners = std::cmp::max(1, game.player_count * (pct_player_count as u32) / 100);
      (n_winners, vec![])
    },
  };

  let indices = INDICES.load(storage)?;
  let mut n_found = 0u32;
  let mut rng = pcg64_from_game_seed(&game.seed)?;
  let mut visited: HashSet<u32> = HashSet::with_capacity(n_winners as usize);

  while n_found < n_winners {
    let i = rng.next_u64() % indices.len() as u64;
    let address_index = indices[i as usize];
    let already_selected = visited.contains(&address_index);
    if !game.has_distinct_winners || !already_selected {
      let addr = INDEX_2_ADDR.load(storage, address_index)?;
      if addr == *sender && is_suspect {
        return Err(ContractError::NotAuthorized {});
      }
      let player = PLAYERS.load(storage, addr.clone())?;
      let claim_amount = allocate_reward(game, total_reward, n_found, &pct_split);
      visited.insert(address_index);
      WINNERS.save(
        storage,
        n_found,
        &Winner {
          address: addr,
          ticket_count: player.ticket_count,
          position: n_found,
          has_claimed: false,
          claim_amount,
        },
      )?;
      n_found += 1
    }
  }

  Ok(n_found)
}

/// Based on a winner's position and the selection method in play, return the
/// portion of the jackpot that the winner is entitled to claim.
fn allocate_reward(
  game: &Game,
  total_reward: Uint128,
  position: u32,
  pct_split: &Vec<u8>,
) -> Uint128 {
  match game.selection {
    WinnerSelection::Fixed { .. } => {
      // calculate the propertion of the total reward to which the
      // player is entitled based on their percent split.
      if let Some(integer_percent) = pct_split.get(position as usize) {
        Uint128::from(*integer_percent) * total_reward / Uint128::from(100u8)
      } else {
        Uint128::zero()
      }
    },
    WinnerSelection::Percent { pct_player_count } => {
      // each winner gets a fixed uniform percent of the jackpot
      let n_winners = Uint128::from(pct_player_count as u32 * game.player_count as u32 / 100);
      total_reward / n_winners
    },
  }
}
