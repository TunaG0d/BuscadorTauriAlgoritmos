use crate::models::{AlgorithmType, FileEntry, QueryType, SearchResult};
use crate::ranking::TopKHeap;
use super::Searcher;

#[derive(Debug, Clone, Default)]
pub struct LinearSearcher {
    indices: Vec<usize>,
}

impl LinearSearcher {
    pub fn new() -> Self {
        Self { indices: Vec::new() }
    }
}

impl Searcher for LinearSearcher {
    fn algorithm_type(&self) -> AlgorithmType {
        AlgorithmType::Linear
    }

    fn build(&mut self, files: &[FileEntry]) {
        self.indices = (0..files.len()).collect();
    }

    fn search(&self, query: &str, query_type: QueryType, files: &[FileEntry], top_k: usize) -> Vec<SearchResult> {
        let mut heap = TopKHeap::new(top_k);

        for &idx in &self.indices {
            if let Some(entry) = files.get(idx) {
                let is_match = match query_type {
                    QueryType::Exact => entry.name_lower == query,
                    QueryType::Prefix => entry.name_lower.starts_with(query),
                    QueryType::Substring => entry.name_lower.contains(query),
                };

                if is_match {
                    heap.push_candidate(entry, query);
                }
            }
        }

        heap.into_ranked_results()
    }

    fn estimate_memory_bytes(&self) -> usize {
        std::mem::size_of::<LinearSearcher>() + self.indices.capacity() * std::mem::size_of::<usize>()
    }

    fn supports_query(&self, _query_type: QueryType) -> bool {
        true
    }
}

