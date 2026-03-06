use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassthroughConfig {
    pub enable_keyboard: bool,
    pub enable_mouse: bool,
    pub enable_video: bool,
}

impl Default for PassthroughConfig {
    fn default() -> Self {
        Self {
            enable_keyboard: true,
            enable_mouse: false,
            enable_video: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub lock_mode: bool,
    pub passthrough: PassthroughConfig,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            lock_mode: false,
            passthrough: PassthroughConfig::default(),
        }
    }
}

pub type SharedAppState = Arc<Mutex<AppState>>;

pub fn new_shared_state() -> SharedAppState {
    Arc::new(Mutex::new(AppState::default()))
}
