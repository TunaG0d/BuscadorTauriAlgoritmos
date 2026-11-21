pub mod avl;
pub mod bfs;
pub mod binary;
pub mod dfs;
pub mod hash_index;
pub mod linear;

pub use avl::AvlSearcher;
pub use bfs::BfsSearcher;
pub use binary::BinarySearcher;
pub use dfs::DfsSearcher;
pub use hash_index::HashSearcher;
pub use linear::LinearSearcher;

use crate::models::{AlgorithmType, FileEntry, QueryType, SearchResult};

pub trait Searcher: Send + Sync {
    fn algorithm_type(&self) -> AlgorithmType;
    fn build(&mut self, files: &[FileEntry]);
    fn search(&self, query: &str, query_type: QueryType, files: &[FileEntry], top_k: usize) -> Vec<SearchResult>;
    fn estimate_memory_bytes(&self) -> usize;
    fn supports_query(&self, query_type: QueryType) -> bool;
}

pub fn create_searcher(alg: AlgorithmType) -> Box<dyn Searcher> {
    match alg {
        AlgorithmType::Linear => Box::new(LinearSearcher::new()),
        AlgorithmType::Binary => Box::new(BinarySearcher::new()),
        AlgorithmType::Hash => Box::new(HashSearcher::new()),
        AlgorithmType::Avl => Box::new(AvlSearcher::new()),
        AlgorithmType::Bfs => Box::new(BfsSearcher::new()),
        AlgorithmType::Dfs => Box::new(DfsSearcher::new()),
    }
}

pub fn all_searchers() -> Vec<Box<dyn Searcher>> {
    vec![
        Box::new(LinearSearcher::new()),
        Box::new(BinarySearcher::new()),
        Box::new(HashSearcher::new()),
        Box::new(AvlSearcher::new()),
        Box::new(BfsSearcher::new()),
        Box::new(DfsSearcher::new()),
    ]
}

