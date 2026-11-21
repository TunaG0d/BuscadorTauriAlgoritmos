use std::collections::HashMap;

use crate::models::{AlgorithmType, FileEntry, QueryType, SearchResult};
use crate::ranking::TopKHeap;
use super::Searcher;

#[derive(Debug, Clone, Default)]
pub struct HashSearcher {
    // Mapeo directo: nombre completo en minúsculas -> lista de índices
    map: HashMap<String, Vec<usize>>,
}

impl HashSearcher {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl Searcher for HashSearcher {
    fn algorithm_type(&self) -> AlgorithmType {
        AlgorithmType::Hash
    }

    fn build(&mut self, files: &[FileEntry]) {
        let mut map: HashMap<String, Vec<usize>> = HashMap::with_capacity(files.len());
        for (idx, entry) in files.iter().enumerate() {
            map.entry(entry.name_lower.clone()).or_default().push(idx);
        }
        self.map = map;
    }

    fn search(&self, query: &str, query_type: QueryType, files: &[FileEntry], top_k: usize) -> Vec<SearchResult> {
        let mut heap = TopKHeap::new(top_k);

        match query_type {
            QueryType::Exact => {
                // Búsqueda O(1) promedio
                if let Some(indices) = self.map.get(query) {
                    for &idx in indices {
                        if let Some(entry) = files.get(idx) {
                            heap.push_candidate(entry, query);
                        }
                    }
                }
            }
            QueryType::Prefix | QueryType::Substring => {
                // La tabla hash no soporta búsquedas por prefijo ni subcadena sin
                // degradar a un escaneo lineal completo sobre todas sus claves O(n).
                // Conforme a la especificación académica, esto no está soportado.
            }
        }

        heap.into_ranked_results()
    }

    fn estimate_memory_bytes(&self) -> usize {
        let mut bytes = std::mem::size_of::<HashSearcher>()
            + self.map.capacity() * (std::mem::size_of::<String>() + std::mem::size_of::<Vec<usize>>());

        for (k, v) in &self.map {
            bytes += k.capacity() + v.capacity() * std::mem::size_of::<usize>();
        }
        bytes
    }

    fn supports_query(&self, query_type: QueryType) -> bool {
        match query_type {
            QueryType::Exact => true,
            QueryType::Prefix | QueryType::Substring => false,
        }
    }
}

