use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("NotAuthorized")]
  NotAuthorized {},

  #[error("NotActive")]
  NotActive {},

  #[error("AlreadyEnded")]
  AlreadyEnded {},

  #[error("StateLoadError")]
  StateLoadError {},

  #[error("StateSaveError")]
  StateSaveError {},
}
