use crate::error::ContractError;
use crate::msg::{InstantiateMsg, WinnerSelection};
use crate::random;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
  ACTIVE,
  ENDED,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Game {
  pub owner: Addr,
  pub name: Option<String>,
  pub id: String,
  pub status: GameStatus,
  pub selection: WinnerSelection,
  pub player_count: u32,
  pub ended_at: Option<Timestamp>,
  pub ended_by: Option<Addr>,
  pub denom: String,
  pub cw20_token_address: Option<Addr>,
  pub ticket_price: Uint128,
  pub ticket_count: u32,
  pub seed: String,
  pub ends_after: Option<Timestamp>,
  pub has_distinct_winners: bool,
  pub max_tickets_per_player: Option<u32>,
  pub funding_threshold: Option<Uint128>,
}

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

pub const GAME: Item<Game> = Item::new("game");
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
    seed: random::seed::init(&msg.id, env.block.height),
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

  GAME.save(deps.storage, &game)?;
  ORDERS.save(deps.storage, &vec![])?;
  INDICES.save(deps.storage, &vec![])?;

  Ok(game)
}

impl Game {}
