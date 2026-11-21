use crate::models::{AlgorithmType, FileEntry, QueryType, SearchResult};
use crate::ranking::TopKHeap;
use crate::tree::DirTree;
use super::Searcher;

#[derive(Debug, Clone, Default)]
pub struct BfsSearcher {
    tree: DirTree,
}

impl BfsSearcher {
    pub fn new() -> Self {
        Self {
            tree: DirTree::new(),
        }
    }
}

impl Searcher for BfsSearcher {
    fn algorithm_type(&self) -> AlgorithmType {
        AlgorithmType::Bfs
    }

    fn build(&mut self, files: &[FileEntry]) {
        // En el enfoque BFS/DFS de la tarea, no hay índice precalculado de búsqueda;
        // se representa la estructura jerárquica en memoria (árbol/grafo de directorios).
        self.tree = DirTree::build(files);
    }

    fn search(&self, query: &str, query_type: QueryType, files: &[FileEntry], top_k: usize) -> Vec<SearchResult> {
        let mut heap = TopKHeap::new(top_k);
        // Recorrido por niveles (BFS) utilizando una cola O(V + E)
        self.tree.bfs_search(query, query_type, files, &mut heap);
        heap.into_ranked_results()
    }

    fn estimate_memory_bytes(&self) -> usize {
        std::mem::size_of::<BfsSearcher>() + self.tree.estimate_memory_bytes()
    }

    fn supports_query(&self, _query_type: QueryType) -> bool {
        true
    }
}

