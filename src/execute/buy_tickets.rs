use crate::error::ContractError;
use crate::random;
use crate::state::{Game, Player, TicketOrder, GAME, ORDERS, PLAYERS};
use cosmwasm_std::{attr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128};

/// Buy tickets. Tickets can be bought even after the `ends_after` date. Only
/// once the `end_game` endpoint has been executed does the game close to new
/// ticket orders.
pub fn execute_buy_tickets(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  ticket_count: u32,
  lucky_phrase: &Option<String>,
) -> Result<Response, ContractError> {
  let mut game: Game = GAME.load(deps.storage)?;
  let owner = info.sender.clone();

  // amount owed by player in exchange for the tickets:
  let payment_amount = game.ticket_price * Uint128::from(ticket_count);

  if PLAYERS.has(deps.storage, owner.clone()) {
    // update player's ticket count
    PLAYERS.update(
      deps.storage,
      owner.clone(),
      |p| -> Result<_, ContractError> {
        let mut player = p.unwrap_or_else(|| Player { ticket_count: 0 });
        if let Some(max_tickets_per_player) = game.max_tickets_per_player {
          // don't let player buy more tickets than max allowed, unless N/A
          if player.ticket_count + ticket_count > max_tickets_per_player {
            return Err(ContractError::ExceededMaxTicketsPerPlayer {});
          }
        }
        player.ticket_count += ticket_count;
        Ok(player)
      },
    )?;
  } else {
    // insert Player with initial ticket count
    // don't let player buy more tickets than max allowed, unless N/A
    if let Some(max_tickets_per_player) = game.max_tickets_per_player {
      if ticket_count > max_tickets_per_player {
        return Err(ContractError::ExceededMaxTicketsPerPlayer {});
      }
    }
    game.player_count += 1;
    PLAYERS.save(deps.storage, owner.clone(), &Player { ticket_count })?;
  }

  // update game's player count and PRNG seed
  game.seed = random::seed::update(&game, &owner, ticket_count, env.block.height, &lucky_phrase);
  game.ticket_count += ticket_count;

  GAME.save(deps.storage, &game)?;

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

  // transfer payment from player to the contract
  Ok(
    Response::new()
      .add_message(CosmosMsg::Bank(BankMsg::Send {
        to_address: env.contract.address.into_string(),
        amount: vec![Coin::new(payment_amount.u128(), game.denom)],
      }))
      .add_attributes(vec![
        attr("action", "buy_tickets"),
        attr("to", info.sender.clone()),
      ]),
  )
}
