use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Clock before epoch")
        .as_secs()
}
