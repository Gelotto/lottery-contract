use crate::error::ContractError;
use crate::random;
use crate::state::{Player, TicketOrder, GAME, ORDERS, PLAYERS};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub fn execute_buy_tickets(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  ticket_count: u32,
) -> Result<Response, ContractError> {
  // owner of the pending TicketOrder:
  let owner = info.sender.clone();

  if PLAYERS.has(deps.storage, owner.clone()) {
    // update player's ticket count
    PLAYERS.update(
      deps.storage,
      owner.clone(),
      |p| -> Result<_, ContractError> {
        let mut player = p.unwrap_or_else(|| Player { ticket_count: 0 });
        player.ticket_count += ticket_count;
        Ok(player)
      },
    )?;
  }
  // insert Player with initial ticket count
  else {
    PLAYERS.save(deps.storage, owner.clone(), &Player { ticket_count })?;

    // update game's player count and PRNG seed
    GAME.update(deps.storage, |mut game| -> Result<_, ContractError> {
      if game.has_ended(env.block.time) {
        return Err(ContractError::AlreadyEnded {});
      }
      game.seed = random::seed::update(&game, &owner, ticket_count, env.block.height);
      game.player_count += 1;
      Ok(game)
    })?;
  }

  // add a TicketOrder to specialized `ORDERS` vec, used in `end_game`
  // when performing binary search to find winners.
  ORDERS.update(
    deps.storage,
    |mut orders: Vec<TicketOrder>| -> Result<_, ContractError> {
      orders.push(TicketOrder {
        owner: owner,
        count: ticket_count,
        cum_count: (ticket_count as u64)
          + if orders.len() > 0 {
            orders[orders.len() - 1].cum_count
          } else {
            0
          },
      });
      Ok(orders)
    },
  )?;

  Ok(Response::new().add_attribute("action", "buy_tickets"))
}
