use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::random;
use cosmwasm_std::{Addr, Coin, DepsMut, Env, MessageInfo, Timestamp, Uint128};
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
  pub id: String,
  pub status: GameStatus,
  pub winner_count: u64,
  pub ends_after: u64,
  pub ended_at: Option<u64>,
  pub ended_by: Option<Addr>,
  pub player_count: u64,
  pub denom: String,
  pub ticket_price: u128,
  pub seed: String,
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
  pub player: Addr,
  pub position: u64,
  pub has_claimed: bool,
}

pub const GAME: Item<Game> = Item::new("game");
pub const ORDERS: Item<Vec<TicketOrder>> = Item::new("orders");
pub const WINNERS: Map<Addr, Winner> = Map::new("winners");
pub const PLAYERS: Map<Addr, Player> = Map::new("players");
pub const PRIZE: Item<Coin> = Item::new("prize");

/// Initialize contract state data.
pub fn initialize(
  deps: DepsMut,
  env: &Env,
  info: &MessageInfo,
  msg: &InstantiateMsg,
) -> Result<(), ContractError> {
  let game = Game {
    seed: random::seed::init(&msg.id, env.block.height),
    owner: info.sender.clone(),
    status: GameStatus::ACTIVE,
    id: msg.id.clone(),
    ends_after: msg.ends_after,
    winner_count: msg.winner_count,
    denom: msg.denom.clone(),
    player_count: 0,
    ticket_price: Uint128::try_from(&msg.ticket_price[..])?.u128(),
    ended_at: None,
    ended_by: None,
  };

  GAME.save(deps.storage, &game)?;
  // ORDERS.save(deps.storage, &vec![])?;
  // PRIZE.save(deps.storage, &Coin::new(0, msg.denom.clone()))?;

  Ok(())
}

impl Game {
  pub fn has_ended(
    &self,
    time: Timestamp,
  ) -> bool {
    time.nanos() > self.ends_after
  }
}
