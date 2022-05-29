use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::random;
use cosmwasm_std::Addr;
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
  pub seed: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TicketOrder {
  pub owner: Addr,
  pub count: u32,
  pub cum_count: u64,
}

pub const GAME: Item<Game> = Item::new("game");
pub const ORDERS: Item<Vec<TicketOrder>> = Item::new("orders");
pub const WINNERS: Item<Vec<Addr>> = Item::new("winners");

/// Initialize contract state data.
pub fn initialize(
  deps: DepsMut,
  env: &Env,
  info: &MessageInfo,
  msg: &InstantiateMsg,
) -> Result<(), ContractError> {
  let game = Game {
    seed: random::seed::init(&msg.id, &env.block.time),
    owner: info.sender.clone(),
    status: GameStatus::ACTIVE,
    id: msg.id.clone(),
    ends_after: msg.ends_after,
    winner_count: msg.winner_count,
    player_count: 0,
    ended_at: None,
    ended_by: None,
  };

  GAME.save(deps.storage, &game)?;
  ORDERS.save(deps.storage, &vec![])?;
  WINNERS.save(deps.storage, &vec![])?;

  Ok(())
}
