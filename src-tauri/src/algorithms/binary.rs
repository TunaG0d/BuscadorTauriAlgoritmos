use crate::models::{AlgorithmType, FileEntry, QueryType, SearchResult};
use crate::ranking::TopKHeap;
use super::Searcher;

#[derive(Debug, Clone, Default)]
pub struct BinarySearcher {
    // Lista ordenada alfabéticamente por (nombre_en_minuscula, indice_original)
    sorted_items: Vec<(String, usize)>,
}

impl BinarySearcher {
    pub fn new() -> Self {
        Self {
            sorted_items: Vec::new(),
        }
    }
}

impl Searcher for BinarySearcher {
    fn algorithm_type(&self) -> AlgorithmType {
        AlgorithmType::Binary
    }

    fn build(&mut self, files: &[FileEntry]) {
        let mut items: Vec<(String, usize)> = files
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name_lower.clone(), i))
            .collect();

        // O(n log n)
        items.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        self.sorted_items = items;
    }

    fn search(&self, query: &str, query_type: QueryType, files: &[FileEntry], top_k: usize) -> Vec<SearchResult> {
        let mut heap = TopKHeap::new(top_k);

        match query_type {
            QueryType::Exact => {
                // Búsqueda binaria O(log n)
                if let Ok(pos) = self.sorted_items.binary_search_by(|(k, _)| k.as_str().cmp(query)) {
                    // Recorrer hacia la izquierda posibles duplicados
                    let mut i = pos;
                    while i > 0 && self.sorted_items[i - 1].0 == query {
                        i -= 1;
                    }
                    // Insertar todas las coincidencias idénticas
                    while i < self.sorted_items.len() && self.sorted_items[i].0 == query {
                        let idx = self.sorted_items[i].1;
                        if let Some(entry) = files.get(idx) {
                            heap.push_candidate(entry, query);
                        }
                        i += 1;
                    }
                }
            }
            QueryType::Prefix => {
                // Localizar el punto inicial mediante partition_point O(log n)
                let start = self.sorted_items.partition_point(|(k, _)| k.as_str() < query);
                let mut i = start;
                while i < self.sorted_items.len() && self.sorted_items[i].0.starts_with(query) {
                    let idx = self.sorted_items[i].1;
                    if let Some(entry) = files.get(idx) {
                        heap.push_candidate(entry, query);
                    }
                    i += 1;
                }
            }
            QueryType::Substring => {
                // Búsqueda binaria no resuelve subcadenas arbitrarias sin índice adicional.
            }
        }

        heap.into_ranked_results()
    }

    fn estimate_memory_bytes(&self) -> usize {
        let mut bytes = std::mem::size_of::<BinarySearcher>()
            + self.sorted_items.capacity() * std::mem::size_of::<(String, usize)>();
        for (k, _) in &self.sorted_items {
            bytes += k.capacity();
        }
        bytes
    }

    fn supports_query(&self, query_type: QueryType) -> bool {
        match query_type {
            QueryType::Exact | QueryType::Prefix => true,
            QueryType::Substring => false,
        }
    }
}

