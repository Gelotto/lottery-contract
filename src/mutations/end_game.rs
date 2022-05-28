use crate::error::ContractError;
use crate::random;
use crate::state::{Game, GameStatus, TicketOrder, GAME, ORDERS, WINNERS};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, Timestamp};
use rand_core::RngCore;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

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
  let n_winners = std::cmp::min(game.winner_count, game.player_count) as usize;
  let mut winners = Vec::with_capacity(n_winners);
  let mut rng: Pcg64 = Seeder::from(&game.seed).make_rng();
  while winners.len() < n_winners {
    let x = rng.next_u64() % (orders.len() as u64);
    let slice = &orders[..];
    let winner: Addr = binary_search(slice, orders.len(), x);
    winners.push(winner);
  }
  Ok(winners)
}

/// Return the owner address of the ticket order whose interval contains x.
fn binary_search(
  orders: &[TicketOrder],
  len: usize,
  x: u64,
) -> Addr {
  let i = len / 2;
  let order = &orders[i];
  if i > 0 {
    let prev_cum_count = orders[i - 1].cum_count;
    if x < prev_cum_count {
      return binary_search(&orders[..i], i, x);
    }
  }
  if x >= order.cum_count {
    return binary_search(&orders[i + 1..len], len - i, x);
  }
  order.owner.clone()
}
