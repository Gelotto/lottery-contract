use crate::constants::GELOTTO_GAME_FUND_ADDR;
use crate::error::ContractError;
use crate::random;
use crate::random::pcg64_from_game_seed;
use crate::state::{
  Game, GameStatus, Player, TicketOrder, Winner, GAME, ORDERS, PLAYERS, PRIZE, WINNERS,
};
use cosmwasm_std::{
  attr, Addr, BankMsg, BlockInfo, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Order, Response,
  Storage, Uint128,
};

const DEBUG: bool = false;

/// Anyone can end a game so long as (1) the game hasn't already ended and (2)
/// the creation time of the block is later than the game's `ends_after`
/// timestamp (stored as nanoseconds).
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

  // find N winners and store in state
  let n_winners = select_winners(deps.storage, &game)?;

  // get total prize balance
  let pot: Coin = deps
    .querier
    .query_balance(env.contract.address.clone(), game.denom.clone())?;

  // calculate amount owed to Gelotto's gaming fund & amount owed to each winner
  let gelotto_amount = pot.amount / Uint128::from(10u128);
  let winner_prize_amount = (pot.amount - gelotto_amount) / Uint128::from(n_winners);

  // set prize amount owed to each winner
  PRIZE.update(deps.storage, |mut prize| -> Result<_, ContractError> {
    prize.amount = winner_prize_amount;
    Ok(prize)
  })?;

  if DEBUG {
    return Ok(Response::new().add_attributes(vec![
      attr("action", "end_game"),
      attr("to", info.sender.clone()),
    ]));
  }

  // transfer Gelotto's 10% to its gaming fund
  Ok(
    Response::new()
      // .add_message(CosmosMsg::Bank(BankMsg::Send {
      //   to_address: GELOTTO_GAME_FUND_ADDR.clone().into(),
      //   amount: vec![Coin::new(gelotto_amount.into(), game.denom.clone())],
      // }))
      .add_attributes(vec![
        attr("action", "end_game"),
        attr("to", info.sender.clone()),
      ]),
  )
}

/// Is the game in a valid state to be ended?
fn authorize_and_validate(
  game: &Game,
  env: &Env,
) -> Result<(), ContractError> {
  if !DEBUG {
    if env.block.time <= game.ends_after {
      return Err(ContractError::NotAuthorized {});
    }
    if game.status != GameStatus::ACTIVE {
      return Err(ContractError::NotActive {});
    }
  }
  if game.player_count == 0 {
    return Err(ContractError::NoWinners {});
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
) -> Result<u32, ContractError> {
  let orders: Vec<TicketOrder> = ORDERS.load(storage)?;
  let mut rng = pcg64_from_game_seed(&game.seed)?;
  let n_winners = std::cmp::min(game.winner_count, game.player_count);
  let n_tickets_sold = orders[orders.len() - 1].cum_count;
  let mut n_winners_found: u32 = 0;

  let mut old_winner_addrs: Vec<Addr> = vec![];

  if DEBUG {
    WINNERS
      .keys(storage, None, None, Order::Ascending)
      .for_each(|result| {
        if result.is_ok() {
          old_winner_addrs.push(result.unwrap())
        }
      });

    old_winner_addrs.iter().for_each(|x| {
      WINNERS.remove(storage, x.clone());
    });
  }

  while n_winners_found < n_winners {
    let x = rng.next_u64() % n_tickets_sold;
    let winner: &TicketOrder = bisect(&orders[..], orders.len(), x);
    if !WINNERS.has(storage, winner.owner.clone()) {
      let player: Player = PLAYERS.load(storage, winner.owner.clone())?;
      WINNERS.save(
        storage,
        winner.owner.clone(),
        &Winner {
          address: winner.owner.clone(),
          ticket_count: player.ticket_count,
          position: n_winners_found,
          has_claimed: false,
        },
      )?;
      n_winners_found += 1;
    }
  }
  Ok(n_winners)
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
