use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: usize,
    pub name: String,
    pub name_lower: String,
    pub path: String,
    pub size: u64,
    pub modified: u64,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlgorithmType {
    Linear,
    Binary,
    Hash,
    Avl,
    Bfs,
    Dfs,
}

impl AlgorithmType {
    pub fn name(&self) -> &'static str {
        match self {
            AlgorithmType::Linear => "Búsqueda lineal",
            AlgorithmType::Binary => "Búsqueda binaria",
            AlgorithmType::Hash => "Índice hash",
            AlgorithmType::Avl => "Árbol AVL",
            AlgorithmType::Bfs => "Recorrido BFS",
            AlgorithmType::Dfs => "Recorrido DFS",
        }
    }

    pub fn supports_substring(&self) -> bool {
        match self {
            AlgorithmType::Hash => false,
            AlgorithmType::Binary => false,
            _ => true,
        }
    }

    pub fn supports_prefix(&self) -> bool {
        match self {
            AlgorithmType::Hash => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryType {
    Exact,
    Prefix,
    Substring,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResult {
    pub id: usize,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: u64,
    pub score: u32,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexProgress {
    pub count: usize,
    pub current_path: String,
    pub finished: bool,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub algorithm: AlgorithmType,
    pub top_k: usize,
    pub last_folder: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            algorithm: AlgorithmType::Linear,
            top_k: 20,
            last_folder: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmComparison {
    pub algorithm_id: String,
    pub name: String,
    pub structure: String,
    pub build_time_ms: f64,
    pub search_time_ms: f64,
    pub memory_mb: f64,
    pub results_count: usize,
    pub supported: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreview {
    pub path: String,
    pub content: String,
    pub size: u64,
    pub truncated: bool,
    pub is_binary: bool,
}
