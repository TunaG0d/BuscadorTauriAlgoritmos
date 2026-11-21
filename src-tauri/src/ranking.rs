use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

use crate::models::{FileEntry, SearchResult};

#[derive(Clone, Eq, PartialEq)]
pub struct RankedItem {
    pub score: u32,
    pub modified: u64,
    pub result: SearchResult,
}

impl Ord for RankedItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .cmp(&other.score)
            .then_with(|| self.modified.cmp(&other.modified))
            .then_with(|| other.result.path.len().cmp(&self.result.path.len()))
            .then_with(|| other.result.id.cmp(&self.result.id))
    }
}

impl PartialOrd for RankedItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn calculate_score(entry: &FileEntry, query: &str) -> u32 {
    let name = &entry.name_lower;
    if name == query {
        1000
    } else if name.starts_with(query) {
        500
    } else if name.contains(query) {
        200
    } else {
        50
    }
}

pub struct TopKHeap {
    k: usize,
    heap: BinaryHeap<Reverse<RankedItem>>,
}

impl TopKHeap {
    pub fn new(k: usize) -> Self {
        let capacity = if k == 0 { 20 } else { k };
        Self {
            k: capacity,
            heap: BinaryHeap::with_capacity(capacity),
        }
    }

    pub fn push_candidate(&mut self, entry: &FileEntry, query: &str) {
        let score = calculate_score(entry, query);
        let result = SearchResult {
            id: entry.id,
            name: entry.name.clone(),
            path: entry.path.clone(),
            size: entry.size,
            modified: entry.modified,
            score,
            is_dir: entry.is_dir,
        };
        let item = RankedItem {
            score,
            modified: entry.modified,
            result,
        };

        if self.heap.len() < self.k {
            self.heap.push(Reverse(item));
        } else if let Some(min_top) = self.heap.peek() {
            if item > min_top.0 {
                self.heap.pop();
                self.heap.push(Reverse(item));
            }
        }
    }

    pub fn into_ranked_results(self) -> Vec<SearchResult> {
        self.heap
            .into_sorted_vec()
            .into_iter()
            .map(|rev| rev.0.result)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_k_ranking() {
        let mut top = TopKHeap::new(2);
        let e1 = FileEntry {
            id: 1,
            name: "test.txt".to_string(),
            name_lower: "test.txt".to_string(),
            path: "/a/test.txt".to_string(),
            size: 10,
            modified: 100,
            is_dir: false,
        };
        let e2 = FileEntry {
            id: 2,
            name: "my_test_file.rs".to_string(),
            name_lower: "my_test_file.rs".to_string(),
            path: "/a/my_test_file.rs".to_string(),
            size: 20,
            modified: 200,
            is_dir: false,
        };
        let e3 = FileEntry {
            id: 3,
            name: "test".to_string(),
            name_lower: "test".to_string(),
            path: "/a/test".to_string(),
            size: 30,
            modified: 300,
            is_dir: false,
        };

        top.push_candidate(&e1, "test");
        top.push_candidate(&e2, "test");
        top.push_candidate(&e3, "test");

        let results = top.into_ranked_results();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "test"); // Exact match: score 1000
        assert_eq!(results[1].name, "test.txt"); // Prefix match: score 500
    }
}

