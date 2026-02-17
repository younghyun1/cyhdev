import {
  createSignal,
  createEffect,
  For,
  Show,
  onMount,
  onCleanup,
} from "solid-js";
import { UserBadge } from "../components/UserBadge";
import { isSuperuser, user } from "../state/auth";
import {
  dropdownApi,
  wasmModuleApi,
  type WasmModuleItem,
} from "../services/all_api";
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

.admin-panel {
  margin: 1.5rem 0 2rem;
  padding: 1.25rem;
  border-radius: 1rem;
  border: 1px solid #e5e7eb;
  background: linear-gradient(135deg, #f8fafc, #eef2ff);
}
.dark .admin-panel {
  border-color: #1f2937;
  background: linear-gradient(135deg, #0f172a, #111827);
}
.admin-panel-header {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}
.admin-panel-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: #0f172a;
}
.dark .admin-panel-title {
  color: #e2e8f0;
}
.admin-panel-meta {
  font-size: 0.9rem;
  color: #64748b;
}
.dark .admin-panel-meta {
  color: #94a3b8;
}
.admin-panel-badge {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.75rem;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.85);
  border: 1px solid #e2e8f0;
}
.dark .admin-panel-badge {
  background: rgba(15, 23, 42, 0.6);
  border-color: #334155;
}
.admin-panel-badge-label {
  font-size: 0.75rem;
  color: #64748b;
}
.dark .admin-panel-badge-label {
  color: #94a3b8;
}

.project-actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.75rem;
}
.action-button {
  border-radius: 0.5rem;
  border: 1px solid #e5e7eb;
  background: white;
  color: #334155;
  font-size: 0.75rem;
  font-weight: 600;
  padding: 0.35rem 0.6rem;
  cursor: pointer;
  transition: background-color 0.2s, border-color 0.2s, color 0.2s;
}
.action-button:hover {
  background: #f1f5f9;
  border-color: #cbd5f5;
}
.action-button.danger {
  border-color: #fecaca;
  color: #b91c1c;
  background: #fff5f5;
}
.action-button.danger:hover {
  background: #fee2e2;
  border-color: #fca5a5;
}
.dark .action-button {
  background: #0f172a;
  border-color: #334155;
  color: #e2e8f0;
}
.dark .action-button:hover {
  background: #1e293b;
  border-color: #475569;
}
.dark .action-button.danger {
  background: rgba(127, 29, 29, 0.2);
  border-color: rgba(127, 29, 29, 0.6);
  color: #fecaca;
}

.admin-modal {
  background-color: white;
  border-radius: 0.9rem;
  width: 92vw;
  max-width: 640px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.dark .admin-modal {
  background-color: #0f172a;
}
.admin-modal-header {
  padding: 1rem 1.5rem;
  border-bottom: 1px solid #e5e7eb;
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.dark .admin-modal-header {
  border-color: #1f2937;
}
.admin-modal-body {
  padding: 1.25rem 1.5rem 1.5rem;
}
.form-grid {
  display: grid;
  gap: 0.9rem;
}
.form-field {
  display: grid;
  gap: 0.4rem;
}
.form-label {
  font-size: 0.8rem;
  font-weight: 600;
  color: #475569;
}
.dark .form-label {
  color: #cbd5f5;
}
.form-help {
  font-size: 0.75rem;
  color: #94a3b8;
}
.dark .form-help {
  color: #64748b;
}
.progress-track {
  width: 100%;
  height: 8px;
  border-radius: 999px;
  background: #e5e7eb;
  overflow: hidden;
}
.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #0f172a, #4f46e5);
  transition: width 0.2s ease;
}
.dark .progress-track {
  background: #1f2937;
}
`;

export default function Projects() {
  const [modules, setModules] = createSignal<WasmModuleItem[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [selectedModule, setSelectedModule] =
    createSignal<WasmModuleItem | null>(null);
  const [showUpload, setShowUpload] = createSignal(false);
  const [uploading, setUploading] = createSignal(false);
  const [uploadProgress, setUploadProgress] = createSignal(0);
  const [uploadError, setUploadError] = createSignal<string | null>(null);
  const [uploadTitle, setUploadTitle] = createSignal("");
  const [uploadDescription, setUploadDescription] = createSignal("");
  const [uploadBundle, setUploadBundle] = createSignal<File | null>(null);
  const [uploadThumbnail, setUploadThumbnail] = createSignal<File | null>(null);
  const [editingModule, setEditingModule] = createSignal<WasmModuleItem | null>(
    null,
  );
  const [editTitle, setEditTitle] = createSignal("");
  const [editDescription, setEditDescription] = createSignal("");
  const [editBundle, setEditBundle] = createSignal<File | null>(null);
  const [editThumbnail, setEditThumbnail] = createSignal<File | null>(null);
  const [savingEdit, setSavingEdit] = createSignal(false);
  const [actionError, setActionError] = createSignal<string | null>(null);
  const [actionSuccess, setActionSuccess] = createSignal<string | null>(null);
  const [deleteInProgress, setDeleteInProgress] = createSignal<string | null>(
    null,
  );
  const [userCountryFlag, setUserCountryFlag] = createSignal<
    string | undefined
  >(undefined);

  const loadModules = async (opts?: { silent?: boolean }) => {
    if (!opts?.silent) setLoading(true);
    setError(null);
    try {
      const response = await wasmModuleApi.getWasmModules();
      setModules(response.data.items);
    } catch (e) {
      console.error("Failed to load WASM modules:", e);
      setError("Failed to load projects. Please try again later.");
    } finally {
      if (!opts?.silent) setLoading(false);
    }
  };

  // Fetch modules on mount
  onMount(async () => {
    await loadModules();
  });

  // Close modal on Escape key
  createEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && selectedModule()) {
        setSelectedModule(null);
      }
      if (e.key === "Escape" && showUpload()) {
        setShowUpload(false);
      }
      if (e.key === "Escape" && editingModule()) {
        setEditingModule(null);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    onCleanup(() => window.removeEventListener("keydown", handleKeyDown));
  });

  createEffect(() => {
    const current = editingModule();
    if (current) {
      setEditTitle(current.wasm_module_title);
      setEditDescription(current.wasm_module_description);
      setEditBundle(null);
      setEditThumbnail(null);
    }
  });

  createEffect(() => {
    const countryId = user()?.user_info?.user_country;
    if (!countryId) {
      setUserCountryFlag(undefined);
      return;
    }
    dropdownApi
      .country(countryId)
      .then((response) => setUserCountryFlag(response.data.country_flag))
      .catch(() => setUserCountryFlag(undefined));
  });

  const handleUpload = async (e: Event) => {
    e.preventDefault();
    setUploadError(null);
    setActionSuccess(null);
    setActionError(null);

    if (!uploadBundle() || !uploadThumbnail()) {
      setUploadError("Please provide both a bundle file and a thumbnail.");
      return;
    }
    if (!uploadTitle().trim() || !uploadDescription().trim()) {
      setUploadError("Title and description are required.");
      return;
    }

    setUploading(true);
    setUploadProgress(0);

    try {
      const formData = new FormData();
      formData.append("bundle_file", uploadBundle()!);
      formData.append("thumbnail", uploadThumbnail()!);
      formData.append("title", uploadTitle().trim());
      formData.append("description", uploadDescription().trim());

      await wasmModuleApi.uploadWasmModule(formData, {
        onUploadProgress: (percent) => {
          setUploadProgress((prev) => (percent > prev ? percent : prev));
        },
      });

      setUploadProgress(100);
      setActionSuccess("Project uploaded successfully.");
      setShowUpload(false);
      setUploadTitle("");
      setUploadDescription("");
      setUploadBundle(null);
      setUploadThumbnail(null);
      await loadModules({ silent: true });
    } catch (err: unknown) {
      console.error("Failed to upload WASM module:", err);
      setUploadError("Upload failed. Please check the files and try again.");
    } finally {
      setUploading(false);
      setUploadProgress(0);
    }
  };

  const handleUpdate = async (e: Event) => {
    e.preventDefault();
    const current = editingModule();
    if (!current) return;

    if (!editTitle().trim() || !editDescription().trim()) {
      setActionError("Title and description are required.");
      return;
    }

    setSavingEdit(true);
    setActionError(null);
    setActionSuccess(null);

    try {
      const formData = new FormData();
      formData.append("title", editTitle().trim());
      formData.append("description", editDescription().trim());
      if (editBundle()) formData.append("bundle_file", editBundle()!);
      if (editThumbnail()) formData.append("thumbnail", editThumbnail()!);

      const response = await wasmModuleApi.updateWasmModuleAssets(
        current.wasm_module_id,
        formData,
      );

      const updated = response.data;
      setModules((prev) =>
        prev.map((item) =>
          item.wasm_module_id === updated.wasm_module_id ? updated : item,
        ),
      );
      if (selectedModule()?.wasm_module_id === updated.wasm_module_id) {
        setSelectedModule(updated);
      }
      setEditingModule(null);
      setActionSuccess("Project details updated.");
    } catch (err) {
      console.error("Failed to update WASM module:", err);
      setActionError("Update failed. Please try again.");
    } finally {
      setSavingEdit(false);
    }
  };

  const handleDelete = async (module: WasmModuleItem) => {
    if (
      !confirm(`Delete "${module.wasm_module_title}"? This cannot be undone.`)
    ) {
      return;
    }

    setDeleteInProgress(module.wasm_module_id);
    setActionError(null);
    setActionSuccess(null);

    try {
      await wasmModuleApi.deleteWasmModule(module.wasm_module_id);
      setModules((prev) =>
        prev.filter((item) => item.wasm_module_id !== module.wasm_module_id),
      );
      if (selectedModule()?.wasm_module_id === module.wasm_module_id) {
        setSelectedModule(null);
      }
      setActionSuccess("Project deleted.");
    } catch (err) {
      console.error("Failed to delete WASM module:", err);
      setActionError("Delete failed. Please try again.");
    } finally {
      setDeleteInProgress(null);
    }
  };

  return (
    <main class={pageStyles.page}>
      <style>{styles}</style>
      <div class={pageStyles.pageInner}>
        <h1 class={pageStyles.title}>Projects & Demos</h1>
        <hr class={`${pageStyles.divider} my-4`} />
        <p class={pageStyles.muted}>
          Interactive WASM demos and projects. Click a card to launch the demo.
        </p>

        <Show when={isSuperuser()}>
          <section class="admin-panel">
            <div class="admin-panel-header">
              <div>
                <h2 class="admin-panel-title">Manage Projects</h2>
                <p class="admin-panel-meta">
                  Upload new bundles, update details, or remove old demos.
                </p>
              </div>
              <div class="admin-panel-badge">
                <span class="admin-panel-badge-label">Signed in as</span>
                <UserBadge
                  userName={user()?.user_info?.user_name || "You"}
                  profilePictureUrl={
                    user()?.user_profile_picture?.user_profile_picture_link ||
                    undefined
                  }
                  countryFlag={userCountryFlag()}
                  size="md"
                />
              </div>
              <button
                class={pageStyles.buttonPrimary}
                onClick={() => setShowUpload(true)}
              >
                Upload Project
              </button>
            </div>

            <Show when={actionError()}>
              <div class={`${pageStyles.alertError} mt-4`}>{actionError()}</div>
            </Show>
            <Show when={actionSuccess()}>
              <div class={`${pageStyles.alertSuccess} mt-4`}>
                {actionSuccess()}
              </div>
            </Show>
          </section>
        </Show>

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
                    <p class="project-description">
                      {module.wasm_module_description}
                    </p>
                    <Show when={isSuperuser()}>
                      <div class="project-actions">
                        <button
                          class="action-button"
                          onClick={(e) => {
                            e.stopPropagation();
                            setEditingModule(module);
                          }}
                        >
                          Edit details
                        </button>
                        <button
                          class="action-button danger"
                          disabled={
                            deleteInProgress() === module.wasm_module_id
                          }
                          onClick={(e) => {
                            e.stopPropagation();
                            handleDelete(module);
                          }}
                        >
                          {deleteInProgress() === module.wasm_module_id
                            ? "Deleting..."
                            : "Delete"}
                        </button>
                      </div>
                    </Show>
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
              <h2 class="wasm-modal-title">
                {selectedModule()!.wasm_module_title}
              </h2>
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

      {/* Upload Modal */}
      <Show when={showUpload()}>
        <div
          class="modal-overlay"
          onClick={(e) => {
            if (e.target === e.currentTarget) {
              setShowUpload(false);
            }
          }}
        >
          <div class="admin-modal">
            <div class="admin-modal-header">
              <h2 class="wasm-modal-title">Upload Project</h2>
              <button class="close-button" onClick={() => setShowUpload(false)}>
                Close
              </button>
            </div>
            <div class="admin-modal-body">
              <form class="form-grid" onSubmit={handleUpload}>
                <div class="form-field">
                  <label class="form-label" for="wasm-title">
                    Title
                  </label>
                  <input
                    id="wasm-title"
                    class={pageStyles.input}
                    value={uploadTitle()}
                    onInput={(e) => setUploadTitle(e.currentTarget.value)}
                    placeholder="Project title"
                  />
                </div>

                <div class="form-field">
                  <label class="form-label" for="wasm-description">
                    Description
                  </label>
                  <textarea
                    id="wasm-description"
                    class={pageStyles.textarea}
                    value={uploadDescription()}
                    onInput={(e) => setUploadDescription(e.currentTarget.value)}
                    placeholder="Short description for the demo"
                  />
                </div>

                <div class="form-field">
                  <label class="form-label" for="wasm-bundle">
                    Bundle file
                  </label>
                  <input
                    id="wasm-bundle"
                    class={pageStyles.input}
                    type="file"
                    accept=".html,.htm,.html.gz,.htm.gz,.wasm,.wasm.gz"
                    onChange={(e) =>
                      setUploadBundle(e.currentTarget.files?.[0] || null)
                    }
                  />
                  <span class="form-help">
                    Upload a `.html`/`.html.gz` bundle or a `.wasm` file.
                  </span>
                </div>

                <div class="form-field">
                  <label class="form-label" for="wasm-thumbnail">
                    Thumbnail image
                  </label>
                  <input
                    id="wasm-thumbnail"
                    class={pageStyles.input}
                    type="file"
                    accept="image/*"
                    onChange={(e) =>
                      setUploadThumbnail(e.currentTarget.files?.[0] || null)
                    }
                  />
                  <span class="form-help">
                    Recommended: square image for best grid cropping.
                  </span>
                </div>

                <Show when={uploadError()}>
                  <div class={pageStyles.alertError}>{uploadError()}</div>
                </Show>

                <Show when={uploading()}>
                  <div class="progress-track">
                    <div
                      class="progress-bar"
                      style={{ width: `${uploadProgress()}%` }}
                    />
                  </div>
                </Show>

                <div class="flex gap-2 justify-end">
                  <button
                    type="button"
                    class={pageStyles.buttonSecondary}
                    onClick={() => setShowUpload(false)}
                    disabled={uploading()}
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    class={pageStyles.buttonPrimary}
                    disabled={uploading()}
                  >
                    {uploading() ? "Uploading..." : "Upload"}
                  </button>
                </div>
              </form>
            </div>
          </div>
        </div>
      </Show>

      {/* Edit Modal */}
      <Show when={editingModule()}>
        <div
          class="modal-overlay"
          onClick={(e) => {
            if (e.target === e.currentTarget) {
              setEditingModule(null);
            }
          }}
        >
          <div class="admin-modal">
            <div class="admin-modal-header">
              <h2 class="wasm-modal-title">Edit Project Details</h2>
              <button
                class="close-button"
                onClick={() => setEditingModule(null)}
              >
                Close
              </button>
            </div>
            <div class="admin-modal-body">
              <form class="form-grid" onSubmit={handleUpdate}>
                <div class="form-field">
                  <label class="form-label" for="edit-title">
                    Title
                  </label>
                  <input
                    id="edit-title"
                    class={pageStyles.input}
                    value={editTitle()}
                    onInput={(e) => setEditTitle(e.currentTarget.value)}
                  />
                </div>
                <div class="form-field">
                  <label class="form-label" for="edit-description">
                    Description
                  </label>
                  <textarea
                    id="edit-description"
                    class={pageStyles.textarea}
                    value={editDescription()}
                    onInput={(e) => setEditDescription(e.currentTarget.value)}
                  />
                </div>
                <div class="form-field">
                  <label class="form-label" for="edit-bundle">
                    Replace bundle (optional)
                  </label>
                  <input
                    id="edit-bundle"
                    class={pageStyles.input}
                    type="file"
                    accept=".html,.htm,.html.gz,.htm.gz,.wasm,.wasm.gz"
                    onChange={(e) =>
                      setEditBundle(e.currentTarget.files?.[0] || null)
                    }
                  />
                  <span class="form-help">
                    Upload a new `.html`/`.html.gz` bundle or `.wasm` file.
                  </span>
                </div>
                <div class="form-field">
                  <label class="form-label" for="edit-thumbnail">
                    Replace thumbnail (optional)
                  </label>
                  <input
                    id="edit-thumbnail"
                    class={pageStyles.input}
                    type="file"
                    accept="image/*"
                    onChange={(e) =>
                      setEditThumbnail(e.currentTarget.files?.[0] || null)
                    }
                  />
                  <span class="form-help">
                    Recommended: square image for best grid cropping.
                  </span>
                </div>

                <div class="flex gap-2 justify-end">
                  <button
                    type="button"
                    class={pageStyles.buttonSecondary}
                    onClick={() => setEditingModule(null)}
                    disabled={savingEdit()}
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    class={pageStyles.buttonPrimary}
                    disabled={savingEdit()}
                  >
                    {savingEdit() ? "Saving..." : "Save changes"}
                  </button>
                </div>
              </form>
            </div>
          </div>
        </div>
      </Show>
    </main>
  );
}
