use std::collections::{HashMap, VecDeque};

use crate::models::{FileEntry, QueryType};
use crate::ranking::TopKHeap;

#[derive(Debug, Clone)]
pub struct DirTreeNode {
    pub file_index: usize,
    pub name: String,
    pub name_lower: String,
    pub is_dir: bool,
    pub children: Vec<DirTreeNode>,
}

#[derive(Debug, Clone, Default)]
pub struct DirTree {
    pub roots: Vec<DirTreeNode>,
    pub total_nodes: usize,
}

impl DirTree {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            total_nodes: 0,
        }
    }

    pub fn build(files: &[FileEntry]) -> Self {
        if files.is_empty() {
            return Self::new();
        }

        // Usamos una tabla de rutas normalizadas para construir la jerarquía
        // sin dependencias de crates externas.
        let mut path_to_idx: HashMap<&str, usize> = HashMap::with_capacity(files.len());
        for (i, f) in files.iter().enumerate() {
            path_to_idx.insert(f.path.as_str(), i);
        }

        // Construir lista de hijos para cada entrada
        let mut children_map: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut root_indices: Vec<usize> = Vec::new();

        for (i, f) in files.iter().enumerate() {
            let p = std::path::Path::new(&f.path);
            if let Some(parent) = p.parent() {
                let parent_str = parent.to_str().unwrap_or("");
                if let Some(&p_idx) = path_to_idx.get(parent_str) {
                    if p_idx != i {
                        children_map.entry(p_idx).or_default().push(i);
                        continue;
                    }
                }
            }
            root_indices.push(i);
        }

        fn build_node(idx: usize, files: &[FileEntry], children_map: &HashMap<usize, Vec<usize>>) -> DirTreeNode {
            let f = &files[idx];
            let children = children_map
                .get(&idx)
                .map(|list| {
                    list.iter()
                        .map(|&c_idx| build_node(c_idx, files, children_map))
                        .collect()
                })
                .unwrap_or_default();

            DirTreeNode {
                file_index: idx,
                name: f.name.clone(),
                name_lower: f.name_lower.clone(),
                is_dir: f.is_dir,
                children,
            }
        }

        let roots: Vec<DirTreeNode> = root_indices
            .into_iter()
            .map(|r_idx| build_node(r_idx, files, &children_map))
            .collect();

        Self {
            roots,
            total_nodes: files.len(),
        }
    }

    fn matches(node_name: &str, query: &str, query_type: QueryType) -> bool {
        match query_type {
            QueryType::Exact => node_name == query,
            QueryType::Prefix => node_name.starts_with(query),
            QueryType::Substring => node_name.contains(query),
        }
    }

    pub fn bfs_search(&self, query: &str, query_type: QueryType, files: &[FileEntry], top_k: &mut TopKHeap) {
        let mut queue: VecDeque<&DirTreeNode> = VecDeque::new();
        for root in &self.roots {
            queue.push_back(root);
        }

        while let Some(current) = queue.pop_front() {
            if Self::matches(&current.name_lower, query, query_type) {
                if let Some(entry) = files.get(current.file_index) {
                    top_k.push_candidate(entry, query);
                }
            }

            for child in &current.children {
                queue.push_back(child);
            }
        }
    }

    pub fn dfs_search(&self, query: &str, query_type: QueryType, files: &[FileEntry], top_k: &mut TopKHeap) {
        let mut stack: Vec<&DirTreeNode> = Vec::new();
        // Invertimos el orden al apilar las raíces para visitar de izquierda a derecha
        for root in self.roots.iter().rev() {
            stack.push(root);
        }

        while let Some(current) = stack.pop() {
            if Self::matches(&current.name_lower, query, query_type) {
                if let Some(entry) = files.get(current.file_index) {
                    top_k.push_candidate(entry, query);
                }
            }

            for child in current.children.iter().rev() {
                stack.push(child);
            }
        }
    }

    pub fn estimate_memory_bytes(&self) -> usize {
        fn node_size(n: &DirTreeNode) -> usize {
            std::mem::size_of::<DirTreeNode>()
                + n.name.capacity()
                + n.name_lower.capacity()
                + n.children.capacity() * std::mem::size_of::<DirTreeNode>()
                + n.children.iter().map(node_size).sum::<usize>()
        }
        std::mem::size_of::<DirTree>()
            + self.roots.capacity() * std::mem::size_of::<DirTreeNode>()
            + self.roots.iter().map(node_size).sum::<usize>()
    }
}
