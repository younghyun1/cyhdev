import {
  createSignal,
  createEffect,
  For,
  Show,
  onSettled,
  untrack,
} from "solid-js";
import { UserBadge } from "../components/UserBadge";
import { isSuperuser, user } from "../state/auth";
import {
  dropdownApi,
  wasmModuleApi,
  type WasmModuleItem,
} from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import { t, tx } from "../state/i18n";
import { MobileDialog } from "../components/MobileDialog";
import "../styles/projects.css";

export default function Projects() {
  const [modules, setModules] = createSignal<ReadonlyArray<WasmModuleItem>>([]);
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
      setError(t("projects.load_failed"));
    } finally {
      if (!opts?.silent) setLoading(false);
    }
  };

  // Fetch modules on mount
  onSettled(() => {
    void loadModules();
  });

  // Close modal on Escape key
  onSettled(() => {
    const handleKeyDown = (e: KeyboardEvent) =>
      untrack(() => {
        if (e.key === "Escape" && selectedModule()) setSelectedModule(null);
        if (e.key === "Escape" && showUpload()) setShowUpload(false);
        if (e.key === "Escape" && editingModule()) setEditingModule(null);
      });
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  });

  createEffect(
    () => editingModule(),
    (current) => {
      if (current) {
        setEditTitle(current.wasm_module_title);
        setEditDescription(current.wasm_module_description);
        setEditBundle(null);
        setEditThumbnail(null);
      }
    },
  );

  createEffect(
    () => user()?.user_info?.user_country,
    (countryId) => {
      if (!countryId) {
        setUserCountryFlag(undefined);
        return;
      }
      dropdownApi
        .country(countryId)
        .then((response) =>
          setUserCountryFlag(response.data.country.country_flag),
        )
        .catch(() => setUserCountryFlag(undefined));
    },
  );

  const handleUpload = async (e: Event) => {
    e.preventDefault();
    setUploadError(null);
    setActionSuccess(null);
    setActionError(null);

    if (!uploadBundle() || !uploadThumbnail()) {
      setUploadError(t("projects.upload_require_files"));
      return;
    }
    if (!uploadTitle().trim() || !uploadDescription().trim()) {
      setUploadError(t("projects.upload_require_text"));
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
      setActionSuccess(t("projects.upload_success"));
      setShowUpload(false);
      setUploadTitle("");
      setUploadDescription("");
      setUploadBundle(null);
      setUploadThumbnail(null);
      await loadModules({ silent: true });
    } catch (err: unknown) {
      console.error("Failed to upload WASM module:", err);
      setUploadError(t("projects.upload_failed"));
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
      setActionError(t("projects.upload_require_text"));
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
      setActionSuccess(t("projects.update_success"));
    } catch (err) {
      console.error("Failed to update WASM module:", err);
      setActionError(t("projects.update_failed"));
    } finally {
      setSavingEdit(false);
    }
  };

  const handleDelete = async (module: WasmModuleItem) => {
    if (
      !confirm(tx("projects.delete_confirm", { title: module.wasm_module_title }))
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
      setActionSuccess(t("projects.deleted"));
    } catch (err) {
      console.error("Failed to delete WASM module:", err);
      setActionError(t("projects.delete_failed"));
    } finally {
      setDeleteInProgress(null);
    }
  };

  return (
    <main class={pageStyles.page}>
      <div class={pageStyles.pageInner}>
        <h1 class={pageStyles.title}>{t("page.projects.title")}</h1>
        <hr class={`${pageStyles.divider} my-4`} />
        <p class={pageStyles.muted}>
          {t("projects.subtitle")}
        </p>

        <Show when={isSuperuser()}>
          <section class="admin-panel">
            <div class="admin-panel-header">
              <div>
                <h2 class="admin-panel-title">{t("projects.manage")}</h2>
                <p class="admin-panel-meta">
                  {t("projects.manage_subtitle")}
                </p>
              </div>
              <div class="admin-panel-badge">
                <span class="admin-panel-badge-label">
                  {t("projects.signed_in_as")}
                </span>
                <UserBadge
                  userName={user()?.user_info?.user_name || t("live_chat.you")}
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
                {t("projects.upload_project")}
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
            <p>{t("projects.loading")}</p>
          </div>
        </Show>

        <Show when={error()}>
          <div class={pageStyles.alertError}>{error()}</div>
        </Show>

        <Show when={!loading() && !error() && modules().length === 0}>
          <div class="empty-state">
            <p>{t("projects.empty")}</p>
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
                  tabindex={0}
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
                          {t("projects.edit_details")}
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
                            ? t("common.deleting")
                            : t("common.delete")}
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
        <MobileDialog
          onClose={() => setSelectedModule(null)}
          overlayClass="modal-overlay"
          panelClass="wasm-modal"
          ariaLabelledBy="wasm-modal-title"
          initialFocusSelector="[data-project-close]"
        >
            <div class="wasm-modal-header">
              <h2 id="wasm-modal-title" class="wasm-modal-title">
                {selectedModule()!.wasm_module_title}
              </h2>
              <a
                href={selectedModule()!.wasm_module_link}
                target="_blank"
                rel="noopener noreferrer"
                class={`${pageStyles.buttonSecondary} mobile-only wasm-open-separately`}
              >
                {t("projects.open_separately")}
              </a>
              <button
                data-project-close
                type="button"
                class="close-button"
                onClick={() => setSelectedModule(null)}
              >
                {t("common.close_esc")}
              </button>
            </div>
            <div class="wasm-iframe-container">
              <iframe
                src={selectedModule()!.wasm_module_link}
                class="wasm-iframe"
                title={selectedModule()!.wasm_module_title}
                sandbox="allow-scripts"
              />
            </div>
        </MobileDialog>
      </Show>

      {/* Upload Modal */}
      <Show when={showUpload()}>
        <MobileDialog
          onClose={() => {
            if (!uploading()) setShowUpload(false);
          }}
          overlayClass="modal-overlay"
          panelClass="admin-modal"
          ariaLabelledBy="project-upload-title"
          initialFocusSelector="#wasm-title"
        >
            <div class="admin-modal-header">
              <h2 id="project-upload-title" class="wasm-modal-title">
                {t("projects.upload_project")}
              </h2>
              <button class="close-button" onClick={() => setShowUpload(false)}>
                {t("common.close")}
              </button>
            </div>
            <div class="admin-modal-body">
              <form class="form-grid" onSubmit={handleUpload}>
                <div class="form-field">
                  <label class="form-label" for="wasm-title">
                    {t("common.title")}
                  </label>
                  <input
                    id="wasm-title"
                    class={pageStyles.input}
                    value={uploadTitle()}
                    onInput={(e) => setUploadTitle(e.currentTarget.value)}
                    placeholder={t("projects.project_title_placeholder")}
                  />
                </div>

                <div class="form-field">
                  <label class="form-label" for="wasm-description">
                    {t("common.description")}
                  </label>
                  <textarea
                    id="wasm-description"
                    class={pageStyles.textarea}
                    value={uploadDescription()}
                    onInput={(e) => setUploadDescription(e.currentTarget.value)}
                    placeholder={t("projects.description_placeholder")}
                  />
                </div>

                <div class="form-field">
                  <label class="form-label" for="wasm-bundle">
                    {t("projects.bundle_file")}
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
                    {t("projects.bundle_help")}
                  </span>
                </div>

                <div class="form-field">
                  <label class="form-label" for="wasm-thumbnail">
                    {t("projects.thumbnail_image")}
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
                    {t("projects.thumbnail_help")}
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
                    {t("common.cancel")}
                  </button>
                  <button
                    type="submit"
                    class={pageStyles.buttonPrimary}
                    disabled={uploading()}
                  >
                    {uploading() ? t("common.uploading") : t("common.upload")}
                  </button>
                </div>
              </form>
            </div>
        </MobileDialog>
      </Show>

      {/* Edit Modal */}
      <Show when={editingModule()}>
        <MobileDialog
          onClose={() => {
            if (!savingEdit()) setEditingModule(null);
          }}
          overlayClass="modal-overlay"
          panelClass="admin-modal"
          ariaLabelledBy="project-edit-title"
          initialFocusSelector="#edit-title"
        >
            <div class="admin-modal-header">
              <h2 id="project-edit-title" class="wasm-modal-title">
                {t("projects.edit_project_details")}
              </h2>
              <button
                class="close-button"
                onClick={() => setEditingModule(null)}
              >
                {t("common.close")}
              </button>
            </div>
            <div class="admin-modal-body">
              <form class="form-grid" onSubmit={handleUpdate}>
                <div class="form-field">
                  <label class="form-label" for="edit-title">
                    {t("common.title")}
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
                    {t("common.description")}
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
                    {t("projects.replace_bundle")}
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
                    {t("projects.bundle_help")}
                  </span>
                </div>
                <div class="form-field">
                  <label class="form-label" for="edit-thumbnail">
                    {t("projects.replace_thumbnail")}
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
                    {t("projects.thumbnail_help")}
                  </span>
                </div>

                <div class="flex gap-2 justify-end">
                  <button
                    type="button"
                    class={pageStyles.buttonSecondary}
                    onClick={() => setEditingModule(null)}
                    disabled={savingEdit()}
                  >
                    {t("common.cancel")}
                  </button>
                  <button
                    type="submit"
                    class={pageStyles.buttonPrimary}
                    disabled={savingEdit()}
                  >
                    {savingEdit()
                      ? t("common.saving")
                      : t("projects.save_changes")}
                  </button>
                </div>
              </form>
            </div>
        </MobileDialog>
      </Show>
    </main>
  );
}
