#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

use buscador_archivos::algorithms::create_searcher;
use buscador_archivos::benchmark::run_comparison;
use buscador_archivos::config::save_config;
use buscador_archivos::file_ops::{
    index_directory, open_containing_folder, open_file_default, read_file_preview,
};
use buscador_archivos::models::{
    AlgorithmComparison, AlgorithmType, AppConfig, FilePreview, IndexProgress, QueryType,
    SearchResult,
};
use buscador_archivos::AppState;
use tauri::{Manager, State};

#[tauri::command]
fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    let cfg = state.config.lock().unwrap();
    Ok(cfg.clone())
}

#[tauri::command]
fn set_algorithm(algorithm: String, state: State<AppState>) -> Result<AppConfig, String> {
    let alg_type = match algorithm.to_lowercase().as_str() {
        "linear" => AlgorithmType::Linear,
        "binary" => AlgorithmType::Binary,
        "hash" => AlgorithmType::Hash,
        "avl" => AlgorithmType::Avl,
        "bfs" => AlgorithmType::Bfs,
        "dfs" => AlgorithmType::Dfs,
        _ => return Err("Algoritmo no válido".to_string()),
    };

    let mut cfg = state.config.lock().unwrap();
    cfg.algorithm = alg_type;
    save_config(&cfg)?;

    // Cambiar la implementación activa y reconstruir su índice sobre los archivos en memoria
    let mut searcher = create_searcher(alg_type);
    let files = state.files.lock().unwrap();
    if !files.is_empty() {
        searcher.build(&files);
    }
    *state.searcher.lock().unwrap() = searcher;

    Ok(cfg.clone())
}

#[tauri::command]
fn set_top_k(top_k: usize, state: State<AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.top_k = if top_k == 0 { 20 } else { top_k };
    save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
fn cancel_indexing(state: State<AppState>) -> Result<(), String> {
    state.cancel_flag.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
async fn start_indexing(
    folder_path: String,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    state.cancel_flag.store(false, Ordering::Relaxed);
    let cancel = Arc::clone(&state.cancel_flag);

    // Guardar última carpeta en configuración
    {
        let mut cfg = state.config.lock().unwrap();
        cfg.last_folder = folder_path.clone();
        let _ = save_config(&cfg);
    }

    let folder_clone = folder_path.clone();
    let app_handle_clone = app_handle.clone();

    // Ejecución en hilo separado para no bloquear la interfaz gráfica
    let entries = thread::spawn(move || {
        index_directory(&folder_clone, cancel, |progress: IndexProgress| {
            let _ = app_handle_clone.emit_all("index-progress", progress);
        })
    })
    .join()
    .map_err(|_| "Error en el hilo de indexación".to_string())??;

    let count = entries.len();

    // Actualizar archivos en el estado y reconstruir el índice del algoritmo activo
    {
        let mut files_lock = state.files.lock().unwrap();
        *files_lock = entries;

        let mut searcher_lock = state.searcher.lock().unwrap();
        searcher_lock.build(&files_lock);
    }

    Ok(count)
}

#[tauri::command]
fn search_files(
    query: String,
    query_type: String,
    state: State<AppState>,
) -> Result<Vec<SearchResult>, String> {
    let q_type = match query_type.to_lowercase().as_str() {
        "exact" => QueryType::Exact,
        "prefix" => QueryType::Prefix,
        "substring" => QueryType::Substring,
        _ => QueryType::Substring,
    };

    let files = state.files.lock().unwrap();
    let top_k = state.config.lock().unwrap().top_k;
    let searcher = state.searcher.lock().unwrap();

    let results = searcher.search(&query.to_lowercase(), q_type, &files, top_k);
    Ok(results)
}

#[tauri::command]
fn compare_algorithms(
    query: String,
    query_type: String,
    state: State<AppState>,
) -> Result<Vec<AlgorithmComparison>, String> {
    let q_type = match query_type.to_lowercase().as_str() {
        "exact" => QueryType::Exact,
        "prefix" => QueryType::Prefix,
        "substring" => QueryType::Substring,
        _ => QueryType::Exact,
    };

    let files = state.files.lock().unwrap();
    if files.is_empty() {
        return Err("No hay archivos indexados para realizar la comparación.".to_string());
    }

    let top_k = state.config.lock().unwrap().top_k;
    let comparisons = run_comparison(&query, q_type, &files, top_k);
    Ok(comparisons)
}

#[tauri::command]
fn read_preview(file_path: String) -> Result<FilePreview, String> {
    // Límite de 512 KB para evitar congelamientos de la interfaz
    read_file_preview(&file_path, 512 * 1024)
}

#[tauri::command]
fn open_file(file_path: String) -> Result<(), String> {
    open_file_default(&file_path)
}

#[tauri::command]
fn open_folder(file_path: String) -> Result<(), String> {
    open_containing_folder(&file_path)
}

#[tauri::command]
fn get_indexed_count(state: State<AppState>) -> Result<usize, String> {
    let files = state.files.lock().unwrap();
    Ok(files.len())
}

fn main() {
    let state = AppState::new();

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_config,
            set_algorithm,
            set_top_k,
            start_indexing,
            cancel_indexing,
            search_files,
            compare_algorithms,
            read_preview,
            open_file,
            open_folder,
            get_indexed_count
        ])
        .run(tauri::generate_context!())
        .expect("Error al inicializar la aplicación Tauri");
}

