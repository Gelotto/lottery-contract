#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::end_game;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{GameState, GameStatus, GAME, PLAYER_ADDR_2_TICKET_COUNT};

const CONTRACT_NAME: &str = "crates.io:cw-gelotto-game";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  msg: InstantiateMsg,
) -> Result<Response, ContractError> {
  let game = GameState {
    owner: info.sender.clone(),
    status: GameStatus::ACTIVE,
    id: msg.id.clone(),
    ends_after: msg.ends_after,
    winner_count: msg.winner_count,
    ended_at: None,
    ended_by: None,
  };

  set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

  GAME.save(deps.storage, &game)?;
  PLAYER_ADDR_2_TICKET_COUNT.save(deps.storage, info.sender.clone(), &1)?;

  Ok(
    Response::new()
      .add_attribute("method", "instantiate")
      .add_attribute("id", msg.id)
      .add_attribute("owner", info.sender)
      .add_attribute("winner_count", msg.winner_count.to_string())
      .add_attribute("ends_after", msg.ends_after.to_string()),
  )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: ExecuteMsg,
) -> Result<Response, ContractError> {
  match msg {
    ExecuteMsg::EndGame {} => end_game::execute(deps, env, info, msg),
  }
}
