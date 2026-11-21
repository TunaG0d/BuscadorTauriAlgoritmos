use std::env;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

fn print_usage() {
    println!("Uso: generate_dataset [--count <N>] [--output <DIR>]");
    println!("Ejemplo: cargo run --bin generate_dataset -- --count 10000 --output ./datos_10k");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut count: usize = 1000;
    let mut output_dir = PathBuf::from("./test_data_1k");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--count" | "-c" => {
                if i + 1 < args.len() {
                    count = args[i + 1].parse().unwrap_or(1000);
                    i += 1;
                }
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--help" | "-h" => {
                print_usage();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    println!("Generando árbol sintético con {} archivos en '{}'...", count, output_dir.display());

    let extensions = ["txt", "rs", "md", "csv", "log"];
    let dirs_per_level = 10;
    let files_per_leaf = 20;

    // Crear jerarquía de carpetas
    let mut current_file_index = 0;
    let mut leaf_index = 0;

    while current_file_index < count {
        let l1 = leaf_index % dirs_per_level;
        let l2 = (leaf_index / dirs_per_level) % dirs_per_level;
        let l3 = leaf_index / (dirs_per_level * dirs_per_level);

        let folder = output_dir
            .join(format!("dir_{:02}", l3))
            .join(format!("sub_{:02}", l2))
            .join(format!("leaf_{:02}", l1));

        create_dir_all(&folder).expect("Error creando directorios");

        for _ in 0..files_per_leaf {
            if current_file_index >= count {
                break;
            }
            let ext = extensions[current_file_index % extensions.len()];
            let filename = format!("documento_{:06}.{}", current_file_index + 1, ext);
            let file_path = folder.join(&filename);

            let mut file = BufWriter::new(File::create(&file_path).expect("Error creando archivo"));
            writeln!(file, "# Archivo de prueba: {}", filename).unwrap();
            writeln!(file, "Indice sintético: {}", current_file_index + 1).unwrap();
            writeln!(file, "Ruta relativa: {}", file_path.display()).unwrap();
            writeln!(file, "Contenido de texto generado para pruebas de indexación y previsualización.").unwrap();

            current_file_index += 1;
        }

        leaf_index += 1;
    }

    println!("Generación completada: {} archivos creados exitosamente.", current_file_index);
}

