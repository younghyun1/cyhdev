import { A, useParams } from "@solidjs/router";
import { Show, createResource } from "solid-js";
import { userApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import { UserBadge } from "../components/UserBadge";
import { t, tx } from "../state/i18n";

export default function UserInfoPage() {
  const params = useParams<{ userName: string }>();
  const [userInfo] = createResource(
    () => params.userName,
    async (userName) => await userApi.getPublicUserInfo(userName),
  );

  return (
    <main class={pageStyles.page}>
      <section class={pageStyles.pageInner}>
        <A href="/blog" class={pageStyles.link}>
          {t("user.back_to_blog")}
        </A>

        <Show
          when={userInfo()?.data}
          fallback={
            <section class={`${pageStyles.card} mt-6 p-6`}>
              <p class={pageStyles.subtitle}>
                {userInfo.loading ? t("user.loading") : t("user.not_found")}
              </p>
            </section>
          }
        >
          {(publicUser) => (
            <section class={`${pageStyles.card} mt-6 p-6`}>
              <div class="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
                <div class="flex items-center gap-4">
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
                      class="h-16 w-16 rounded-full border border-line object-cover"
                    />
                  </Show>
                  <div>
                    <UserBadge
                      userName={publicUser().user_name}
                      profilePictureUrl={
                        publicUser().user_profile_picture_url ?? undefined
                      }
                      countryFlag={publicUser().user_country_flag}
                      size="md"
                      link={false}
                    />
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
      </section>
    </main>
  );
}
