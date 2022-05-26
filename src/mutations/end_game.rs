use crate::error::ContractError;
use crate::random;
use crate::state::{get_player_addresses_from_indices, GameState, GameStatus, GAME, WINNERS};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, Timestamp};
use rand_core::RngCore;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

/// Anyone can end a game so long as (1) the game hasn't already ended and (2)
/// the creation time of the block is later than the game's `ends_after`
/// timestamp (stored as nanoseconds).
pub fn execute(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
) -> Result<Response, ContractError> {
  // update game state
  GAME.update(deps.storage, |mut game| -> Result<_, ContractError> {
    validate_request(&game, &env)?;
    update_game_state_to_ended(&mut game, &env.block.time, &info.sender);
    Ok(game)
  })?;

  // compute vector of winning player addresses
  match GAME.load(deps.storage) {
    Ok(game) => {
      // get number of winners to select
      let winner_count: u64 = std::cmp::min(game.winner_count, game.player_count);

      // initialize PRNG with the game's seed & choose winning player indices
      let mut rng: Pcg64 = Seeder::from(&game.seed).make_rng();
      let mut indices: Vec<usize> = Vec::with_capacity(winner_count as usize);
      for _ in 0..winner_count {
        let index = rng.next_u64() as usize;
        indices.push(index);
      }
      // insert winning player addresses into WINNERS map as keys
      let winners: Vec<Addr> = get_player_addresses_from_indices(deps.storage, indices);
      for addr in winners {
        WINNERS.save(deps.storage, addr, &true)?;
      }
    },
    Err(_) => {
      return Err(ContractError::StateLoadError {});
    },
  }

  Ok(
    Response::new()
      .add_attribute("method", "end_game")
      .add_attribute("ended_at", env.block.time.nanos().to_string())
      .add_attribute("ended_by", info.sender),
  )
}

/// Is the game in a valid state to be ended?
fn validate_request(
  game: &GameState,
  env: &Env,
) -> Result<(), ContractError> {
  if env.block.time.nanos() <= game.ends_after {
    return Err(ContractError::AlreadyEnded {});
  } else if game.status != GameStatus::ACTIVE {
    return Err(ContractError::NotActive {});
  }
  Ok(())
}

/// Update the game's state, effectively "ending" it.
fn update_game_state_to_ended(
  game: &mut GameState,
  ended_at: &Timestamp,
  ended_by: &Addr,
) {
  game.status = GameStatus::ENDED;
  game.ended_at = Some(ended_at.seconds());
  game.ended_by = Some(ended_by.clone());
  game.seed = random::seed::update(&game.seed, ended_by, ended_at, None);
}
