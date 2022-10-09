#[cfg(not(feature = "library"))]
use crate::constants::LOTTERY_REGISTRY_CONTRACT_ADDRESS;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query;
use crate::{execute, state};
use cosmwasm_std::{entry_point, Addr, SubMsg, WasmMsg};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw_lottery_lib::msg::RegistryExecuteMsg;

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

  // initialize lottery game data
  let game = state::initialize(deps, &env, &info, &msg)?;

  // register lottery with registration contract with SubMsg
  let on_create_lottery_msg = RegistryExecuteMsg::OnCreateLottery {
    code_id: msg.code_id,
    addr: env.contract.address.clone(),
    game: game.clone(),
  };
  let on_create_lottery_wasm_msg = WasmMsg::Execute {
    contract_addr: msg
      .registry_contract_address
      .unwrap_or(Addr::unchecked(LOTTERY_REGISTRY_CONTRACT_ADDRESS))
      .clone()
      .into(),
    msg: to_binary(&on_create_lottery_msg)?,
    funds: vec![],
  };

  Ok(
    Response::new()
      .add_submessage(SubMsg::new(on_create_lottery_wasm_msg))
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
    QueryMsg::GetRound { round } => to_binary(&query::get_round(deps, round)?),
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
