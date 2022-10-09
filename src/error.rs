use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("StateLoadError")]
  StateLoadError {},

  #[error("StateSaveError")]
  StateSaveError {},

  #[error("NotAuthorized")]
  NotAuthorized {},

  #[error("NotActive")]
  NotActive {},

  #[error("AlreadyEnded")]
  AlreadyEnded {},

  #[error("NoWinners")]
  NoWinners {},

  #[error("InvalidSeed")]
  InvalidSeed { seed: String },

  #[error("ExceededMaxTicketsPerPlayer")]
  ExceededMaxTicketsPerPlayer {},

  #[error("UnderFundingThreshold")]
  UnderFundingThreshold { funding_threshold: Uint128 },

  #[error("InsufficientFunds")]
  InsufficientFunds {},

  #[error("ExcessFunds")]
  ExcessFunds {},
}
