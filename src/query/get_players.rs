use crate::msg::{GetPlayersResponse, PlayerResponse};
use crate::state::PLAYERS;
use cosmwasm_std::{Deps, Order, StdResult};

pub fn get_players(deps: Deps) -> StdResult<GetPlayersResponse> {
  let mut players: Vec<PlayerResponse> = vec![];
  PLAYERS
    .range(deps.storage, None, None, Order::Ascending)
    .for_each(|result| {
      if result.is_ok() {
        let (addr, player) = result.unwrap();
        players.push(PlayerResponse {
          address: addr,
          ticket_count: player.ticket_count,
        });
      }
    });

  Ok(GetPlayersResponse { players })
}
