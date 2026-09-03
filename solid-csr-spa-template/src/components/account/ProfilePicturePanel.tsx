import {
  Show,
  createMemo,
  createSignal,
  onCleanup,
  onSettled,
} from "solid-js";
import type {
  DeleteProfilePictureResponse,
  ProfilePictureHistoryItem,
} from "../../generated";
import { authApi, userApi } from "../../services/all_api";
import { uploadWithProgress } from "../../services/upload_with_progress";
import { setUser, user } from "../../state/auth";
import { t } from "../../state/i18n";
import { pageStyles } from "../../styles/pageStyles";
import ProfilePictureHistory from "./ProfilePictureHistory";

const MAX_PROFILE_PICTURE_BYTES = 10 * 1024 * 1024;
const MAX_PROFILE_PICTURE_HISTORY = 8;
const ALLOWED_MIME_TYPES = new Set([
  "image/png",
  "image/jpeg",
  "image/gif",
  "image/webp",
  "image/x-portable-anymap",
  "image/tiff",
  "image/x-tga",
  "image/vnd-ms.dds",
  "image/bmp",
  "image/vnd.microsoft.icon",
  "image/vnd.radiance",
  "image/x-exr",
  "image/farbfeld",
  "image/avif",
  "image/qoi",
  "image/vnd.zbrush.pcx",
]);

export default function ProfilePicturePanel() {
  const [profileImage, setProfileImage] = createSignal<File | null>(null);
  const [uploading, setUploading] = createSignal(false);
  const [uploadError, setUploadError] = createSignal<string | null>(null);
  const [uploadSucceeded, setUploadSucceeded] = createSignal(false);
  const [progress, setProgress] = createSignal(0);
  const [profilePictureUrl, setProfilePictureUrl] = createSignal<string | null>(
    user()?.user_profile_picture?.user_profile_picture_link ?? null,
  );
  const [previewUrl, setPreviewUrl] = createSignal<string | null>(null);
  const [history, setHistory] = createSignal<
    ReadonlyArray<ProfilePictureHistoryItem>
  >([]);
  const [maximumHistory, setMaximumHistory] = createSignal(
    MAX_PROFILE_PICTURE_HISTORY,
  );
  const [historyLoading, setHistoryLoading] = createSignal(true);
  const [historyError, setHistoryError] = createSignal<string | null>(null);
  let fileInput: HTMLInputElement | undefined;

  const activeHistoryUrl = createMemo(
    () => history().find((item) => item.is_active)?.object_url ?? null,
  );
  const replacePreview = (next: string | null) => {
    setPreviewUrl((current) => {
      if (current !== null) URL.revokeObjectURL(current);
      return next;
    });
  };

  onCleanup(() => {
    const current = previewUrl();
    if (current !== null) URL.revokeObjectURL(current);
  });

  const refreshProfileState = async () => {
    setHistoryLoading(true);
    setHistoryError(null);
    try {
      const [historyResponse, meResponse] = await Promise.all([
        userApi.profilePictures(),
        authApi.me(),
      ]);
      if (!historyResponse.success || !meResponse.success || !meResponse.data) {
        throw new Error(t("profile.picture_history.load_failed"));
      }
      setHistory(historyResponse.data.profile_pictures);
      setMaximumHistory(
        Math.min(
          MAX_PROFILE_PICTURE_HISTORY,
          historyResponse.data.maximum_profile_pictures,
        ),
      );
      setUser(meResponse.data);
      setProfilePictureUrl(
        meResponse.data.user_profile_picture?.user_profile_picture_link ?? null,
      );
    } catch (caught: unknown) {
      setHistoryError(
        caught instanceof Error
          ? caught.message
          : t("profile.picture_history.load_failed"),
      );
      throw caught;
    } finally {
      setHistoryLoading(false);
    }
  };

  onSettled(() => {
    refreshProfileState().catch(() => {});
  });

  const handleFileChange = (event: Event) => {
    const target = event.currentTarget as HTMLInputElement;
    const file = target.files?.[0];
    if (file === undefined) return;
    setUploadError(null);
    setUploadSucceeded(false);
    if (file.size > MAX_PROFILE_PICTURE_BYTES) {
      setProfileImage(null);
      replacePreview(null);
      setUploadError(t("profile.file_too_big"));
      return;
    }
    if (!ALLOWED_MIME_TYPES.has(file.type)) {
      setProfileImage(null);
      replacePreview(null);
      setUploadError(t("profile.unsupported_image"));
      return;
    }
    setProfileImage(file);
    replacePreview(URL.createObjectURL(file));
  };

  const handleUpload = async () => {
    const image = profileImage();
    if (image === null) {
      setUploadError(t("profile.select_image"));
      return;
    }
    setUploading(true);
    setUploadError(null);
    setUploadSucceeded(false);
    setProgress(0);
    const formData = new FormData();
    formData.append("profile_picture", image);
    try {
      const response = await uploadWithProgress<null>({
        url: "/api/user/upload-profile-picture",
        formData,
        onProgress: setProgress,
      });
      if (!response.success) {
        setUploadError(t("profile.upload_failed"));
        return;
      }
      await refreshProfileState();
      setProfileImage(null);
      replacePreview(null);
      setUploadSucceeded(true);
    } catch (caught: unknown) {
      setUploadError(
        caught instanceof Error
          ? caught.message
          : t("profile.upload_unknown_error"),
      );
    } finally {
      setUploading(false);
    }
  };

  const selectProfilePicture = async (profilePictureId: string) => {
    const response = await userApi.selectProfilePicture(profilePictureId);
    if (!response.success) {
      throw new Error(t("profile.picture_history.action_failed"));
    }
    await refreshProfileState();
  };

  const deleteProfilePicture = async (
    profilePictureId: string,
  ): Promise<DeleteProfilePictureResponse> => {
    const response = await userApi.deleteProfilePicture(profilePictureId);
    if (!response.success) {
      throw new Error(t("profile.picture_history.action_failed"));
    }
    await refreshProfileState();
    return response.data;
  };

  return (
    <section class={`${pageStyles.cardPadded} profile-picture-panel`}>
      <h2 class="text-lg font-semibold">{t("profile.change_picture")}</h2>
      <hr class={`my-3 ${pageStyles.divider}`} />
      <div class="profile-picture-layout flex items-start gap-6">
        <div class="h-32 w-32 shrink-0 overflow-hidden rounded-full ring-2 ring-line shadow">
          <img
            src={
              previewUrl() ??
              activeHistoryUrl() ??
              profilePictureUrl() ??
              "/default-profile.png"
            }
            alt={t("profile.picture_alt")}
            class="h-full w-full object-cover"
          />
        </div>
        <div class="min-w-0 flex-1">
          <input
            ref={fileInput}
            type="file"
            accept="image/*"
            onChange={handleFileChange}
            class="hidden"
          />
          <div class="profile-picture-actions mt-1 flex flex-wrap gap-2">
            <button
              type="button"
              onClick={() => fileInput?.click()}
              disabled={uploading()}
              class={pageStyles.buttonSecondary}
            >
              {t("profile.choose_image")}
            </button>
            <button
              type="button"
              onClick={handleUpload}
              disabled={uploading() || profileImage() === null}
              class={pageStyles.buttonPrimary}
            >
              {uploading() ? t("common.uploading") : t("common.upload")}
            </button>
          </div>
          <p class={`${pageStyles.muted} mt-3`}>{t("profile.image_help")}</p>
          <Show when={uploading()}>
            <div class="mt-4 w-full">
              <div class="h-2 overflow-hidden rounded-full bg-surface-2">
                <div
                  class="h-full bg-ink transition-all"
                  style={{ width: `${progress()}%` }}
                />
              </div>
              <div class="mt-1 text-xs text-ink-muted">{progress()}%</div>
            </div>
          </Show>
          <Show when={uploadError()}>
            <div class={`${pageStyles.alertError} mt-3`} role="alert">
              {uploadError()}
            </div>
          </Show>
          <Show when={uploadSucceeded()}>
            <div class={`${pageStyles.alertSuccess} mt-3`} aria-live="polite">
              {t("profile.upload_success")}
            </div>
          </Show>
        </div>
      </div>
      <ProfilePictureHistory
        items={history()}
        maximum={maximumHistory()}
        loading={historyLoading()}
        loadError={historyError()}
        onSelect={selectProfilePicture}
        onDelete={deleteProfilePicture}
      />
    </section>
  );
}
