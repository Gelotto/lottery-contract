use crate::msg::{InstantiateMsg, RegistryQueryMsg};
use crate::random;
use crate::{constants::LOTTERY_REGISTRY_CONTRACT_ADDRESS, error::ContractError};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, StdResult, Uint128};
use cw_lottery_lib::game::{Game, GameStatus};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TicketOrder {
  pub owner: Addr,
  pub count: u32,
  pub cum_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Player {
  pub ticket_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Winner {
  pub address: Addr,
  pub position: u32,
  pub ticket_count: u32,
  pub has_claimed: bool,
  pub claim_amount: Uint128,
}

pub const SEED: Item<String> = Item::new("seed");
pub const ORDERS: Item<Vec<TicketOrder>> = Item::new("orders");
pub const WINNERS: Map<u32, Winner> = Map::new("winners");
pub const PLAYERS: Map<Addr, Player> = Map::new("players");
pub const ADDR_2_INDEX: Map<Addr, u32> = Map::new("addr_2_index");
pub const INDEX_2_ADDR: Map<u32, Addr> = Map::new("index_2_addr");
pub const INDICES: Item<Vec<u32>> = Item::new("indices");

/// Initialize contract state data.
pub fn initialize(
  deps: DepsMut,
  env: &Env,
  info: &MessageInfo,
  msg: &InstantiateMsg,
) -> Result<Game, ContractError> {
  let game = Game {
    style: msg.style.clone(),
    name: msg.name.clone(),
    owner: info.sender.clone(),
    status: GameStatus::ACTIVE,
    id: msg.id.clone(),
    selection: msg.selection.clone(),
    ticket_price: Uint128::try_from(&msg.ticket_price[..])?,
    ends_after: match msg.duration_minutes {
      Some(duration_minutes) => Some(env.block.time.plus_seconds(60 * duration_minutes as u64)),
      None => None,
    },
    denom: msg.denom.clone(),
    cw20_token_address: msg.cw20_token_address.clone(),
    max_tickets_per_player: msg.max_tickets_per_player.clone(),
    has_distinct_winners: msg.has_distinct_winners,
    funding_threshold: msg.funding_threshold.clone(),
    player_count: 0,
    ticket_count: 0,
    ended_at: None,
    ended_by: None,
  };

  SEED.save(deps.storage, &random::seed::init(&msg.id, env.block.height))?;
  ORDERS.save(deps.storage, &vec![])?;
  INDICES.save(deps.storage, &vec![])?;

  Ok(game)
}

pub fn query_game(deps: &DepsMut) -> StdResult<Game> {
  deps.querier.query_wasm_smart(
    LOTTERY_REGISTRY_CONTRACT_ADDRESS,
    &RegistryQueryMsg::GetGame {},
  )
}
