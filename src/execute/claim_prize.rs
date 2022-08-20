use crate::error::ContractError;
use crate::state::{Game, GameStatus, GAME, WINNERS};
use cosmwasm_std::{attr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128};

pub fn execute_claim_prize(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  positions: &Vec<u32>,
) -> Result<Response, ContractError> {
  let game: Game = GAME.load(deps.storage)?;

  // abort if the game is still active
  if game.status != GameStatus::ENDED {
    return Err(ContractError::NotAuthorized {});
  }

  // total amount claimed by the sender
  let mut claimed_amount = Uint128::zero();

  // iterate through all "positions" won by the sender,
  // computing the total amount to be claimed
  for position in positions.iter() {
    let mut winner = WINNERS.load(deps.storage, *position)?;
    if winner.address != info.sender {
      return Err(ContractError::NotAuthorized {});
    }
    if !winner.has_claimed {
      winner.has_claimed = true;
      claimed_amount += winner.claim_amount;
      WINNERS.save(deps.storage, *position, &winner)?;
    }
  }

  // transfer balance to the winner
  Ok(
    Response::new()
      .add_message(CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.clone().into(),
        amount: vec![Coin::new(claimed_amount.u128(), game.denom)],
      }))
      .add_attributes(vec![
        attr("action", "claim_prize"),
        attr("claimed_amount", claimed_amount.to_string()),
        attr("to", info.sender.clone()),
      ]),
  )
}
