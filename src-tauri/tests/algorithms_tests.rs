use std::collections::HashSet;

use buscador_archivos::algorithms::all_searchers;
use buscador_archivos::models::{FileEntry, QueryType};
use buscador_archivos::ranking::TopKHeap;

fn create_sample_files() -> Vec<FileEntry> {
    let raw = vec![
        ("main.rs", "/root/src/main.rs", false),
        ("lib.rs", "/root/src/lib.rs", false),
        ("avl.rs", "/root/src/algorithms/avl.rs", false),
        ("bfs.rs", "/root/src/algorithms/bfs.rs", false),
        ("dfs.rs", "/root/src/algorithms/dfs.rs", false),
        ("linear.rs", "/root/src/algorithms/linear.rs", false),
        ("binary.rs", "/root/src/algorithms/binary.rs", false),
        ("hash_index.rs", "/root/src/algorithms/hash_index.rs", false),
        ("config.json", "/root/config.json", false),
        ("Cargo.toml", "/root/Cargo.toml", false),
        ("README.md", "/root/README.md", false),
        ("src", "/root/src", true),
        ("algorithms", "/root/src/algorithms", true),
    ];

    raw.into_iter()
        .enumerate()
        .map(|(i, (name, path, is_dir))| FileEntry {
            id: i,
            name: name.to_string(),
            name_lower: name.to_lowercase(),
            path: path.to_string(),
            size: 512,
            modified: 1700000000 + i as u64,
            is_dir,
        })
        .collect()
}

#[test]
fn test_all_algorithms_exact_match_equivalence() {
    let files = create_sample_files();
    let query = "avl.rs";

    let mut results_sets = Vec::new();

    for mut searcher in all_searchers() {
        searcher.build(&files);
        let results = searcher.search(query, QueryType::Exact, &files, 10);
        assert!(!results.is_empty(), "El algoritmo {:?} debe encontrar '{}'", searcher.algorithm_type(), query);

        let names: HashSet<String> = results.into_iter().map(|r| r.name).collect();
        results_sets.push((searcher.algorithm_type(), names));
    }

    // Verificar que todos los modos devolvieron el mismo conjunto para la consulta exacta
    let first = &results_sets[0].1;
    for (alg, set) in &results_sets[1..] {
        assert_eq!(
            first, set,
            "Discrepancia de resultados entre modo {:?} y {:?}",
            results_sets[0].0, alg
        );
    }
}

#[test]
fn test_all_algorithms_non_existent_query() {
    let files = create_sample_files();
    let query = "archivo_inexistente_12345.xyz";

    for mut searcher in all_searchers() {
        searcher.build(&files);
        let results = searcher.search(query, QueryType::Exact, &files, 10);
        assert!(
            results.is_empty(),
            "Algoritmo {:?} no debería encontrar resultados para {}",
            searcher.algorithm_type(),
            query
        );
    }
}

#[test]
fn test_supported_prefix_equivalence() {
    let files = create_sample_files();
    let query = "b"; // Debe coincidir con bfs.rs y binary.rs

    let mut sets = Vec::new();

    for mut searcher in all_searchers() {
        if !searcher.supports_query(QueryType::Prefix) {
            continue;
        }
        searcher.build(&files);
        let results = searcher.search(query, QueryType::Prefix, &files, 10);
        let names: HashSet<String> = results.into_iter().map(|r| r.name).collect();
        sets.push((searcher.algorithm_type(), names));
    }

    assert!(sets.len() >= 4); // Linear, Binary, AVL, BFS, DFS
    let first = &sets[0].1;
    for (alg, set) in &sets[1..] {
        assert_eq!(
            first, set,
            "Discrepancia en búsqueda de prefijo entre {:?} y {:?}",
            sets[0].0, alg
        );
    }
}

#[test]
fn test_heap_top_k_bound() {
    let files = create_sample_files();
    let mut top = TopKHeap::new(3);

    for f in &files {
        top.push_candidate(f, "rs");
    }

    let results = top.into_ranked_results();
    assert_eq!(results.len(), 3);
    // Verificar que están ordenados de mayor a menor score
    for i in 0..results.len() - 1 {
        assert!(results[i].score >= results[i + 1].score);
    }
}

