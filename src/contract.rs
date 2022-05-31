#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::execute;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state;
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:cw-gelotto-game";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: InstantiateMsg,
) -> Result<Response, ContractError> {
  set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
  state::initialize(deps, &env, &info, &msg)?;
  Ok(
    Response::new()
      .add_attribute("action", "instantiate")
      .add_attribute("owner", info.sender)
      .add_attribute("id", msg.id),
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
    ExecuteMsg::EndGame {} => execute::end_game(deps, env, info),
    ExecuteMsg::BuyTickets { ticket_count } => execute::buy_tickets(deps, env, info, ticket_count),
    ExecuteMsg::ClaimPrize {} => execute::claim_prize(deps, env, info),
  }
}
