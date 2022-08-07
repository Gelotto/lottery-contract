use crate::msg::GetWinnersResponse;
use crate::state::{Winner, WINNERS};
use cosmwasm_std::{Deps, Order, StdResult};

pub fn get_winners(deps: Deps) -> StdResult<GetWinnersResponse> {
  let mut winners: Vec<Winner> = vec![];
  WINNERS
    .range(deps.storage, None, None, Order::Ascending)
    .for_each(|result| {
      if result.is_ok() {
        let (_, winner) = result.unwrap();
        winners.push(winner);
      }
    });

  Ok(GetWinnersResponse { winners })
}
