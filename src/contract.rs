#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TextResponse};
use crate::state::{State, STATE};

const CONTRACT_NAME: &str = "crates.io:cw-gelotto-game";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  msg: InstantiateMsg,
) -> Result<Response, ContractError> {
  let state = State {
    text: msg.text.clone(),
    owner: info.sender.clone(),
  };
  set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
  STATE.save(deps.storage, &state)?;
  Ok(
    Response::new()
      .add_attribute("method", "instantiate")
      .add_attribute("owner", info.sender)
      .add_attribute("text", msg.text),
  )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  msg: ExecuteMsg,
) -> Result<Response, ContractError> {
  match msg {
    ExecuteMsg::Write { text } => set_text(deps, info, text),
  }
}

pub fn set_text(
  deps: DepsMut,
  info: MessageInfo,
  text: String,
) -> Result<Response, ContractError> {
  STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
    if info.sender != state.owner {
      return Err(ContractError::NotAuthorized {});
    }
    state.text = text.clone();
    Ok(state)
  })?;
  Ok(Response::new().add_attribute("method", "write"))
}
