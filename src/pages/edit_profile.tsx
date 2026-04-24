import {
  createSignal,
  Show,
  onCleanup,
  createResource,
  createMemo,
} from "solid-js";
import { uploadWithProgress } from "../services/upload_with_progress";
import { user, setUser } from "../state/auth";
import { authApi, dropdownApi } from "../services/all_api";
import type {
  IsoCountry,
  IsoCountrySubdivision,
  IsoLanguage,
} from "../dtos/responses/dropdown";
import { pageStyles } from "../styles/pageStyles";
import { t } from "../state/i18n";

const MAX_SIZE_OF_UPLOADABLE_PROFILE_PICTURE = 10 * 1024 * 1024; // 10MB
const ALLOWED_MIME_TYPES: string[] = [
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
];

function EditProfilePage() {
  const [profileImage, setProfileImage] = createSignal<File | null>(null);
  const [uploading, setUploading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [profilePicUrl, setProfilePicUrl] = createSignal<string | null>(
    user()?.user_profile_picture?.user_profile_picture_link ?? null,
  );
  const [progress, setProgress] = createSignal<number>(0);
  const [selectedPreviewUrl, setSelectedPreviewUrl] = createSignal<
    string | null
  >(null);

  const userCountryId = () => user()?.user_info?.user_country;

  // Load dropdown caches/resources
  const [countries] = createResource(
    async () => await dropdownApi.countryList(),
  );
  const [languages] = createResource(
    async () => await dropdownApi.languageList(),
  );
  const [subdivisions] = createResource(userCountryId, async (cid) => {
    if (cid === null || cid === undefined) return [];
    try {
      const res = await dropdownApi.countrySubdivisions(Number(cid));
      return Array.isArray(res?.data) ? res.data : [];
    } catch {
      return [];
    }
  });

  // Build lookup maps
  const countryMap = createMemo(() => {
    const res = countries();
    const list = Array.isArray(res?.data?.countries) ? res.data.countries : [];
    const m: Record<number, IsoCountry> = {};
    for (const c of list) {
      m[Number(c.country_code)] = c;
    }
    return m;
  });
  const languageMap = createMemo(() => {
    const res = languages();
    const list = Array.isArray(res?.data) ? res.data : [];
    const m: Record<number, IsoLanguage> = {};
    for (const l of list) {
      m[Number(l.language_code)] = l;
    }
    return m;
  });
  const subdivisionMap = createMemo(() => {
    const res = subdivisions();
    const list = Array.isArray(res) ? res : [];
    const m: Record<number, IsoCountrySubdivision> = {};
    for (const s of list) {
      m[Number(s.subdivision_id)] = s;
    }
    return m;
  });

  const formatCountry = (code: number | null | undefined) => {
    if (code == null) return "";
    const c = countryMap()[Number(code)];
    if (!c) return String(code);
    const flag = c.country_flag ? c.country_flag + " " : "";
    const name = c.country_eng_name ?? "";
    return `${flag}${name || code}`.trim();
  };
  const formatLanguage = (code: number | null | undefined) => {
    if (code == null) return "";
    const l = languageMap()[Number(code)];
    if (!l) return String(code);
    return l.language_eng_name ?? String(code);
  };
  const formatSubdivision = (id: number | null | undefined) => {
    if (id == null) return t("auth.signup.no_subdivision");
    const s = subdivisionMap()[Number(id)];
    if (!s) return String(id);
    return s.subdivision_name ?? String(id);
  };

  let fileInputRef: HTMLInputElement | undefined;

  onCleanup(() => {
    const url = selectedPreviewUrl();
    if (url) URL.revokeObjectURL(url);
  });

  const handleFileChange = (e: Event) => {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    setError(null);

    if (file.size > MAX_SIZE_OF_UPLOADABLE_PROFILE_PICTURE) {
      setProfileImage(null);
      setSelectedPreviewUrl((prev) => {
        if (prev) URL.revokeObjectURL(prev);
        return null;
      });
      setError(t("profile.file_too_big"));
      return;
    }

    if (!ALLOWED_MIME_TYPES.includes(file.type)) {
      setProfileImage(null);
      setSelectedPreviewUrl((prev) => {
        if (prev) URL.revokeObjectURL(prev);
        return null;
      });
      setError(t("profile.unsupported_image"));
      return;
    }

    setProfileImage(file);
    setSelectedPreviewUrl((prev) => {
      if (prev) URL.revokeObjectURL(prev);
      return URL.createObjectURL(file);
    });
  };

  const handleUpload = async () => {
    if (!profileImage()) {
      setError(t("profile.select_image"));
      return;
    }
    setUploading(true);
    setError(null);
    setProgress(0);

    const formData = new FormData();
    formData.append("profile_picture", profileImage() as File);

    try {
      // Use XMLHttpRequest-based utility for progress
      const resp = await uploadWithProgress({
        url: "/api/user/upload-profile-picture",
        formData,
        onProgress: setProgress,
      });
      if (resp && resp.success) {
        try {
          const me = await authApi.me();
          if (me?.success && me.data) {
            setUser(me.data);
            setProfilePicUrl(
              me.data.user_profile_picture?.user_profile_picture_link ?? null,
            );
            setProfileImage(null);
            setSelectedPreviewUrl((prev) => {
              if (prev) URL.revokeObjectURL(prev);
              return null;
            });
          } else {
            setError(t("profile.refresh_failed"));
          }
        } catch (e: unknown) {
          setError(
            e instanceof Error ? e.message : t("profile.refresh_failed"),
          );
        }
      } else {
        setError(t("profile.upload_failed"));
      }
    } catch (err: unknown) {
      setError(
        err instanceof Error ? err.message : t("profile.upload_unknown_error"),
      );
    } finally {
      setUploading(false);
    }
  };

  const readOnlyInputClass = `${pageStyles.input} bg-slate-100 dark:bg-slate-900/60 text-slate-700 dark:text-slate-200 disabled:opacity-70`;

  return (
    <main class={pageStyles.page}>
      <div class={pageStyles.pageInnerNarrow}>
        <h1 class={`${pageStyles.title} mb-6`}>
          {t("page.edit_profile.title")}
        </h1>

        <div class="space-y-6">
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-semibold">{t("profile.change_picture")}</h2>
            <hr class={`my-3 ${pageStyles.divider}`} />
            <div class="flex items-start gap-6">
              <div class="w-32 h-32 rounded-full overflow-hidden ring-2 ring-gray-200 dark:ring-gray-700 shadow">
                <img
                  src={
                    selectedPreviewUrl() ||
                    profilePicUrl() ||
                    user()?.user_profile_picture?.user_profile_picture_link ||
                    "/default-profile.png"
                  }
                  alt={t("profile.picture_alt")}
                  class="w-full h-full object-cover"
                />
              </div>
              <div class="flex-1">
                <input
                  ref={fileInputRef}
                  type="file"
                  accept="image/*"
                  onChange={handleFileChange}
                  class="hidden"
                />

                <div class="flex gap-2 mt-1 flex-wrap">
                  <button
                    type="button"
                    onClick={() => fileInputRef?.click()}
                    disabled={uploading()}
                    class={`${pageStyles.buttonSecondary} disabled:opacity-60`}
                  >
                    {t("profile.choose_image")}
                  </button>

                  <button
                    onClick={handleUpload}
                    disabled={uploading() || !profileImage()}
                    class={`${pageStyles.buttonPrimary} disabled:opacity-60`}
                  >
                    {uploading() ? t("common.uploading") : t("common.upload")}
                  </button>
                </div>

                <div class={`${pageStyles.muted} mt-3`}>
                  {t("profile.image_help")}
                </div>

                <Show when={uploading()}>
                  <div class="w-full mt-4">
                    <div class="h-2 bg-gray-200 dark:bg-gray-800 rounded-full overflow-hidden">
                      <div
                        class="h-full bg-slate-900 dark:bg-slate-100 transition-all"
                        style={{ width: `${progress()}%` }}
                      />
                    </div>
                    <div class="text-xs mt-1 text-gray-600 dark:text-gray-300">
                      {progress()}%
                    </div>
                  </div>
                </Show>

                <Show when={error()}>
                  <div
                    class={`mt-3 ${pageStyles.alertError}`}
                    aria-live="polite"
                  >
                    {error()}
                  </div>
                </Show>
              </div>
            </div>
          </section>

          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-semibold">{t("profile.info")}</h2>
            <hr class={`my-3 ${pageStyles.divider}`} />
            <div class="space-y-4">
              <div class="grid grid-cols-12 items-center gap-3">
                <div class={`col-span-4 ${pageStyles.muted}`}>
                  {t("profile.display_name")}
                </div>
                <div class="col-span-8">
                  <input
                    disabled
                    value={user()?.user_info?.user_name ?? ""}
                    class={readOnlyInputClass}
                  />
                </div>
              </div>
              <div class="grid grid-cols-12 items-center gap-3">
                <div class={`col-span-4 ${pageStyles.muted}`}>
                  {t("common.email")}
                </div>
                <div class="col-span-8">
                  <input
                    disabled
                    value={user()?.user_info?.user_email ?? ""}
                    class={readOnlyInputClass}
                  />
                </div>
              </div>
              <div class="grid grid-cols-12 items-center gap-3">
                <div class={`col-span-4 ${pageStyles.muted}`}>
                  {t("profile.user_id")}
                </div>
                <div class="col-span-8">
                  <input
                    disabled
                    value={user()?.user_info?.user_id ?? ""}
                    class={readOnlyInputClass}
                  />
                </div>
              </div>
              <div class="grid grid-cols-12 items-center gap-3">
                <div class={`col-span-4 ${pageStyles.muted}`}>
                  {t("profile.email_verified")}
                </div>
                <div class="col-span-8">
                  <input
                    disabled
                    value={
                      user()?.user_info?.user_is_email_verified
                        ? t("common.yes")
                        : t("common.no")
                    }
                    class={readOnlyInputClass}
                  />
                </div>
              </div>
              <div class="grid grid-cols-12 items-center gap-3">
                <div class={`col-span-4 ${pageStyles.muted}`}>
                  {t("common.country")}
                </div>
                <div class="col-span-8">
                  <input
                    disabled
                    value={formatCountry(user()?.user_info?.user_country)}
                    class={readOnlyInputClass}
                  />
                </div>
              </div>
              <div class="grid grid-cols-12 items-center gap-3">
                <div class={`col-span-4 ${pageStyles.muted}`}>
                  {t("common.language")}
                </div>
                <div class="col-span-8">
                  <input
                    disabled
                    value={formatLanguage(user()?.user_info?.user_language)}
                    class={readOnlyInputClass}
                  />
                </div>
              </div>
              <div class="grid grid-cols-12 items-center gap-3">
                <div class={`col-span-4 ${pageStyles.muted}`}>
                  {t("common.subdivision")}
                </div>
                <div class="col-span-8">
                  <input
                    disabled
                    value={formatSubdivision(
                      user()?.user_info?.user_subdivision,
                    )}
                    class={readOnlyInputClass}
                  />
                </div>
              </div>
            </div>
          </section>
        </div>
      </div>
    </main>
  );
}

export default EditProfilePage;
