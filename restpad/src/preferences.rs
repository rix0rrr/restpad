use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Preferences {
    /// Brightness on a scale from 0 to 8
    pub brightness: u8,
}

impl Default for Preferences {
    fn default() -> Self {
        Self { brightness: 8 }
    }
}
