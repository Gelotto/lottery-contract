use base64ct::{Base64, Encoding};
use cosmwasm_std::Addr;
use sha2::{Digest, Sha256};

pub fn init(
  game_id: &String,
  block_height: u64,
) -> String {
  let mut sha256 = Sha256::new();
  sha256.update(game_id.as_bytes());
  sha256.update(block_height.to_le_bytes());
  let hash = sha256.finalize();
  Base64::encode_string(&hash)
}

pub fn update(
  seed: &String,
  owner: &Addr,
  ticket_count: u32,
  block_height: u64,
  lucky_phrase: &Option<String>,
) -> String {
  let mut sha256 = Sha256::new();
  sha256.update(seed.as_bytes());
  sha256.update(owner.as_bytes());
  sha256.update(ticket_count.to_le_bytes());
  sha256.update(block_height.to_le_bytes());
  if let Some(lucky_phrase_str) = lucky_phrase {
    sha256.update(lucky_phrase_str.as_bytes());
  }
  let hash = sha256.finalize();
  Base64::encode_string(&hash)
}

pub fn finalize(
  seed: &String,
  sender: &Addr,
  block_height: u64,
) -> String {
  let mut sha256 = Sha256::new();
  sha256.update(seed.as_bytes());
  sha256.update(sender.as_bytes());
  sha256.update(block_height.to_le_bytes());
  let hash = sha256.finalize();
  Base64::encode_string(&hash)
}
