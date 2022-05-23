use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
  ACTIVE,
  ENDED,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GameState {
  pub owner: Addr,
  pub id: String,
  pub status: GameStatus,
  pub winner_count: u32,
  pub ends_after: u64,
  pub ended_at: Option<u64>,
  pub ended_by: Option<Addr>,
}

pub const GAME: Item<GameState> = Item::new("game");
pub const PLAYER_ADDR_2_TICKET_COUNT: Map<Addr, u16> = Map::new("player_addr_2_ticket_count");
