use crate::msg::GameRound;
use crate::state::{query_game, ORDERS, WINNERS};
use cosmwasm_std::{Deps, Order, StdResult};
use cw_lottery_lib::game::GameStatus;

pub fn get_round(
  deps: Deps,
  _round: u32,
) -> StdResult<GameRound> {
  let game = query_game(&deps.querier)?;
  let activity = ORDERS.load(deps.storage)?;
  let winners = if game.status == GameStatus::ENDED {
    WINNERS
      .range(deps.storage, None, None, Order::Ascending)
      .map(|entry| entry.unwrap().1)
      .collect()
  } else {
    vec![]
  };

  Ok(GameRound {
    game,
    activity,
    winners,
  })
}
