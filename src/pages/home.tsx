import { createResource, For, Show, Suspense } from "solid-js";
import { A } from "@solidjs/router";
import { blogApi, photographyApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";

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
      <header class="border-b border-slate-200/80 dark:border-slate-800">
        <div class="max-w-7xl mx-auto px-6 py-8 md:py-10">
          <div class="flex flex-col md:flex-row md:items-start justify-between gap-6">
            {/* Identity Block */}
            <div class="space-y-2 max-w-2xl">
              <h1 class="text-4xl md:text-3xl font-black tracking-tighter uppercase font-mono">
                Younghyun Chi //{" "}
                <span class="font-sans tracking-normal normal-case">
                  지영현
                </span>{" "}
                // 池營賢 // 池营贤
              </h1>
              <p class="text-lg md:text-xl text-slate-600 dark:text-slate-400 font-medium leading-relaxed border-l-4 border-amber-500 pl-4">
                Experienced backend, infrastructure, and data engineer.
                <span class="block text-sm mt-2 font-mono text-slate-500">
                  Passionate about creating secure and high-performance servers
                  and infrastructure. I also enjoy photography and have dabbled
                  in journalism, translation/interpretation, soldiering, manual
                  labor, activism, and various misadventures.
                  <br></br>
                  <br></br>I hold craftsmanship and good governance to be
                  sacred.
                </span>
              </p>
            </div>

            {/* Quick Links / Command Center */}
            <div class="flex flex-col gap-3 font-mono text-sm shrink-0">
              <div class="text-slate-400 uppercase text-xs tracking-widest mb-1">
                Connect
              </div>
              <a
                href="mailto:younghyun1@gmail.com"
                class="group flex items-center gap-2 hover:text-amber-600 dark:hover:text-amber-400 transition-colors"
              >
                <span class="opacity-50 group-hover:opacity-100">[</span> EMAIL{" "}
                <span class="opacity-50 group-hover:opacity-100">]</span>
              </a>
              <a
                href="https://github.com/younghyun1"
                target="_blank"
                rel="noreferrer"
                class="group flex items-center gap-2 hover:text-amber-600 dark:hover:text-amber-400 transition-colors"
              >
                <span class="opacity-50 group-hover:opacity-100">[</span> GITHUB{" "}
                <span class="opacity-50 group-hover:opacity-100">]</span>
              </a>
              <a
                href="https://www.linkedin.com/in/young-hyun-chi-553431376/"
                target="_blank"
                rel="noreferrer"
                class="group flex items-center gap-2 hover:text-amber-600 dark:hover:text-amber-400 transition-colors"
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
              Read Blog
            </A>
            <A
              href="/photographs"
              class={`${pageStyles.buttonSecondary} px-6 py-3 text-base font-mono`}
            >
              View My Photography
            </A>
          </div>
        </div>
      </header>

      {/* Main Content Grid */}
      <div class="grow max-w-7xl mx-auto w-full px-6 py-12 grid grid-cols-1 lg:grid-cols-12 gap-12">
        {/* Latest Blog Posts - Spans 7 columns */}
        <section class="lg:col-span-7 flex flex-col h-full">
          <div class="flex items-end justify-between mb-6 pb-2 border-b border-dashed border-slate-200/80 dark:border-slate-800">
            <h2 class="text-2xl font-black uppercase font-mono tracking-tight flex items-center gap-2">
              <span class="w-3 h-3 bg-amber-600"></span> Latest Posts
            </h2>
            <A href="/blog" class={`${pageStyles.link} font-mono text-sm`}>
              view blog posts &rarr;
            </A>
          </div>

          <div class="grow space-y-4">
            <Suspense
              fallback={
                <div class="space-y-4">
                  {[1, 2, 3].map(() => (
                    <div class="h-24 bg-slate-200 dark:bg-slate-800 animate-pulse border border-slate-300 dark:border-slate-700"></div>
                  ))}
                </div>
              }
            >
              <Show
                when={posts()}
                fallback={
                  <div class="p-6 border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900 font-mono text-sm">
                    No data found.
                  </div>
                }
              >
                <For each={posts()?.data?.posts}>
                  {(post) => (
                    <article class="group relative bg-white dark:bg-slate-900 p-5 border border-slate-200 dark:border-slate-800 hover:border-slate-900 dark:hover:border-slate-100 transition-colors duration-200">
                      <div class="flex flex-col gap-1">
                        <div class="flex justify-between items-center text-xs font-mono text-slate-500 dark:text-slate-400 mb-1">
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
                              <span class="rounded-full bg-amber-100 px-2 py-0.5 text-[0.65rem] font-semibold uppercase tracking-wide text-amber-800 dark:bg-amber-900/40 dark:text-amber-200">
                                Draft
                              </span>
                            </Show>
                          </div>
                          <span class="opacity-0 group-hover:opacity-100 text-amber-600 dark:text-amber-400 transition-opacity">
                            &lt;READ&gt;
                          </span>
                        </div>
                        <h3 class="text-xl font-bold text-slate-900 dark:text-slate-100 group-hover:text-amber-600 dark:group-hover:text-amber-400 transition-colors">
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
          <div class="flex items-end justify-between mb-6 pb-2 border-b border-dashed border-slate-200/80 dark:border-slate-800">
            <h2 class="text-2xl font-black uppercase font-mono tracking-tight flex items-center gap-2">
              <span class="w-3 h-3 bg-amber-600"></span> Photography
            </h2>
            <A
              href="/photographs"
              class={`${pageStyles.link} font-mono text-sm`}
            >
              view gallery &rarr;
            </A>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <Suspense
              fallback={
                <>
                  {[1, 2, 3, 4].map(() => (
                    <div class="aspect-square bg-slate-200 dark:bg-slate-800 animate-pulse border border-slate-300 dark:border-slate-700"></div>
                  ))}
                </>
              }
            >
              <Show
                when={getPhotoItems().length > 0}
                fallback={
                  <div class="col-span-2 p-6 border border-slate-200 dark:border-slate-800 font-mono text-sm text-center">
                    /img/null
                  </div>
                }
              >
                <For each={getPhotoItems()}>
                  {(photo) => (
                    <A
                      href="/photographs"
                      class="group relative block aspect-square overflow-hidden bg-slate-100 dark:bg-slate-800 border border-slate-200 dark:border-slate-800"
                    >
                      <img
                        src={
                          photo.photograph_thumbnail_link ||
                          photo.photograph_link
                        }
                        alt={photo.photograph_comments || "Photograph"}
                        class="w-full h-full object-cover transition-all duration-300 group-hover:scale-105"
                        loading="lazy"
                      />
                      {/* Crosshair overlay effect */}
                      <div class="absolute inset-0 border-2 border-transparent group-hover:border-amber-500/50 transition-colors pointer-events-none z-10"></div>
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
    </main>
  );
}
