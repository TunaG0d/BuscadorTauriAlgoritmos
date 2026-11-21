use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use walkdir::WalkDir;

use crate::models::{FileEntry, FilePreview, IndexProgress};

pub fn index_directory<F>(
    root_path: &str,
    cancel_flag: Arc<AtomicBool>,
    mut progress_callback: F,
) -> Result<Vec<FileEntry>, String>
where
    F: FnMut(IndexProgress),
{
    let path = Path::new(root_path);
    if !path.exists() {
        return Err(format!("La ruta especificada no existe: {}", root_path));
    }

    let mut entries = Vec::new();
    let mut count = 0;

    let walker = WalkDir::new(path)
        .follow_links(false)
        .into_iter();

    for result in walker {
        if cancel_flag.load(Ordering::Relaxed) {
            progress_callback(IndexProgress {
                count,
                current_path: String::new(),
                finished: false,
                cancelled: true,
            });
            return Err("Indexación cancelada por el usuario.".to_string());
        }

        // Manejo de errores de acceso (permisos denegados, enlaces rotos, etc.) sin caerse
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let file_name = entry.file_name().to_string_lossy().to_string();
        let name_lower = file_name.to_lowercase();
        let full_path = entry.path().to_string_lossy().to_string();

        let metadata = entry.metadata().ok();
        let (size, modified, is_dir) = if let Some(meta) = metadata {
            let s = if meta.is_file() { meta.len() } else { 0 };
            let m = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            (s, m, meta.is_dir())
        } else {
            (0, 0, entry.file_type().is_dir())
        };

        entries.push(FileEntry {
            id: count,
            name: file_name,
            name_lower,
            path: full_path.clone(),
            size,
            modified,
            is_dir,
        });

        count += 1;

        if count % 100 == 0 {
            progress_callback(IndexProgress {
                count,
                current_path: full_path,
                finished: false,
                cancelled: false,
            });
        }
    }

    progress_callback(IndexProgress {
        count,
        current_path: String::new(),
        finished: true,
        cancelled: false,
    });

    Ok(entries)
}

pub fn read_file_preview(file_path: &str, max_bytes: usize) -> Result<FilePreview, String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err("El archivo no existe.".to_string());
    }
    if path.is_dir() {
        return Err("La ruta seleccionada es un directorio.".to_string());
    }

    let metadata = path.metadata().map_err(|e| e.to_string())?;
    let total_size = metadata.len();

    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let read_limit = total_size.min(max_bytes as u64) as usize;
    let mut buffer = vec![0u8; read_limit];
    let bytes_read = file.read(&mut buffer).map_err(|e| e.to_string())?;
    buffer.truncate(bytes_read);

    // Detección de archivos binarios: búsqueda de bytes nulos en los primeros 1024 bytes
    let sample = &buffer[..buffer.len().min(1024)];
    let is_binary = sample.contains(&0);

    let content = if is_binary {
        "[Archivo binario - vista previa no disponible]".to_string()
    } else {
        String::from_utf8_lossy(&buffer).to_string()
    };

    Ok(FilePreview {
        path: file_path.to_string(),
        content,
        size: total_size,
        truncated: total_size > (bytes_read as u64),
        is_binary,
    })
}

pub fn open_file_default(file_path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/c", "start", "", file_path])
            .spawn()
            .map_err(|e| format!("Error al abrir archivo: {}", e))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("xdg-open")
            .arg(file_path)
            .spawn()
            .map_err(|e| format!("Error al abrir archivo: {}", e))?;
    }
    Ok(())
}

pub fn open_containing_folder(file_path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // En Windows, explorer.exe con /select,ruta abre la carpeta y resalta el archivo
        let status = Command::new("explorer")
            .arg(format!("/select,{}", file_path))
            .status();
        match status {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error al abrir explorador: {}", e)),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(parent) = Path::new(file_path).parent() {
            Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("Error al abrir carpeta: {}", e))?;
            Ok(())
        } else {
            Err("No se pudo determinar el directorio padre.".to_string())
        }
    }
}

