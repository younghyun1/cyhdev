import { createSignal, createEffect, For, Show, onMount, onCleanup } from "solid-js";
import { wasmModuleApi, type WasmModuleItem } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";

const styles = `
.projects-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1.5rem;
  padding: 1rem 0;
}

.project-card {
  border-radius: 0.75rem;
  overflow: hidden;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
  background-color: white;
  border: 1px solid #e5e7eb;
}
.dark .project-card {
  background-color: #1f2937;
  border-color: #374151;
}
.project-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.15);
}

.project-thumbnail {
  width: 100%;
  aspect-ratio: 1;
  object-fit: cover;
  background-color: #f3f4f6;
}
.dark .project-thumbnail {
  background-color: #374151;
}

.project-info {
  padding: 1rem;
}

.project-title {
  font-size: 1.125rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: #111827;
}
.dark .project-title {
  color: #f9fafb;
}

.project-description {
  font-size: 0.875rem;
  color: #6b7280;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.dark .project-description {
  color: #9ca3af;
}

/* Modal styles */
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.85);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 50;
  padding: 1rem;
}

.wasm-modal {
  background-color: white;
  border-radius: 0.75rem;
  width: 90vw;
  height: 85vh;
  max-width: 1200px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.dark .wasm-modal {
  background-color: #1f2937;
}

.wasm-modal-header {
  padding: 1rem 1.5rem;
  border-bottom: 1px solid #e5e7eb;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}
.dark .wasm-modal-header {
  border-color: #374151;
}

.wasm-modal-title {
  font-size: 1.25rem;
  font-weight: 600;
  color: #111827;
}
.dark .wasm-modal-title {
  color: #f9fafb;
}

.wasm-iframe-container {
  flex: 1;
  overflow: hidden;
  background-color: #000;
}

.wasm-iframe {
  width: 100%;
  height: 100%;
  border: none;
}

.close-button {
  padding: 0.5rem 1rem;
  border-radius: 0.375rem;
  border: 1px solid #d1d5db;
  background-color: transparent;
  color: #374151;
  font-size: 0.875rem;
  cursor: pointer;
  transition: background-color 0.2s;
}
.close-button:hover {
  background-color: #f3f4f6;
}
.dark .close-button {
  border-color: #4b5563;
  color: #d1d5db;
}
.dark .close-button:hover {
  background-color: #374151;
}

.empty-state {
  text-align: center;
  padding: 4rem 2rem;
  color: #6b7280;
}
.dark .empty-state {
  color: #9ca3af;
}
`;

export default function Projects() {
  const [modules, setModules] = createSignal<WasmModuleItem[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [selectedModule, setSelectedModule] = createSignal<WasmModuleItem | null>(null);

  // Fetch modules on mount
  onMount(async () => {
    try {
      const response = await wasmModuleApi.getWasmModules();
      setModules(response.data.items);
    } catch (e) {
      console.error("Failed to load WASM modules:", e);
      setError("Failed to load projects. Please try again later.");
    } finally {
      setLoading(false);
    }
  });

  // Close modal on Escape key
  createEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && selectedModule()) {
        setSelectedModule(null);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    onCleanup(() => window.removeEventListener("keydown", handleKeyDown));
  });

  return (
    <main class={pageStyles.page}>
      <style>{styles}</style>
      <div class={pageStyles.pageInner}>
        <h1 class={pageStyles.title}>Projects & Demos</h1>
        <hr class={`${pageStyles.divider} my-4`} />
        <p class={pageStyles.muted}>
          Interactive WASM demos and projects. Click a card to launch the demo.
        </p>

        <Show when={loading()}>
          <div class="empty-state">
            <p>Loading projects...</p>
          </div>
        </Show>

        <Show when={error()}>
          <div class={pageStyles.alertError}>{error()}</div>
        </Show>

        <Show when={!loading() && !error() && modules().length === 0}>
          <div class="empty-state">
            <p>No projects available yet. Check back later!</p>
          </div>
        </Show>

        <Show when={!loading() && !error() && modules().length > 0}>
          <div class="projects-grid">
            <For each={modules()}>
              {(module) => (
                <div
                  class="project-card"
                  onClick={() => setSelectedModule(module)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      setSelectedModule(module);
                    }
                  }}
                >
                  <img
                    src={module.wasm_module_thumbnail_link}
                    alt={module.wasm_module_title}
                    class="project-thumbnail"
                    loading="lazy"
                  />
                  <div class="project-info">
                    <h3 class="project-title">{module.wasm_module_title}</h3>
                    <p class="project-description">{module.wasm_module_description}</p>
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>

      {/* WASM Module Modal */}
      <Show when={selectedModule()}>
        <div
          class="modal-overlay"
          onClick={(e) => {
            if (e.target === e.currentTarget) {
              setSelectedModule(null);
            }
          }}
        >
          <div class="wasm-modal">
            <div class="wasm-modal-header">
              <h2 class="wasm-modal-title">{selectedModule()!.wasm_module_title}</h2>
              <button
                class="close-button"
                onClick={() => setSelectedModule(null)}
              >
                Close (Esc)
              </button>
            </div>
            <div class="wasm-iframe-container">
              <iframe
                src={selectedModule()!.wasm_module_link}
                class="wasm-iframe"
                title={selectedModule()!.wasm_module_title}
                sandbox="allow-scripts allow-same-origin allow-forms"
              />
            </div>
          </div>
        </div>
      </Show>
    </main>
  );
}
