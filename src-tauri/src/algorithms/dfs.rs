use crate::models::{AlgorithmType, FileEntry, QueryType, SearchResult};
use crate::ranking::TopKHeap;
use crate::tree::DirTree;
use super::Searcher;

#[derive(Debug, Clone, Default)]
pub struct DfsSearcher {
    tree: DirTree,
}

impl DfsSearcher {
    pub fn new() -> Self {
        Self {
            tree: DirTree::new(),
        }
    }
}

impl Searcher for DfsSearcher {
    fn algorithm_type(&self) -> AlgorithmType {
        AlgorithmType::Dfs
    }

    fn build(&mut self, files: &[FileEntry]) {
        self.tree = DirTree::build(files);
    }

    fn search(&self, query: &str, query_type: QueryType, files: &[FileEntry], top_k: usize) -> Vec<SearchResult> {
        let mut heap = TopKHeap::new(top_k);
        // Recorrido en profundidad (DFS) utilizando una pila O(V + E)
        self.tree.dfs_search(query, query_type, files, &mut heap);
        heap.into_ranked_results()
    }

    fn estimate_memory_bytes(&self) -> usize {
        std::mem::size_of::<DfsSearcher>() + self.tree.estimate_memory_bytes()
    }

    fn supports_query(&self, _query_type: QueryType) -> bool {
        true
    }
}

