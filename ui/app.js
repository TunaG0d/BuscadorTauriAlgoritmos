// Inicialización del cliente Tauri
const tauri = window.__TAURI__ || null;
const invoke = tauri ? tauri.invoke : async (cmd, args) => { console.warn("Tauri invoke mock:", cmd, args); return null; };
const listen = tauri && tauri.event ? tauri.event.listen : () => { };

// Elementos del DOM
const tabButtons = document.querySelectorAll(".nav-btn");
const tabPanes = document.querySelectorAll(".tab-pane");

const folderPathInput = document.getElementById("folder-path-input");
const btnBrowse = document.getElementById("btn-browse");
const btnIndex = document.getElementById("btn-index");
const btnCancelIndex = document.getElementById("btn-cancel-index");

const indexingProgressContainer = document.getElementById("indexing-progress-container");
const indexingStatusText = document.getElementById("indexing-status-text");
const indexingCountText = document.getElementById("indexing-count-text");
const indexingCurrentFile = document.getElementById("indexing-current-file");

const searchInput = document.getElementById("search-input");
const queryTypeSelect = document.getElementById("query-type-select");
const btnSearch = document.getElementById("btn-search");
const searchSpinner = document.getElementById("search-spinner");
const algoCompatibilityAlert = document.getElementById("algo-compatibility-alert");

const resultsTitle = document.getElementById("results-title");
const resultsMeta = document.getElementById("results-meta");
const resultsTbody = document.getElementById("results-tbody");

const sidebarActiveAlgo = document.getElementById("sidebar-active-algo");
const sidebarIndexedCount = document.getElementById("sidebar-indexed-count");

const compareQueryInput = document.getElementById("compare-query-input");
const compareTypeSelect = document.getElementById("compare-type-select");
const btnRunCompare = document.getElementById("btn-run-compare");
const compareSpinner = document.getElementById("compare-spinner");
const compareTbody = document.getElementById("compare-tbody");

const configAlgoSelect = document.getElementById("config-algo-select");
const configTopK = document.getElementById("config-top-k");
const btnSaveConfig = document.getElementById("btn-save-config");
const configSavedHint = document.getElementById("config-saved-hint");

const previewModal = document.getElementById("preview-modal");
const previewFileName = document.getElementById("preview-file-name");
const previewMeta = document.getElementById("preview-meta");
const previewBody = document.getElementById("preview-body");
const previewSpinner = document.getElementById("preview-spinner");
const btnClosePreview = document.getElementById("btn-close-preview");
const btnModalOpenFile = document.getElementById("btn-modal-open-file");
const btnModalOpenFolder = document.getElementById("btn-modal-open-folder");

let currentPreviewPath = "";
let currentActiveAlgorithm = "linear";

// Nombres legibles para algoritmos
const ALGO_NAMES = {
  linear: "Búsqueda lineal",
  binary: "Búsqueda binaria",
  hash: "Índice hash",
  avl: "Árbol AVL",
  bfs: "Recorrido BFS",
  dfs: "Recorrido DFS"
};

// Formato de bytes
function formatBytes(bytes) {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

// Formato de fecha
function formatDate(timestamp) {
  if (!timestamp) return "-";
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
}

// Navegación por pestañas
tabButtons.forEach(btn => {
  btn.addEventListener("click", () => {
    tabButtons.forEach(b => b.classList.remove("active"));
    tabPanes.forEach(p => p.classList.remove("active"));

    btn.classList.add("active");
    const targetTab = btn.getAttribute("data-tab");
    document.getElementById(targetTab).classList.add("active");
  });
});

// Validación de compatibilidad teórica del algoritmo
function checkAlgorithmCompatibility() {
  const queryType = queryTypeSelect.value;
  algoCompatibilityAlert.classList.add("hidden");
  algoCompatibilityAlert.textContent = "";

  if (currentActiveAlgorithm === "hash" && (queryType === "substring" || queryType === "prefix")) {
    algoCompatibilityAlert.textContent = "Nota teórica: El índice hash O(1) solo resuelve coincidencias de nombre exacto. Para subcadenas o prefijos debe utilizarse otro algoritmo.";
    algoCompatibilityAlert.classList.remove("hidden");
  } else if (currentActiveAlgorithm === "binary" && queryType === "substring") {
    algoCompatibilityAlert.textContent = "Nota teórica: La búsqueda binaria O(log n) solo soporta nombres exactos y prefijos continuos en el arreglo ordenado, no subcadenas arbitrarias.";
    algoCompatibilityAlert.classList.remove("hidden");
  } else if (currentActiveAlgorithm === "avl" && queryType === "substring") {
    algoCompatibilityAlert.textContent = "Nota teórica: El árbol AVL indexado por nombre soporta búsquedas exactas y por prefijo en O(log n), no subcadenas arbitrarias.";
    algoCompatibilityAlert.classList.remove("hidden");
  }
}

queryTypeSelect.addEventListener("change", checkAlgorithmCompatibility);

// Cargar configuración inicial
async function initApp() {
  try {
    const config = await invoke("get_config");
    if (config) {
      const algKey = (typeof config.algorithm === "string" ? config.algorithm : Object.keys(config.algorithm)[0] || "linear").toLowerCase();
      currentActiveAlgorithm = algKey;
      configAlgoSelect.value = algKey;
      sidebarActiveAlgo.textContent = ALGO_NAMES[algKey] || algKey;
      configTopK.value = config.top_k || 20;
      if (config.last_folder) {
        folderPathInput.value = config.last_folder;
      }
    }

    const count = await invoke("get_indexed_count");
    sidebarIndexedCount.textContent = count || 0;
    checkAlgorithmCompatibility();
  } catch (err) {
    console.error("Error al inicializar:", err);
  }

  // Escuchar eventos de progreso de indexación en tiempo real
  if (listen) {
    listen("index-progress", (event) => {
      const p = event.payload;
      if (!p) return;
      indexingCountText.textContent = `${p.count} procesados`;
      if (p.current_path) {
        indexingCurrentFile.textContent = p.current_path;
      }
      if (p.finished || p.cancelled) {
        indexingProgressContainer.classList.add("hidden");
        btnIndex.classList.remove("hidden");
        btnCancelIndex.classList.add("hidden");
        sidebarIndexedCount.textContent = p.count;
      }
    });
  }
}

// Examinar carpeta mediante diálogo nativo de Tauri
btnBrowse.addEventListener("click", async () => {
  if (tauri && tauri.dialog) {
    try {
      const selected = await tauri.dialog.open({
        directory: true,
        multiple: false,
        title: "Seleccionar carpeta para indexar"
      });
      if (selected && typeof selected === "string") {
        folderPathInput.value = selected;
      }
    } catch (e) {
      console.error("Error abriendo diálogo:", e);
    }
  }
});

// Iniciar indexación
btnIndex.addEventListener("click", async () => {
  const folder = folderPathInput.value.trim();
  if (!folder) {
    alert("Por favor ingresa o selecciona una ruta de carpeta.");
    return;
  }

  btnIndex.classList.add("hidden");
  btnCancelIndex.classList.remove("hidden");
  indexingProgressContainer.classList.remove("hidden");
  indexingStatusText.textContent = "Indexando archivos en hilo secundario...";
  indexingCountText.textContent = "0 procesados";
  indexingCurrentFile.textContent = "Iniciando recorrido...";

  try {
    const total = await invoke("start_indexing", { folderPath: folder });
    sidebarIndexedCount.textContent = total;
  } catch (err) {
    alert("Error de indexación: " + err);
  } finally {
    btnIndex.classList.remove("hidden");
    btnCancelIndex.classList.add("hidden");
    indexingProgressContainer.classList.add("hidden");
  }
});

// Cancelar indexación
btnCancelIndex.addEventListener("click", async () => {
  try {
    await invoke("cancel_indexing");
  } catch (e) {
    console.error("Error cancelando:", e);
  }
});

// Ejecutar búsqueda
async function performSearch() {
  const query = searchInput.value.trim();
  if (!query) return;

  searchSpinner.classList.remove("hidden");
  resultsTbody.innerHTML = `<tr><td colspan="5" class="empty-placeholder">Buscando...</td></tr>`;

  try {
    const results = await invoke("search_files", {
      query: query,
      queryType: queryTypeSelect.value
    });

    resultsMeta.textContent = `${results.length} mejores resultados (Top-K)`;
    renderResults(results);
  } catch (err) {
    resultsTbody.innerHTML = `<tr><td colspan="5" class="empty-placeholder" style="color: var(--danger);">Error: ${err}</td></tr>`;
  } finally {
    searchSpinner.classList.add("hidden");
  }
}

btnSearch.addEventListener("click", performSearch);
searchInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") performSearch();
});

// Renderizar tabla de resultados
function renderResults(results) {
  if (!results || results.length === 0) {
    resultsTbody.innerHTML = `<tr><td colspan="5" class="empty-placeholder">No se encontraron archivos que coincidan con la búsqueda.</td></tr>`;
    return;
  }

  let html = "";
  for (const r of results) {
    const safePath = encodeURIComponent(r.path);
    html += `
      <tr>
        <td class="bold text-truncate" title="${r.name}">${r.name}</td>
        <td class="text-truncate text-muted" title="${r.path}">${r.path}</td>
        <td>${formatBytes(r.size)}</td>
        <td>${formatDate(r.modified)}</td>
        <td class="actions-cell">
          <button class="btn btn-secondary btn-xs" onclick="previewFile('${safePath}')">Ver</button>
          <button class="btn btn-primary btn-xs" onclick="openFile('${safePath}')">Abrir</button>
          <button class="btn btn-secondary btn-xs" onclick="openFolder('${safePath}')">Carpeta</button>
        </td>
      </tr>
    `;
  }
  resultsTbody.innerHTML = html;
}

// Acciones sobre archivos
window.openFile = async function (encodedPath) {
  const path = decodeURIComponent(encodedPath);
  try {
    await invoke("open_file", { filePath: path });
  } catch (e) {
    alert("Error al abrir archivo: " + e);
  }
};

window.openFolder = async function (encodedPath) {
  const path = decodeURIComponent(encodedPath);
  try {
    await invoke("open_folder", { filePath: path });
  } catch (e) {
    alert("Error al abrir carpeta: " + e);
  }
};

window.previewFile = async function (encodedPath) {
  const path = decodeURIComponent(encodedPath);
  currentPreviewPath = path;

  previewModal.classList.remove("hidden");
  previewFileName.textContent = path.split(/[\\/]/).pop() || path;
  previewMeta.textContent = "Cargando...";
  previewBody.textContent = "";
  previewSpinner.classList.remove("hidden");

  try {
    const res = await invoke("read_preview", { filePath: path });
    previewMeta.textContent = `Tamaño: ${formatBytes(res.size)} ${res.truncated ? "(truncado a 512 KB por rendimiento)" : ""}`;
    previewBody.textContent = res.content;
  } catch (err) {
    previewMeta.textContent = "Error al leer archivo";
    previewBody.textContent = err.toString();
  } finally {
    previewSpinner.classList.add("hidden");
  }
};

btnClosePreview.addEventListener("click", () => {
  previewModal.classList.add("hidden");
});

previewModal.addEventListener("click", (e) => {
  if (e.target === previewModal) {
    previewModal.classList.add("hidden");
  }
});

btnModalOpenFile.addEventListener("click", () => {
  if (currentPreviewPath) window.openFile(encodeURIComponent(currentPreviewPath));
});

btnModalOpenFolder.addEventListener("click", () => {
  if (currentPreviewPath) window.openFolder(encodeURIComponent(currentPreviewPath));
});

// Guardar configuración
btnSaveConfig.addEventListener("click", async () => {
  const selectedAlgo = configAlgoSelect.value;
  const topK = parseInt(configTopK.value, 10) || 20;

  try {
    await invoke("set_algorithm", { algorithm: selectedAlgo });
    await invoke("set_top_k", { topK: topK });

    currentActiveAlgorithm = selectedAlgo;
    sidebarActiveAlgo.textContent = ALGO_NAMES[selectedAlgo] || selectedAlgo;
    checkAlgorithmCompatibility();

    configSavedHint.classList.remove("hidden");
    setTimeout(() => {
      configSavedHint.classList.add("hidden");
    }, 2000);
  } catch (err) {
    alert("Error al guardar configuración: " + err);
  }
});

// Comparación de algoritmos
btnRunCompare.addEventListener("click", async () => {
  const query = compareQueryInput.value.trim();
  if (!query) {
    alert("Por favor ingresa una consulta para la prueba comparativa.");
    return;
  }

  btnRunCompare.disabled = true;
  compareSpinner.classList.remove("hidden");
  compareTbody.innerHTML = `<tr><td colspan="7" class="empty-placeholder">Ejecutando mediciones en los 6 algoritmos...</td></tr>`;

  try {
    const comparisons = await invoke("compare_algorithms", {
      query: query,
      queryType: compareTypeSelect.value
    });

    renderComparisonTable(comparisons);
  } catch (err) {
    compareTbody.innerHTML = `<tr><td colspan="7" class="empty-placeholder" style="color: var(--danger);">Error: ${err}</td></tr>`;
  } finally {
    btnRunCompare.disabled = false;
    compareSpinner.classList.add("hidden");
  }
});

function renderComparisonTable(data) {
  if (!data || data.length === 0) {
    compareTbody.innerHTML = `<tr><td colspan="7" class="empty-placeholder">Sin datos de comparación.</td></tr>`;
    return;
  }

  let html = "";
  for (const row of data) {
    const timeDisplay = row.supported ? `${row.search_time_ms.toFixed(4)} ms` : `<span style="color: var(--warning);">-</span>`;
    const memDisplay = `${row.memory_mb.toFixed(3)} MB`;
    const buildDisplay = `${row.build_time_ms.toFixed(2)} ms`;
    const statusNote = row.supported
      ? `<span style="color: var(--success);">✓ Soportado</span> (${row.note})`
      : `<span style="color: var(--warning);">⚠️ ${row.note}</span>`;

    html += `
      <tr>
        <td class="bold">${row.name}</td>
        <td>${row.structure}</td>
        <td>${buildDisplay}</td>
        <td class="bold" style="color: #60a5fa;">${timeDisplay}</td>
        <td>${memDisplay}</td>
        <td>${row.supported ? row.results_count : 0}</td>
        <td style="font-size: 12px;">${statusNote}</td>
      </tr>
    `;
  }
  compareTbody.innerHTML = html;
}

// Inicializar al cargar
window.addEventListener("DOMContentLoaded", initApp);

