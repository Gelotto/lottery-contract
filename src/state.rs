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
  pub ends_after: Timestamp,
  pub ended_at: Option<Timestamp>,
  pub ended_by: Option<Addr>,
  pub denom: String,
  pub ticket_price: Uint128,
  pub ticket_count: u32,
  pub seed: String,
  pub max_tickets_per_player: Option<u32>,
  pub has_distinct_winners: bool,
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
pub const WINNERS: Map<Addr, Winner> = Map::new("winners");
pub const PLAYERS: Map<Addr, Player> = Map::new("players");

/// Initialize contract state data.
pub fn initialize(
  deps: DepsMut,
  env: &Env,
  info: &MessageInfo,
  msg: &InstantiateMsg,
) -> Result<(), ContractError> {
  let game = Game {
    seed: random::seed::init(&msg.id, env.block.height),
    name: msg.name.clone(),
    owner: info.sender.clone(),
    status: GameStatus::ACTIVE,
    id: msg.id.clone(),
    selection: msg.selection.clone(),
    ticket_price: Uint128::try_from(&msg.ticket_price[..])?,
    ends_after: env
      .block
      .time
      .plus_seconds(60 * msg.duration_minutes as u64),
    denom: msg.denom.clone(),
    max_tickets_per_player: msg.max_tickets_per_player.clone(),
    has_distinct_winners: msg.has_distinct_winners,
    player_count: 0,
    ticket_count: 0,
    ended_at: None,
    ended_by: None,
  };

  GAME.save(deps.storage, &game)?;
  ORDERS.save(deps.storage, &vec![])?;

  Ok(())
}

impl Game {
  /// Has the game "ended". If so, it implies that no more tickets can be
  /// ordered and winners have been chosen.
  pub fn has_ended(
    &self,
    time: Timestamp,
  ) -> bool {
    time > self.ends_after
  }

  /// Calculates the max number of players who are elegible to be counted as
  /// winners when the game ends.
  pub fn compute_winner_count(&self) -> u32 {
    match self.selection {
      WinnerSelection::Fixed { winner_count, .. } => {
        std::cmp::min(self.player_count, winner_count as u32)
      },
      WinnerSelection::Percent { pct_player_count } => {
        std::cmp::max(1, self.player_count * (pct_player_count as u32) / 100)
      },
    }
  }
}
