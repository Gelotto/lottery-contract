use crate::state::Winner;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// WinnerSelection defines the number of and manner in which winners are chosen
/// when a game ends.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WinnerSelection {
  Fixed {
    // Ex: [60, 30, 10] means 60% to 1st place, 30% to 2nd, 10% to 3rd
    pct_split: Vec<u8>,
    winner_count: u32,
  },
  Percent {
    // Ex: 2 means that max(1, 0.02 * player_count) win
    pct_player_count: u8,
  },
}

/// Initial contract state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
  pub id: String,
  pub name: Option<String>,
  pub duration_minutes: u32,
  pub denom: String,
  pub ticket_price: String,
  pub selection: WinnerSelection,
}

/// Executable contract endpoints.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
  EndGame {
    lucky_phrase: Option<String>,
  },
  BuyTickets {
    ticket_count: u32,
    lucky_phrase: Option<String>,
  },
  ClaimPrize {},
}

/// Custom contract query endpoints.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
  GetWinners {},
  GetPlayers {},
  GetPlayerTicketCount { addr: Addr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetWinnersResponse {
  pub winners: Vec<Winner>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PlayerResponse {
  pub address: Addr,
  pub ticket_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetPlayersResponse {
  pub players: Vec<PlayerResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetTicketCountResponse {
  pub ticket_count: u32,
}
