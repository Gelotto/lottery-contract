use crate::error::ContractError;
use crate::random;
use crate::random::pcg64_from_game_seed;
use crate::state::{Game, GameStatus, TicketOrder, GAME, ORDERS, WINNERS};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, Timestamp};
// use rand_core::RngCore;
// use rand_pcg::Pcg64;
// use rand_seeder::Seeder;
use std::collections::HashSet;

/// Anyone can end a game so long as (1) the game hasn't already ended and (2)
/// the creation time of the block is later than the game's `ends_after`
/// timestamp (stored as nanoseconds).
pub fn execute(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
) -> Result<Response, ContractError> {
  let mut game: Game = GAME.load(deps.storage)?;

  authorize_and_validate(&game, &env)?;
  update_game_state(&mut game, &info.sender, &env.block.time);

  let orders: Vec<TicketOrder> = ORDERS.load(deps.storage)?;
  let winners = select_winners(&game, &orders)?;

  GAME.save(deps.storage, &game)?;
  WINNERS.save(deps.storage, &winners)?;

  Ok(
    Response::new()
      .add_attribute("method", "end_game")
      .add_attribute("ended_at", env.block.time.nanos().to_string())
      .add_attribute("ended_by", info.sender),
  )
}

/// Is the game in a valid state to be ended?
fn authorize_and_validate(
  game: &Game,
  env: &Env,
) -> Result<(), ContractError> {
  if env.block.time.nanos() <= game.ends_after {
    return Err(ContractError::NotAuthorized {});
  } else if game.status != GameStatus::ACTIVE {
    return Err(ContractError::NotActive {});
  }
  Ok(())
}

/// Update the game's state, effectively "ending" it.
fn update_game_state(
  game: &mut Game,
  sender: &Addr,
  time: &Timestamp,
) {
  game.status = GameStatus::ENDED;
  game.seed = random::seed::finalize(game, sender, time);
  game.ended_at = Some(time.nanos());
  game.ended_by = Some(sender.clone());
}

/// select the winners using game's seed
fn select_winners(
  game: &Game,
  orders: &Vec<TicketOrder>,
) -> Result<Vec<Addr>, ContractError> {
  let mut rng = pcg64_from_game_seed(&game.seed)?;
  let n_winners = std::cmp::min(game.winner_count, game.player_count) as usize;
  let n_tickets_sold = orders[orders.len() - 1].cum_count;
  let mut visited: HashSet<Addr> = HashSet::with_capacity(n_winners);
  let mut winners: Vec<Addr> = Vec::with_capacity(n_winners);
  while winners.len() < n_winners {
    let x = rng.next_u64() % n_tickets_sold;
    let winner: &TicketOrder = bisect(&orders[..], orders.len(), x);
    if !visited.contains(&winner.owner) {
      visited.insert(winner.owner.clone());
      winners.push(winner.owner.clone());
    }
  }
  Ok(winners)
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
