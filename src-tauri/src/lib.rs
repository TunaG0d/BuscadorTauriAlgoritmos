pub mod algorithms;
pub mod benchmark;
pub mod config;
pub mod file_ops;
pub mod models;
pub mod ranking;
pub mod tree;

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use algorithms::{create_searcher, Searcher};
use models::{AppConfig, FileEntry};

pub struct AppState {
    pub files: Mutex<Vec<FileEntry>>,
    pub searcher: Mutex<Box<dyn Searcher>>,
    pub config: Mutex<AppConfig>,
    pub cancel_flag: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        let cfg = config::load_config();
        let searcher = create_searcher(cfg.algorithm);
        Self {
            files: Mutex::new(Vec::new()),
            searcher: Mutex::new(searcher),
            config: Mutex::new(cfg),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

