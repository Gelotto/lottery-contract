pub mod seed {
  use cosmwasm_std::{Addr, Timestamp};
  use hex::encode;
  use sha2::{Digest, Sha256};

  pub fn init(
    sender: &Addr,
    time: &Timestamp,
  ) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sender.as_bytes());
    hasher.update(time.nanos().to_le_bytes());
    let hash = hasher.finalize();
    encode(hash)
  }

  pub fn update(
    prev_seed: &String,
    sender: &Addr,
    time: &Timestamp,
    ticket_count: Option<u64>,
  ) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prev_seed.as_bytes());
    hasher.update(sender.as_bytes());
    hasher.update(time.nanos().to_le_bytes());
    if let Some(n) = ticket_count {
      hasher.update(n.to_le_bytes());
    }
    let hash = hasher.finalize();
    encode(hash)
  }
}
