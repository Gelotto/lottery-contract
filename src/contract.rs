#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::execute;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query;
use crate::state;
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
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
    ExecuteMsg::EndGame { lucky_phrase } => execute::end_game(deps, env, info, &lucky_phrase),
    ExecuteMsg::ClaimPrize { positions } => execute::claim_prize(deps, env, info, &positions),
    ExecuteMsg::BuyTickets {
      ticket_count,
      lucky_phrase,
    } => execute::buy_tickets(deps, env, info, ticket_count, &lucky_phrase),
  }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
  deps: Deps,
  _env: Env,
  msg: QueryMsg,
) -> StdResult<Binary> {
  let result = match msg {
    QueryMsg::GetWinners {} => to_binary(&query::get_winners(deps)?),
    QueryMsg::GetPlayers {} => to_binary(&query::get_players(deps)?),
    QueryMsg::GetPlayerTicketCount { addr } => {
      to_binary(&query::get_player_ticket_count(deps, addr)?)
    },
  }?;
  Ok(result)
}

#[entry_point]
pub fn migrate(
  _deps: DepsMut,
  _env: Env,
  _msg: MigrateMsg,
) -> Result<Response, ContractError> {
  // No state migrations performed, just returned a Response
  Ok(Response::default())
}
