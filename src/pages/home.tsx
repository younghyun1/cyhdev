import { createResource, For, Show, Suspense } from "solid-js";
import { A } from "@solidjs/router";
import { blogApi, photographyApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import { t } from "../state/i18n";

export default function Home() {
  const [posts] = createResource(() =>
    blogApi.getPosts({ page: 1, posts_per_page: 3 }),
  );
  const [photos] = createResource(() => photographyApi.getPhotographs(1, 4));

  const getPhotoItems = () => {
    const res = photos();
    if (!res?.data) return [];
    return res.data.items ?? [];
  };

  return (
    <main class={`${pageStyles.page} font-sans`}>
      {/* Top Navigation / Compact Hero */}
      <header class="border-b border-line">
        <div class="max-w-7xl mx-auto px-6 py-8 md:py-10">
          <div class="flex flex-col md:flex-row md:items-start justify-between gap-6">
            {/* Identity Block */}
            <div class="space-y-2 max-w-2xl">
              <h1 class="text-4xl md:text-3xl font-bold tracking-tighter uppercase font-mono">
                Younghyun Chi //{" "}
                <span class="font-sans tracking-normal normal-case">
                  지영현
                </span>{" "}
                // 池營賢 // 池营贤
              </h1>
              <p class="text-lg md:text-xl text-ink-muted font-medium leading-relaxed border-l-4 border-accent pl-4">
                {t("home.hero.role")}
                <span class="block text-sm mt-2 font-mono text-ink-muted">
                  {t("home.hero.summary")}
                  <br />
                  <br />
                  {t("home.hero.principle")}
                </span>
              </p>
            </div>

            {/* Quick Links / Command Center */}
            <div class="flex flex-col gap-3 font-mono text-sm shrink-0">
              <div class="text-ink-faint uppercase text-xs tracking-widest mb-1">
                {t("home.connect.title")}
              </div>
              <a
                href="mailto:younghyun1@gmail.com"
                class="group flex items-center gap-2 hover:text-accent transition-colors"
              >
                <span class="opacity-50 group-hover:opacity-100">[</span> EMAIL{" "}
                <span class="opacity-50 group-hover:opacity-100">]</span>
              </a>
              <a
                href="https://github.com/younghyun1"
                target="_blank"
                rel="noreferrer"
                class="group flex items-center gap-2 hover:text-accent transition-colors"
              >
                <span class="opacity-50 group-hover:opacity-100">[</span> GITHUB{" "}
                <span class="opacity-50 group-hover:opacity-100">]</span>
              </a>
              <a
                href="https://www.linkedin.com/in/young-hyun-chi-553431376/"
                target="_blank"
                rel="noreferrer"
                class="group flex items-center gap-2 hover:text-accent transition-colors"
              >
                <span class="opacity-50 group-hover:opacity-100">[</span>{" "}
                LINKEDIN{" "}
                <span class="opacity-50 group-hover:opacity-100">]</span>
              </a>
            </div>
          </div>

          {/* Action Bar */}
          <div class="mt-8 flex gap-4">
            <A
              href="/blog"
              class={`${pageStyles.buttonPrimary} px-6 py-3 text-base font-mono`}
            >
              {t("home.cta.blog")}
            </A>
            <A
              href="/photographs"
              class={`${pageStyles.buttonSecondary} px-6 py-3 text-base font-mono`}
            >
              {t("home.cta.photography")}
            </A>
          </div>
        </div>
      </header>

      {/* Main Content Grid */}
      <div class="grow">
        <div class="relative mx-auto max-w-7xl px-6 py-12">
          <div class="grid w-full grid-cols-1 gap-8 lg:grid-cols-12">
            {/* Latest Blog Posts - Spans 7 columns */}
            <section class="lg:col-span-7 flex flex-col h-full">
              <div class="flex items-end justify-between mb-6 pb-2 border-b border-dashed border-line">
                <h2 class="text-2xl font-bold uppercase font-mono tracking-tight flex items-center gap-2">
                  <span class="w-3 h-3 bg-accent" /> {t("home.latest_posts")}
                </h2>
                <A href="/blog" class={`${pageStyles.link} font-mono text-sm`}>
                  {t("home.view_blog_posts")}
                </A>
              </div>

              <div class="grow space-y-4">
                <Suspense
                  fallback={
                    <div class="space-y-4">
                      <For each={[1, 2, 3]}>
                        {() => (
                          <div class="h-24 bg-surface-2 animate-pulse border border-line" />
                        )}
                      </For>
                    </div>
                  }
                >
                  <Show
                    when={posts()}
                    fallback={
                      <div class="p-6 border border-line bg-surface font-mono text-sm">
                        {t("home.no_data_found")}
                      </div>
                    }
                  >
                    <For each={posts()?.data?.posts}>
                      {(post) => (
                        <article class="group relative bg-surface p-5 border border-line hover:border-ink transition-colors duration-200">
                          <div class="flex flex-col gap-1">
                            <div class="flex justify-between items-center text-xs font-mono tabular-nums text-ink-muted mb-1">
                              <div class="flex items-center gap-2">
                                <span>
                                  {new Date(
                                    post.post_created_at,
                                  ).toLocaleDateString(undefined, {
                                    year: "numeric",
                                    month: "long",
                                    day: "numeric",
                                  })}
                                </span>
                                <Show when={!post.post_is_published}>
                                  <span class="rounded-full bg-accent-soft px-2 py-0.5 text-[0.65rem] font-semibold uppercase tracking-wide text-accent">
                                    {t("common.draft")}
                                  </span>
                                </Show>
                              </div>
                              <span class="opacity-0 group-hover:opacity-100 text-accent transition-opacity">
                                {t("home.read")}
                              </span>
                            </div>
                            <h3 class="text-xl font-bold text-ink group-hover:text-accent transition-colors">
                              <A
                                href={`/blog/${encodeURIComponent(post.post_slug || post.post_id)}`}
                              >
                                <span class="absolute inset-0" />
                                {post.post_title}
                              </A>
                            </h3>
                          </div>
                        </article>
                      )}
                    </For>
                  </Show>
                </Suspense>
              </div>
            </section>

            {/* Recent Photographs - Spans 5 columns */}
            <section class="lg:col-span-5 flex flex-col h-full">
              <div class="flex items-end justify-between mb-6 pb-2 border-b border-dashed border-line">
                <h2 class="text-2xl font-bold uppercase font-mono tracking-tight flex items-center gap-2">
                  <span class="w-3 h-3 bg-accent" /> {t("home.photography")}
                </h2>
                <A
                  href="/photographs"
                  class={`${pageStyles.link} font-mono text-sm`}
                >
                  {t("home.view_gallery")}
                </A>
              </div>

              <div class="grid grid-cols-2 gap-3">
                <Suspense
                  fallback={
                    <>
                      <For each={[1, 2, 3, 4]}>
                        {() => (
                          <div class="aspect-square bg-surface-2 animate-pulse border border-line" />
                        )}
                      </For>
                    </>
                  }
                >
                  <Show
                    when={getPhotoItems().length > 0}
                    fallback={
                      <div class="col-span-2 p-6 border border-line font-mono text-sm text-center">
                        /img/null
                      </div>
                    }
                  >
                    <For each={getPhotoItems()}>
                      {(photo) => (
                        <A
                          href="/photographs"
                          class="group relative block aspect-square overflow-hidden bg-surface-2 border border-line"
                        >
                          <img
                            src={
                              photo.photograph_thumbnail_link ||
                              photo.photograph_link
                            }
                            alt={photo.photograph_comments || t("home.photography")}
                            class="w-full h-full object-cover transition-all duration-300 group-hover:scale-105"
                            loading="lazy"
                          />
                          {/* Crosshair overlay effect */}
                          <div class="absolute inset-0 border-2 border-transparent group-hover:border-accent/50 transition-colors pointer-events-none z-10" />
                          <div class="absolute top-2 right-2 text-[10px] font-mono bg-black text-white px-1 opacity-0 group-hover:opacity-100 transition-opacity z-20">
                            {photo.photograph_id}
                          </div>
                        </A>
                      )}
                    </For>
                  </Show>
                </Suspense>
              </div>
            </section>
          </div>
        </div>
      </div>
    </main>
  );
}
