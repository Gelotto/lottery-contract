use crate::state::{Game, TicketOrder};
use base64ct::{Base64, Encoding};
use cosmwasm_std::{Addr, Timestamp};
use sha2::{Digest, Sha256};

pub fn init(
  game_id: &String,
  time: &Timestamp,
) -> String {
  let mut sha256 = Sha256::new();
  sha256.update(game_id.as_bytes());
  sha256.update(time.nanos().to_le_bytes());
  let hash = sha256.finalize();
  Base64::encode_string(&hash)
}

pub fn update(
  game: &Game,
  order: &TicketOrder,
  time: &Timestamp,
) -> String {
  let mut sha256 = Sha256::new();
  sha256.update(game.seed.as_bytes());
  sha256.update(order.owner.as_bytes());
  sha256.update(order.count.to_le_bytes());
  sha256.update(time.nanos().to_le_bytes());
  let hash = sha256.finalize();
  Base64::encode_string(&hash)
}

pub fn finalize(
  game: &Game,
  sender: &Addr,
  time: &Timestamp,
) -> String {
  let mut sha256 = Sha256::new();
  sha256.update(game.seed.as_bytes());
  sha256.update(sender.as_bytes());
  sha256.update(&[0]);
  sha256.update(time.nanos().to_le_bytes());
  let hash = sha256.finalize();
  Base64::encode_string(&hash)
}
