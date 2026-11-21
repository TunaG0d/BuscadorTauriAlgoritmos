# Buscador de Archivos con Tauri y Comparación de Algoritmos

Aplicación de escritorio desarrollada en Rust con Tauri y frontend en HTML/CSS/JavaScript para la búsqueda e indexación de archivos locales, comparando empíricamente estructuras de datos fundamentales de la asignatura.

## Estructuras de Datos y Algoritmos Implementados

| Algoritmo | Estructura de Datos | Complejidad Búsqueda | Complejidad Construcción | Soporte de Consultas |
| :--- | :--- | :--- | :--- | :--- |
| **Búsqueda lineal** | `Vec` sin ordenar | O(n) | O(n) | Exacta, Prefijo, Subcadena |
| **Búsqueda binaria** | `Vec` ordenado por nombre | O(\log n) | O(n \log n) | Exacta, Prefijo |
| **Índice hash** | `std::collections::HashMap` | O(1) promedio | O(n) | Solo exacta |
| **Árbol AVL** | Árbol balanceado propio con rotaciones | O(\log n) | O(n \log n) | Exacta, Prefijo |
| **Recorrido BFS** | Árbol de directorios propio con cola (`VecDeque`) | O(V + E) | Sin índice previo | Exacta, Prefijo, Subcadena |
| **Recorrido DFS** | Árbol de directorios propio con pila (`Vec`) | O(V + E) | Sin índice previo | Exacta, Prefijo, Subcadena |

### Ranking de Resultados (Top-K con Montículo)
Todos los modos retornan los K resultados más relevantes mediante un min-heap de capacidad K (`std::collections::BinaryHeap` con `std::cmp::Reverse`).
- Cada elemento candidato se compara contra la raíz del heap en O(1).
- Si supera el puntaje mínimo de los mejores K, se reemplaza en O(\log K).
- El costo total sobre n elementos es O(n \log K), significativamente más eficiente que ordenar todo el conjunto en O(n \log n).

## Requisitos Previos

- **Rust** (versión 1.75 o superior, toolchain con cargo).
- WebView2 Runtime (instalado por defecto en Windows 10 y 11).

## Compilación y Ejecución

Para iniciar la aplicación directamente:
```bash
cargo run
```

O si se tiene instalada la CLI de Tauri:
```bash
cargo tauri dev
```

Para generar la versión de producción optimizada:
```bash
cargo build --release
```

## Pruebas Automáticas

El proyecto incluye pruebas unitarias para el balanceo y rotaciones del árbol AVL, recorridos BFS y DFS, ranking de montículo y verificación de equivalencia entre todos los algoritmos:
```bash
cargo test
```

## Generador de Datos de Prueba

Para generar árboles sintéticos de directorios (por ejemplo, para escenarios de 1.000, 10.000 y 100.000 archivos):
```bash
cargo run --bin generate_dataset -- --count 10000 --output ./datos_10k
```

Parámetros:
- `--count` o `-c`: Cantidad total de archivos a generar (por defecto: 1000).
- `--output` o `-o`: Directorio de destino (por defecto: `./test_data_1k`).

## Estructura del Proyecto

```
.
├── Cargo.toml                  # Workspace raíz de Cargo
├── README.md                   # Documentación y guía de ejecución
├── src-tauri/
│   ├── Cargo.toml              # Manifiesto y dependencias de Rust
│   ├── tauri.conf.json         # Configuración de ventana y permisos Tauri
│   ├── build.rs                # Script de compilación de Tauri
│   ├── src/
│   │   ├── main.rs             # Comandos Tauri y punto de entrada
│   │   ├── lib.rs              # Estado y exportación de módulos
│   │   ├── models.rs           # Definiciones de datos y estructuras
│   │   ├── tree.rs             # Árbol de directorios propio para BFS y DFS
│   │   ├── ranking.rs          # Algoritmo Top-K con BinaryHeap
│   │   ├── file_ops.rs         # Indexación, lectura previa y apertura de archivos
│   │   ├── config.rs           # Persistencia de configuración en disco
│   │   ├── benchmark.rs        # Mediciones de tiempo (mediana 30x) y memoria
│   │   ├── algorithms/         # Implementaciones de los 6 algoritmos
│   │   │   ├── mod.rs          # Trait común Searcher y factoría
│   │   │   ├── linear.rs       # Búsqueda lineal
│   │   │   ├── binary.rs       # Búsqueda binaria
│   │   │   ├── hash_index.rs   # Índice hash
│   │   │   ├── avl.rs          # Árbol AVL balanceado a mano
│   │   │   ├── bfs.rs          # Recorrido en anchura
│   │   │   └── dfs.rs          # Recorrido en profundidad
│   │   └── bin/
│   │       └── generate_dataset.rs # Generador de datos sintéticos
│   └── tests/
│       ├── avl_tests.rs        # Pruebas de rotaciones y balance AVL
│       └── algorithms_tests.rs # Pruebas de equivalencia y correctitud
└── ui/
    ├── index.html              # Interfaz de usuario
    ├── style.css               # Estilos y diseño visual
    └── app.js                  # Lógica del cliente y eventos IPC
```

