import {
  Errored,
  For,
  Loading,
  Show,
  createMemo,
  createSignal,
  lazy,
  onCleanup,
  onSettled,
} from "solid-js";
import type { JSX } from "@solidjs/web";
import HeroField from "../components/home/HeroField";
import { blogApi, photographyApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import { locale, t } from "../state/i18n";

/** Chat and its call code stay out of the initial route graph. */
const LiveChatPanel = lazy(() => import("../components/LiveChatPanel"));

/** Recent engineering write-ups surfaced on the home page. */
const SHOWCASE = [
  { id: "pgo", href: "/about-blog/performance" },
  { id: "nodelay", href: "/about-blog/performance#transport" },
  { id: "sfu", href: "/about-blog/subsystems#live-chat" },
  { id: "minecraft", href: "/minecraft" },
  { id: "geoip", href: "/about-blog/subsystems#geo-ip" },
  { id: "eu5", href: "/eu5-locations-db" },
] as const;

const CONNECT_LINKS = [
  { label: "EMAIL", href: "mailto:younghyun1@gmail.com", external: false },
  { label: "GITHUB", href: "https://github.com/younghyun1", external: true },
  {
    label: "LINKEDIN",
    href: "https://www.linkedin.com/in/young-hyun-chi-553431376/",
    external: true,
  },
] as const;

const POST_COUNT = 5;
const PHOTO_COUNT = 6;

function SectionHeader(props: {
  readonly id?: string;
  readonly title: string;
  readonly href: string;
  readonly linkLabel: string;
}): JSX.Element {
  return (
    <div class="home-section-header">
      <h2 id={props.id} class="home-section-title">
        <span class="home-section-marker" aria-hidden="true" /> {props.title}
      </h2>
      <a href={props.href} class={`${pageStyles.link} home-section-link`}>
        {props.linkLabel}
      </a>
    </div>
  );
}

/**
 * Mounts chat once its slot nears the viewport, so a phone visitor who never
 * scrolls to it does not download the chunk or open a socket.
 */
function HomeChat(): JSX.Element {
  const [near, setNear] = createSignal(typeof IntersectionObserver === "undefined");
  let slot: HTMLDivElement | undefined;
  let observer: IntersectionObserver | undefined;

  onSettled(() => {
    if (near() || !slot) return;
    observer = new IntersectionObserver(
      (entries) => {
        if (!entries.some((entry) => entry.isIntersecting)) return;
        setNear(true);
        observer?.disconnect();
      },
      { rootMargin: "120px" },
    );
    observer.observe(slot);
  });
  onCleanup(() => observer?.disconnect());

  const skeleton = <div class="home-skeleton home-chat-skeleton" />;
  return (
    <div class="home-chat-slot" ref={(element) => (slot = element)}>
      {/* A failed chunk, as after a deploy, must not take the whole page with it. */}
      <Errored fallback={<div class="home-empty">{t("live_chat.connection_failed")}</div>}>
        <Show when={near()} fallback={skeleton}>
          <Loading fallback={skeleton}>
            <LiveChatPanel mode="compact" />
          </Loading>
        </Show>
      </Errored>
    </div>
  );
}

export default function Home() {
  const posts = createMemo(() =>
    blogApi.getPosts({ page: 1, posts_per_page: POST_COUNT }),
  );
  const photos = createMemo(() => photographyApi.getPhotographs(1, PHOTO_COUNT));

  const getPhotoItems = () => {
    const res = photos();
    if (!res?.data) return [];
    return res.data.items ?? [];
  };

  let hero: HTMLElement | undefined;

  return (
    <main class={`${pageStyles.page} home-page font-sans`}>
      <header class="home-hero" ref={(element) => (hero = element)}>
        <HeroField pointerTarget={() => hero} />
        <div class="home-hero-inner">
          <div class="home-hero-identity">
            <h1 class="home-hero-title">
              Younghyun Chi{" "}
              <span class="home-hero-names">
                //{" "}
                <span class="font-sans tracking-normal normal-case">지영현</span>{" "}
                // 池營賢 // 池营贤
              </span>
            </h1>
            <p class="home-hero-role">{t("home.hero.role")}</p>
            <p class="home-hero-summary">{t("home.hero.summary")}</p>
            <p class="home-hero-summary">{t("home.hero.principle")}</p>
          </div>

          <div class="home-hero-side">
            <div class="home-actions">
              <a href="/blog" class={`${pageStyles.buttonPrimary} font-mono`}>
                {t("home.cta.blog")}
              </a>
              <a href="/photographs" class={`${pageStyles.buttonSecondary} font-mono`}>
                {t("home.cta.photography")}
              </a>
            </div>
            <div class="home-connect">
              <span class="home-connect-title">{t("home.connect.title")}</span>
              <For each={CONNECT_LINKS}>
                {(link) => (
                  <a
                    href={link.href}
                    class="home-connect-link"
                    target={link.external ? "_blank" : undefined}
                    rel={link.external ? "noreferrer" : undefined}
                  >
                    <span aria-hidden="true">[</span> {link.label}{" "}
                    <span aria-hidden="true">]</span>
                  </a>
                )}
              </For>
            </div>
          </div>
        </div>
      </header>

      <div class="home-board">
        <section class="home-panel home-panel--posts">
          <SectionHeader
            title={t("home.latest_posts")}
            href="/blog"
            linkLabel={t("home.view_blog_posts")}
          />
          <Loading
            fallback={
              <div class="home-post-list">
                <For each={[1, 2, 3, 4, 5]}>
                  {() => <div class="home-skeleton home-post-skeleton" />}
                </For>
              </div>
            }
          >
            <Show
              when={posts()}
              fallback={<div class="home-empty">{t("home.no_data_found")}</div>}
            >
              <div class="home-post-list">
                <For each={posts()?.data?.posts}>
                  {(post) => (
                    <article class="home-post">
                      <h3 class="home-post-title">
                        <a
                          href={`/blog/${encodeURIComponent(post.post_slug || post.post_id)}`}
                        >
                          {post.post_title}
                        </a>
                      </h3>
                      <div class="home-post-meta">
                        <time datetime={post.post_created_at}>
                          {new Date(post.post_created_at).toLocaleDateString(locale(), {
                            year: "numeric",
                            month: "short",
                            day: "numeric",
                          })}
                        </time>
                        <Show when={!post.post_is_published}>
                          <span class="home-post-draft">{t("common.draft")}</span>
                        </Show>
                      </div>
                    </article>
                  )}
                </For>
              </div>
            </Show>
          </Loading>
        </section>

        <section class="home-panel home-panel--photos">
          <SectionHeader
            title={t("home.photography")}
            href="/photographs"
            linkLabel={t("home.view_gallery")}
          />
          <div class="home-photo-grid">
            <Loading
              fallback={
                <For each={[1, 2, 3, 4, 5, 6]}>
                  {() => <div class="home-skeleton home-photo-skeleton" />}
                </For>
              }
            >
              <Show
                when={getPhotoItems().length > 0}
                fallback={<div class="home-empty home-photo-empty">/img/null</div>}
              >
                <For each={getPhotoItems()}>
                  {(photo) => (
                    <a
                      href={`/photographs/${encodeURIComponent(photo.photograph_id)}`}
                      class="home-photo"
                    >
                      <img
                        src={photo.photograph_thumbnail_link || photo.photograph_link}
                        alt={photo.photograph_comments || t("home.photography")}
                        loading="lazy"
                        decoding="async"
                      />
                    </a>
                  )}
                </For>
              </Show>
            </Loading>
          </div>
        </section>

        <aside class="home-panel home-panel--chat" aria-label={t("page.live_chat.title")}>
          <HomeChat />
        </aside>

        <section
          class="home-panel home-panel--notes"
          aria-labelledby="home-showcase-title"
        >
          <SectionHeader
            id="home-showcase-title"
            title={t("home.showcase.title")}
            href="/about-blog"
            linkLabel={t("home.showcase.view_all")}
          />
          <ul class="home-showcase-grid">
            <For each={SHOWCASE}>
              {(item) => (
                <li class="home-showcase-card">
                  <p class="home-showcase-kicker">{t(`home.showcase.${item.id}.kicker`)}</p>
                  <h3 class="home-showcase-title">
                    <a href={item.href} class="home-showcase-link">
                      {t(`home.showcase.${item.id}.title`)}
                      <span class="home-showcase-arrow" aria-hidden="true">
                        →
                      </span>
                    </a>
                  </h3>
                  <p class="home-showcase-text">{t(`home.showcase.${item.id}.text`)}</p>
                </li>
              )}
            </For>
          </ul>
        </section>
      </div>
    </main>
  );
}
