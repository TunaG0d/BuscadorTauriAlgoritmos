use std::time::Instant;

use crate::algorithms::all_searchers;
use crate::models::{AlgorithmComparison, AlgorithmType, FileEntry, QueryType};

pub fn run_comparison(
    query: &str,
    query_type: QueryType,
    files: &[FileEntry],
    top_k: usize,
) -> Vec<AlgorithmComparison> {
    let mut comparisons = Vec::new();
    let q_lower = query.to_lowercase();
    let searchers = all_searchers();

    for mut searcher in searchers {
        let alg_type = searcher.algorithm_type();
        let name = alg_type.name().to_string();
        let structure = match alg_type {
            AlgorithmType::Linear => "Vec sin ordenar",
            AlgorithmType::Binary => "Vec ordenado por nombre",
            AlgorithmType::Hash => "HashMap (std)",
            AlgorithmType::Avl => "Árbol AVL balanceado a mano",
            AlgorithmType::Bfs => "Árbol de directorios con cola",
            AlgorithmType::Dfs => "Árbol de directorios con pila",
        }
        .to_string();

        let is_supported = searcher.supports_query(query_type);

        // Medir tiempo de construcción del índice
        let t_build_start = Instant::now();
        searcher.build(files);
        let build_time_ms = t_build_start.elapsed().as_secs_f64() * 1000.0;

        // Estimar memoria ocupada en MB
        let memory_mb = searcher.estimate_memory_bytes() as f64 / (1024.0 * 1024.0);

        if !is_supported {
            let note = match (alg_type, query_type) {
                (AlgorithmType::Hash, QueryType::Substring) => {
                    "No soportado: tabla hash O(1) no resuelve subcadenas arbitrarias sin degradar a O(n)".to_string()
                }
                (AlgorithmType::Hash, QueryType::Prefix) => {
                    "No soportado: tabla hash no indexa orden de prefijo sin escaneo completo".to_string()
                }
                (AlgorithmType::Binary, QueryType::Substring) => {
                    "No soportado: búsqueda binaria O(log n) requiere prefijo o coincidencia exacta".to_string()
                }
                (AlgorithmType::Avl, QueryType::Substring) => {
                    "No soportado: árbol AVL indexado por nombre no resuelve subcadenas en O(log n)".to_string()
                }
                _ => "Consulta no soportada para esta estructura".to_string(),
            };

            comparisons.push(AlgorithmComparison {
                algorithm_id: format!("{:?}", alg_type).to_lowercase(),
                name,
                structure,
                build_time_ms,
                search_time_ms: 0.0,
                memory_mb,
                results_count: 0,
                supported: false,
                note,
            });
            continue;
        }

        // Ejecución de calentamiento (descartada para evitar efectos de caché fría)
        let _ = searcher.search(&q_lower, query_type, files, top_k);

        // 30 repeticiones para calcular la mediana
        let mut timings = Vec::with_capacity(30);
        let mut results_count = 0;

        for i in 0..30 {
            let t_query_start = Instant::now();
            let res = searcher.search(&q_lower, query_type, files, top_k);
            let elapsed_ms = t_query_start.elapsed().as_secs_f64() * 1000.0;
            timings.push(elapsed_ms);
            if i == 0 {
                results_count = res.len();
            }
        }

        timings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let search_time_ms = timings[15]; // Mediana exacta (posición 15 de 30)

        comparisons.push(AlgorithmComparison {
            algorithm_id: format!("{:?}", alg_type).to_lowercase(),
            name,
            structure,
            build_time_ms,
            search_time_ms,
            memory_mb,
            results_count,
            supported: true,
            note: format!("Mediana de 30 ejecuciones (descartada ejecución fría)"),
        });
    }

    comparisons
}
