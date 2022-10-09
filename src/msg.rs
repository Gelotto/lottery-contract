use crate::state::{TicketOrder, Winner};
use cosmwasm_std::{Addr, Uint128};
use cw_lottery_lib::game::{Game, Style, WinnerSelection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundStyle {
  Image { uri: String },
  Color { hex: String },
}

/// Initial contract state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
  pub code_id: u32,
  pub registry_contract_address: Option<Addr>,
  pub id: String,
  pub name: Option<String>,
  pub duration_minutes: Option<u32>,
  pub denom: String,
  pub cw20_token_address: Option<Addr>,
  pub ticket_price: String,
  pub selection: WinnerSelection,
  pub has_distinct_winners: bool,
  pub max_tickets_per_player: Option<u32>,
  pub funding_threshold: Option<Uint128>,
  pub style: Style,
}

/// Executable contract endpoints.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
  EndGame {},
  BuyTickets {
    ticket_count: u32,
    lucky_phrase: Option<String>,
  },
  ClaimPrize {
    positions: Vec<u32>,
  },
}

/// Custom contract query endpoints.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
  GetWinners {},
  GetPlayers {},
  GetPlayerTicketCount { addr: Addr },
  GetRound { round: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GameRound {
  pub game: Game,
  pub activity: Vec<TicketOrder>,
  pub winners: Vec<Winner>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum RegistryQueryMsg {
  GetGame {},
}
