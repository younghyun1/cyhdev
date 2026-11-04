import { useParams } from "@solidjs/router";
import { Loading, Show, createMemo } from "solid-js";
import { userApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import { t, tx } from "../state/i18n";

export default function UserInfoPage() {
  const params = useParams<{ userName: string }>();
  const userInfo = createMemo(async () => {
    const userName = params.userName;
    try {
      return await userApi.getPublicUserInfo(userName);
    } catch {
      // Renders the not-found card below instead of the route error boundary.
      return null;
    }
  });

  return (
    <main class={pageStyles.page}>
      <section class={pageStyles.pageInner}>
        <a href="/blog" class={pageStyles.link}>
          {t("user.back_to_blog")}
        </a>

        <Loading
          fallback={
            <section class={`${pageStyles.card} mt-6 p-6`}>
              <p class={pageStyles.subtitle}>{t("user.loading")}</p>
            </section>
          }
        >
        <Show
          when={userInfo()?.data}
          fallback={
            <section class={`${pageStyles.card} mt-6 p-6`}>
              <p class={pageStyles.subtitle}>{t("user.not_found")}</p>
            </section>
          }
        >
          {(publicUser) => (
            <section class={`${pageStyles.card} mt-6 p-6`}>
              <div class="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
                <div class="flex min-w-0 flex-col items-start gap-4 sm:flex-row sm:items-center">
                  <Show
                    when={publicUser().user_profile_picture_url}
                    fallback={
                      <div class="flex h-16 w-16 shrink-0 items-center justify-center rounded-full border border-line bg-surface-2 font-mono text-2xl font-bold uppercase text-ink-muted">
                        {publicUser().user_name.slice(0, 1)}
                      </div>
                    }
                  >
                    <img
                      src={publicUser().user_profile_picture_url ?? undefined}
                      alt={publicUser().user_name}
                      class="h-16 w-16 shrink-0 rounded-full border border-line object-cover"
                    />
                  </Show>
                  <div class="min-w-0 max-w-full">
                    <h1 class="user-info-name">
                      {publicUser().user_name}
                      <Show when={publicUser().user_country_flag}>
                        <span class="user-info-flag">{publicUser().user_country_flag}</span>
                      </Show>
                    </h1>
                    <p class={pageStyles.subtitle}>
                      {tx("user.joined", {
                        date: new Date(
                          publicUser().user_created_at,
                        ).toLocaleDateString(undefined, {
                          year: "numeric",
                          month: "long",
                          day: "numeric",
                        }),
                      })}
                    </p>
                  </div>
                </div>
              </div>
            </section>
          )}
        </Show>
        </Loading>
      </section>
    </main>
  );
}
