use crate::state::Winner;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
  pub id: String,
  pub duration_minutes: u32,
  pub winner_count: u32,
  pub denom: String,
  pub ticket_price: String,
}

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
