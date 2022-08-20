use crate::constants::GELOTTO_GAME_FUND_ADDR;
use crate::error::ContractError;
use crate::msg::WinnerSelection;
use crate::random;
use crate::random::pcg64_from_game_seed;
use crate::state::{Game, GameStatus, Player, TicketOrder, Winner, GAME, ORDERS, PLAYERS, WINNERS};
use cosmwasm_std::{
  attr, Addr, BankMsg, BlockInfo, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Storage,
  Uint128,
};
use std::collections::HashSet;

pub fn execute_end_game(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  lucky_phrase: &Option<String>,
) -> Result<Response, ContractError> {
  let mut game: Game = GAME.load(deps.storage)?;

  authorize_and_validate(&game, &env)?;
  update_game(
    deps.storage,
    &mut game,
    &info.sender,
    &env.block,
    lucky_phrase,
  )?;

  // get total prize balance
  let jackpot: Coin = deps
    .querier
    .query_balance(env.contract.address.clone(), game.denom.clone())?;

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
      Ok(
        Response::new()
          .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: ticket_order.owner.clone().into(),
            amount: vec![jackpot],
          }))
          .add_attributes(vec![attr("action", "end_game"), attr("winner_count", "1")]),
      )
    } else {
      Ok(
        Response::new().add_attributes(vec![attr("action", "end_game"), attr("winner_count", "0")]),
      )
    }
  } else {
    // calculate amount owed to Gelotto's gaming fund (10%)
    let gelotto_jackpot_amount = jackpot.amount / Uint128::from(10u128);

    // find N winners and store in state
    let n_winners = select_winners(deps.storage, &game, jackpot.amount - gelotto_jackpot_amount)?;

    Ok(
      Response::new()
        // transfer Gelotto's 10% to its gaming fund
        .add_message(CosmosMsg::Bank(BankMsg::Send {
          to_address: GELOTTO_GAME_FUND_ADDR.clone().into(),
          amount: vec![Coin::new(gelotto_jackpot_amount.into(), game.denom.clone())],
        }))
        .add_attributes(vec![
          attr("action", "end_game"),
          attr("winner_count", n_winners.to_string()),
        ]),
    )
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
  storage: &mut dyn Storage,
  game: &Game,
  total_reward: Uint128,
) -> Result<u32, ContractError> {
  let orders: Vec<TicketOrder> = ORDERS.load(storage)?;
  let n_tickets_sold = orders[orders.len() - 1].cum_count;

  let (n_winners, pct_split) = match game.selection.clone() {
    WinnerSelection::Fixed {
      winner_count,
      pct_split,
    } => {
      let n_winners = std::cmp::min(game.player_count, winner_count as u32);
      (n_winners, pct_split.clone())
    },
    WinnerSelection::Percent { pct_player_count } => {
      let n_winners = std::cmp::max(1, game.player_count * (pct_player_count as u32) / 100);
      (n_winners, vec![])
    },
  };

  let mut n_found = 0u32;
  let mut rng = pcg64_from_game_seed(&game.seed)?;
  let mut visited: HashSet<Addr> = HashSet::with_capacity(n_winners as usize);

  // keep picking winners (not necessarily distinct)
  while n_found < n_winners {
    let x = rng.next_u64() % n_tickets_sold;
    let ticket_order: &TicketOrder = bisect(&orders[..], orders.len(), x);
    let winner_exists = visited.contains(&ticket_order.owner);

    if !game.has_distinct_winners || !winner_exists {
      // create and save the winner
      let player: Player = PLAYERS.load(storage, ticket_order.owner.clone())?;
      let claim_amount = allocate_reward(game, total_reward, n_found, &pct_split);
      visited.insert(ticket_order.owner.clone());
      WINNERS.save(
        storage,
        n_found,
        &Winner {
          address: ticket_order.owner.clone(),
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

/// Return the owner address of the ticket order whose interval contains x.
fn bisect(
  orders: &[TicketOrder],
  n: usize, // == size of `orders` slice
  x: u64,
) -> &TicketOrder {
  let i = n / 2;
  let order = &orders[i];
  let lower = order.cum_count - order.count as u64;
  let upper = order.cum_count;
  if x < lower {
    // go left
    return bisect(&orders[..i], i, x);
  } else if x >= upper {
    // go right
    return bisect(&orders[i..], n - i, x);
  }
  &order
}
