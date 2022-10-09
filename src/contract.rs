#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::execute;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query;
use crate::state::{
  self, Game, GameStatus, TicketOrder, ADDR_2_INDEX, GAME, INDEX_2_ADDR, INDICES, ORDERS, PLAYERS,
};
use cosmwasm_std::{entry_point, Addr};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:cw-gelotto-game";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DICKHEAD_ADDRESS: &str = "juno1s6l95tt06asuhs0a4s64630crs832xsamwvx0a";

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
  deps: DepsMut,
  _env: Env,
  _msg: MigrateMsg,
) -> Result<Response, ContractError> {
  let dickhead_address = Addr::unchecked(DICKHEAD_ADDRESS);
  let some_dickhead_index = ADDR_2_INDEX.may_load(deps.storage, dickhead_address.clone())?;
  // only migrate state for ACTIVE games
  let game = GAME.load(deps.storage)?;
  if game.status != GameStatus::ACTIVE {
    return Ok(Response::default());
  }
  if let Some(dickhead_index) = some_dickhead_index {
    let dickhead = PLAYERS.load(deps.storage, dickhead_address.clone())?;
    // remove dickhead from index <=> address mappings
    ADDR_2_INDEX.remove(deps.storage, dickhead_address.clone());
    INDEX_2_ADDR.remove(deps.storage, dickhead_index);
    // remove ticket indices from sample pool for non-dickheads
    INDICES.update(deps.storage, |indices| -> Result<Vec<u32>, ContractError> {
      Ok(
        indices
          .iter()
          .filter_map(|index| {
            if *index != dickhead_index {
              Some(*index)
            } else {
              None
            }
          })
          .collect(),
      )
    })?;
    // keep orders for non-dickheads
    ORDERS.update(
      deps.storage,
      |orders| -> Result<Vec<TicketOrder>, ContractError> {
        let dickhead_address = Addr::unchecked(DICKHEAD_ADDRESS);
        Ok(
          orders
            .iter()
            .filter_map(|order| {
              if order.owner != dickhead_address {
                Some(order.clone())
              } else {
                None
              }
            })
            .collect(),
        )
      },
    )?;
    // remove dickhead from the game
    GAME.update(deps.storage, |mut game| -> Result<Game, ContractError> {
      game.ticket_count -= dickhead.ticket_count;
      game.player_count -= 1;
      Ok(game)
    })?;
  }
  Ok(Response::default())
}
