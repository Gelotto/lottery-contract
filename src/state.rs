use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::random;
use cosmwasm_std::Addr;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Storage};
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
  pub winner_count: u64,
  pub ends_after: u64,
  pub ended_at: Option<u64>,
  pub ended_by: Option<Addr>,
  pub player_count: u64,
  pub seed: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PlayerState {
  address: Addr,
  ticket_count: u64,
}

pub const GAME: Item<GameState> = Item::new("game");
pub const PLAYER_ADDR_2_INDEX: Map<Addr, usize> = Map::new("player_addr_2_index");
pub const PLAYERS: Item<Vec<PlayerState>> = Item::new("players");
pub const WINNERS: Map<Addr, bool> = Map::new("winners");

/// Initialize contract state data.
pub fn initialize(
  deps: DepsMut,
  env: &Env,
  info: &MessageInfo,
  msg: &InstantiateMsg,
) -> Result<(), ContractError> {
  // add contract as player 0 with one ticket
  let players = vec![PlayerState {
    address: info.sender.clone(),
    ticket_count: 1,
  }];

  // init the game
  let game = GameState {
    owner: info.sender.clone(),
    status: GameStatus::ACTIVE,
    id: msg.id.clone(),
    ends_after: msg.ends_after,
    winner_count: msg.winner_count,
    ended_at: None,
    ended_by: None,
    seed: random::seed::init(&info.sender, &env.block.time),
    player_count: 0,
  };

  GAME.save(deps.storage, &game)?;
  PLAYERS.save(deps.storage, &players)?;
  PLAYER_ADDR_2_INDEX.save(deps.storage, info.sender.clone(), &0)?;

  Ok(())
}

/// Fetch a player's state from their wallet address.
pub fn get_player(
  store: &dyn Storage,
  addr: Addr,
) -> Option<PlayerState> {
  match PLAYER_ADDR_2_INDEX.load(store, addr) {
    Ok(index) => match PLAYERS.load(store) {
      Ok(players) => {
        if index >= players.len() {
          return None;
        }
        if let Some(player) = players.get(index) {
          return Some(player.clone());
        }
        return None;
      },
      Err(_) => {
        return None;
      },
    },
    Err(_) => {
      return None;
    },
  }
}

/// Return a vector of player addresses, given a vector of corresponding PLAYER
/// vector indices.
pub fn get_player_addresses_from_indices(
  store: &dyn Storage,
  indices: Vec<usize>,
) -> Vec<Addr> {
  let mut addresses: Vec<Addr> = Vec::with_capacity(indices.len());
  match PLAYERS.load(store) {
    Ok(players) => indices.iter().for_each(|i| {
      if let Some(player) = players.get(*i) {
        addresses.push((*player).address.clone());
      }
    }),
    Err(_) => {},
  }
  return addresses;
}
