use crate::error::ContractError;
use crate::msg::ExecuteMsg;
use crate::state::{GameState, GameStatus, GAME};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response};

/// Anyone can end a game so long as (1) the game hasn't already ended and (2)
/// the creation time of the current block occurs after the game's `ends_after`
/// timestamp (stored as seconds since start of epoch).
pub fn execute(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
  let ended_at = env.block.time.seconds();
  GAME.update(deps.storage, |mut game| -> Result<_, ContractError> {
    validate_game_state_update(&game, &env)?;
    update_game_state(&mut game, ended_at, &info.sender);
    return Ok(game);
  })?;
  Ok(
    Response::new()
      .add_attribute("method", "end_game")
      .add_attribute("ended_at", ended_at.to_string())
      .add_attribute("ended_by", info.sender),
  )
}

/// Is the game in a valid state to be ended?
fn validate_game_state_update(
  game: &GameState,
  env: &Env,
) -> Result<(), ContractError> {
  if env.block.time.seconds() <= game.ends_after {
    return Err(ContractError::AlreadyEnded {});
  } else if game.status != GameStatus::ACTIVE {
    return Err(ContractError::NotActive {});
  }
  Ok(())
}

/// Update game state, effectively "ending" it.
fn update_game_state(
  game: &mut GameState,
  ended_at: u64,
  ended_by: &Addr,
) {
  game.status = GameStatus::ENDED;
  game.ended_at = Some(ended_at);
  game.ended_by = Some(ended_by.clone());
}
