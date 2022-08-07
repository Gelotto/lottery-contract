use crate::msg::GetTicketCountResponse;
use crate::state::PLAYERS;
use cosmwasm_std::{Addr, Deps, StdResult};

pub fn get_player_ticket_count(
  deps: Deps,
  addr: Addr,
) -> StdResult<GetTicketCountResponse> {
  match PLAYERS.load(deps.storage, addr) {
    Ok(player) => Ok(GetTicketCountResponse {
      ticket_count: player.ticket_count,
    }),
    Err(_error) => Ok(GetTicketCountResponse { ticket_count: 0 }),
  }
}
