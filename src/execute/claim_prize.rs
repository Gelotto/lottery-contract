use crate::error::ContractError;
use crate::state::{Game, GameStatus, GAME, WINNERS};
use cosmwasm_std::{
  attr, to_binary, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

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
  let response = match game.cw20_token_address {
    Some(cw20_token_address) => {
      let transfer = Cw20ExecuteMsg::Transfer {
        recipient: info.sender.clone().into(),
        amount: claimed_amount,
      };

      let execute_msg = WasmMsg::Execute {
        contract_addr: cw20_token_address.clone().into(),
        msg: to_binary(&transfer)?,
        funds: vec![],
      };

      Response::new()
        .add_submessage(SubMsg::new(execute_msg))
        .add_attributes(vec![
          attr("action", "claim_prize"),
          attr("claimed_amount", claimed_amount.to_string()),
          attr("to", info.sender.clone()),
        ])
    },
    None => Response::new()
      .add_message(CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.clone().into(),
        amount: vec![Coin::new(claimed_amount.u128(), game.denom)],
      }))
      .add_attributes(vec![
        attr("action", "claim_prize"),
        attr("claimed_amount", claimed_amount.to_string()),
        attr("to", info.sender.clone()),
      ]),
  };

  Ok(response)
}
