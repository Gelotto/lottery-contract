use crate::error::ContractError;
use base64ct::{Base64, Encoding};

mod pcg64;
pub mod seed;

pub use pcg64::Pcg64;

pub fn pcg64_from_game_seed(seed: &String) -> Result<Pcg64, ContractError> {
  match Base64::decode_vec(seed) {
    Ok(bytes_vec) => {
      let mut bytes = [0u8; 32];
      bytes.copy_from_slice(bytes_vec.as_slice());
      Ok(Pcg64::from_seed(bytes))
    },
    Err(_err) => Err(ContractError::InvalidSeed { seed: seed.clone() }),
  }
}
