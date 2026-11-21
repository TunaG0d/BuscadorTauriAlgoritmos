use std::cmp::Ordering;

use crate::models::{AlgorithmType, FileEntry, QueryType, SearchResult};
use crate::ranking::TopKHeap;
use super::Searcher;

#[derive(Debug, Clone)]
pub struct AvlNode {
    pub key: String,
    pub indices: Vec<usize>,
    pub height: i32,
    pub left: Option<Box<AvlNode>>,
    pub right: Option<Box<AvlNode>>,
}

impl AvlNode {
    pub fn new(key: String, index: usize) -> Self {
        Self {
            key,
            indices: vec![index],
            height: 1,
            left: None,
            right: None,
        }
    }

    pub fn node_height(node: &Option<Box<AvlNode>>) -> i32 {
        node.as_ref().map_or(0, |n| n.height)
    }

    pub fn update_height(&mut self) {
        let hl = Self::node_height(&self.left);
        let hr = Self::node_height(&self.right);
        self.height = 1 + hl.max(hr);
    }

    pub fn balance_factor(&self) -> i32 {
        Self::node_height(&self.left) - Self::node_height(&self.right)
    }

    fn rotate_right(mut y: Box<AvlNode>) -> Box<AvlNode> {
        let mut x = y.left.take().expect("Nodo izquierdo requerido para rotación derecha");
        y.left = x.right.take();
        y.update_height();
        x.right = Some(y);
        x.update_height();
        x
    }

    fn rotate_left(mut x: Box<AvlNode>) -> Box<AvlNode> {
        let mut y = x.right.take().expect("Nodo derecho requerido para rotación izquierda");
        x.right = y.left.take();
        x.update_height();
        y.left = Some(x);
        y.update_height();
        y
    }

    pub fn insert(node_opt: Option<Box<AvlNode>>, key: String, index: usize) -> Box<AvlNode> {
        let mut node = match node_opt {
            None => return Box::new(AvlNode::new(key, index)),
            Some(n) => n,
        };

        match key.cmp(&node.key) {
            Ordering::Equal => {
                node.indices.push(index);
                return node;
            }
            Ordering::Less => {
                node.left = Some(Self::insert(node.left.take(), key.clone(), index));
            }
            Ordering::Greater => {
                node.right = Some(Self::insert(node.right.take(), key.clone(), index));
            }
        }

        node.update_height();
        let balance = node.balance_factor();

        // Caso Izquierda-Izquierda (LL)
        if balance > 1 {
            if let Some(ref left) = node.left {
                if key < left.key {
                    return Self::rotate_right(node);
                }
            }
        }

        // Caso Derecha-Derecha (RR)
        if balance < -1 {
            if let Some(ref right) = node.right {
                if key > right.key {
                    return Self::rotate_left(node);
                }
            }
        }

        // Caso Izquierda-Derecha (LR)
        if balance > 1 {
            if let Some(ref left) = node.left {
                if key > left.key {
                    node.left = Some(Self::rotate_left(node.left.take().unwrap()));
                    return Self::rotate_right(node);
                }
            }
        }

        // Caso Derecha-Izquierda (RL)
        if balance < -1 {
            if let Some(ref right) = node.right {
                if key < right.key {
                    node.right = Some(Self::rotate_right(node.right.take().unwrap()));
                    return Self::rotate_left(node);
                }
            }
        }

        node
    }

    pub fn find_exact<'a>(&'a self, query: &str) -> Option<&'a [usize]> {
        match query.cmp(&self.key) {
            Ordering::Equal => Some(&self.indices),
            Ordering::Less => self.left.as_ref().and_then(|n| n.find_exact(query)),
            Ordering::Greater => self.right.as_ref().and_then(|n| n.find_exact(query)),
        }
    }

    pub fn collect_prefix<'a>(&'a self, prefix: &str, output: &mut Vec<&'a [usize]>) {
        if self.key.starts_with(prefix) {
            output.push(&self.indices);
            if let Some(ref left) = self.left {
                left.collect_prefix(prefix, output);
            }
            if let Some(ref right) = self.right {
                right.collect_prefix(prefix, output);
            }
        } else if prefix < self.key.as_str() {
            if let Some(ref left) = self.left {
                left.collect_prefix(prefix, output);
            }
        } else {
            if let Some(ref right) = self.right {
                right.collect_prefix(prefix, output);
            }
        }
    }

    pub fn memory_size(&self) -> usize {
        let self_size = std::mem::size_of::<AvlNode>()
            + self.key.capacity()
            + self.indices.capacity() * std::mem::size_of::<usize>();

        let left_size = self.left.as_ref().map_or(0, |n| n.memory_size());
        let right_size = self.right.as_ref().map_or(0, |n| n.memory_size());

        self_size + left_size + right_size
    }
}

#[derive(Debug, Clone)]
pub struct AvlSearcher {
    pub root: Option<Box<AvlNode>>,
}

impl AvlSearcher {
    pub fn new() -> Self {
        Self { root: None }
    }
}

impl Searcher for AvlSearcher {
    fn algorithm_type(&self) -> AlgorithmType {
        AlgorithmType::Avl
    }

    fn build(&mut self, files: &[FileEntry]) {
        let mut root = None;
        for (idx, entry) in files.iter().enumerate() {
            root = Some(AvlNode::insert(root, entry.name_lower.clone(), idx));
        }
        self.root = root;
    }

    fn search(&self, query: &str, query_type: QueryType, files: &[FileEntry], top_k: usize) -> Vec<SearchResult> {
        let mut heap = TopKHeap::new(top_k);
        let root = match &self.root {
            Some(r) => r,
            None => return Vec::new(),
        };

        match query_type {
            QueryType::Exact => {
                if let Some(indices) = root.find_exact(query) {
                    for &idx in indices {
                        if let Some(entry) = files.get(idx) {
                            heap.push_candidate(entry, query);
                        }
                    }
                }
            }
            QueryType::Prefix => {
                let mut matches = Vec::new();
                root.collect_prefix(query, &mut matches);
                for group in matches {
                    for &idx in group {
                        if let Some(entry) = files.get(idx) {
                            heap.push_candidate(entry, query);
                        }
                    }
                }
            }
            QueryType::Substring => {
                // El árbol AVL clásico indexado por clave completa no resuelve subcadenas arbitrarias
                // en O(log n). Si se solicita subcadena en modo AVL, se devuelve vacío respetando
                // las restricciones teóricas y documentadas de la estructura.
            }
        }

        heap.into_ranked_results()
    }

    fn estimate_memory_bytes(&self) -> usize {
        std::mem::size_of::<AvlSearcher>() + self.root.as_ref().map_or(0, |n| n.memory_size())
    }

    fn supports_query(&self, query_type: QueryType) -> bool {
        match query_type {
            QueryType::Exact | QueryType::Prefix => true,
            QueryType::Substring => false,
        }
    }
}

