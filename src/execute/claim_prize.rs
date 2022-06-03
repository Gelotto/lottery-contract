use crate::error::ContractError;
use crate::state::{Game, GAME, PRIZE, WINNERS};
use cosmwasm_std::{attr, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response};

pub fn execute_claim_prize(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
) -> Result<Response, ContractError> {
  let game: Game = GAME.load(deps.storage)?;

  // abort if the game is still active
  if !game.has_ended(env.block.time) {
    return Err(ContractError::NotAuthorized {});
  }

  if let Some(mut winner) = WINNERS.may_load(deps.storage, info.sender.clone())? {
    // don't let the player claim multiple times
    if winner.has_claimed {
      return Err(ContractError::NotAuthorized {});
    }
    winner.has_claimed = true;
    WINNERS.save(deps.storage, info.sender.clone(), &winner)?;
  } else {
    // abort if the sender isn't a winner or the game hasn't ended
    return Err(ContractError::NotAuthorized {});
  }

  // transfer balance to the winner
  Ok(
    Response::new()
      .add_message(CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.clone().into(),
        amount: vec![PRIZE.load(deps.storage)?],
      }))
      .add_attributes(vec![
        attr("action", "claim_prize"),
        attr("to", info.sender.clone()),
      ]),
  )
}
