import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "November 2026",
  chrome: {
    overview: "Overview",
    updated: "Last updated",
    onThisPage: "On this page",
    sectionNavigation: "Tech stack pages",
    previous: "Previous",
    next: "Next",
    readingTime: "{minutes} min read",
  },
  pages: {
    overview: {
      eyebrow: "About this site",
      title: "Blog Tech Stack",
      lede: "One Rust binary, one PostgreSQL database, one small machine. These pages describe how the site is built, hosted, secured, and measured, and why it is shaped that way.",
      sections: [
        {
          id: "at-a-glance",
          heading: "At a glance",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53,889 req/s",
                  label: "Median mixed public-read throughput with PGO and BOLT on four server cores",
                },
                {
                  value: "~150 µs",
                  label: "Observed database round trip over a UNIX socket, against ~600 µs over localhost TCP",
                },
                {
                  value: "0",
                  label: "Node.js processes in production; the frontend is embedded in the server binary",
                },
                {
                  value: "8",
                  count: 8,
                  label: "Interface languages, each downloaded only when selected",
                },
                {
                  value: "4,096",
                  label: "Open connections admitted before TLS, with at most 64 per address or IPv6 /64",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Process serving TLS, the API, WebSockets, WebRTC media, and the embedded frontend",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Request path",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Browser",
                  detail: "HTTP/2 or HTTP/1.1 over TLS. The frontend is cached under fingerprinted, immutable URLs.",
                },
                {
                  label: "Admission",
                  detail: "TCP connections are admitted before TLS: 4,096 in total and 64 per address or IPv6 /64.",
                },
                {
                  label: "rustls",
                  detail: "TLS terminates inside the Rust process, with `TCP_NODELAY` on every socket.",
                },
                {
                  label: "Axum",
                  detail: "Security headers, sessions, rate limits, compression, and body limits, then a feature handler.",
                },
                {
                  label: "Cache · Diesel",
                  detail: "Bounded in-memory caches answer hot reads; everything else goes through async Diesel.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "On the same host over a UNIX socket, with io_uring asynchronous I/O.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Chapters",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Infrastructure",
                  href: "/about-blog/infrastructure",
                  meta: "Host · Gentoo · network",
                  text: "The machine, the operating system and filesystem, the network, and why there is no reverse proxy.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "The service stack, data model, runtime bounds, security headers, and API contracts.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "The application shell, loading, localization, design system, and motion.",
                },
                {
                  title: "Subsystems",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "Live chat with an in-process SFU, the Minecraft explorer, Geo-IP lookups, and WebAssembly.",
                },
                {
                  title: "Performance",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Optimized builds, how they were measured, and the results.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "How changes are checked",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy on native and WebAssembly targets as the implementation gate.",
                "Local cargo xtask checks for formatting, native unit tests, OpenAPI contract drift, and the frontend typecheck, lint, unit tests, and build.",
                "Playwright on Chromium and WebKit with axe accessibility checks and mobile reflow checks down to 320 CSS pixels.",
                "Integration and migration rollback tests against a disposable PostgreSQL instance, plus a local gitleaks secret scan.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Tech stack",
      title: "Infrastructure",
      lede: "A small machine at home, a source-based Linux distribution, and no layer that the traffic does not need.",
      sections: [
        {
          id: "host",
          heading: "The host",
          blocks: [
            {
              kind: "paragraph",
              text: "The site runs on a [miniserver](https://store.minisforum.com/products/minisforum-um690l-slim) at my residence: an eight-core Ryzen 9 mobile processor that boosts to 4.9 GHz, 32 GB of RAM, and a 1 TB NVMe SSD. At about $400 it costs less than a year of noticeably weaker cloud hardware, and it reflects my enthusiasm for self-hosting.",
            },
            {
              kind: "paragraph",
              text: "It runs the combined backend and frontend server, PostgreSQL, and a Minecraft server. If you would like to play, email me.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo and btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) is not the obvious choice for a server; at work I would pick Debian stable on ext4. Here I enjoy compiling packages myself with optimizations for the exact processor.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) brings snapshots, compression, and copy-on-write, but copy-on-write fragments database files badly. The PostgreSQL and Minecraft data directories are therefore excluded from it. A [filesystem benchmark](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) shows default btrfs trailing ext4 and XFS; I suspect disabling copy-on-write closes most of that gap, which would make an interesting measurement.",
            },
          ],
        },
        {
          id: "network",
          heading: "Network",
          blocks: [
            {
              kind: "paragraph",
              text: "Traffic arrives over a wired 1 Gbps Xfinity connection; the provider's modem lacks 2.5 Gb Ethernet, which this site will not notice. Route 53 hosts DNS.",
            },
            {
              kind: "paragraph",
              text: "Inside the machine there is no reverse proxy, container runtime, or service mesh. PostgreSQL runs on the operating system, and one Rust binary serves the frontend and the API. Plain HTTP requests are redirected to HTTPS by the same process.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "UNIX sockets",
          blocks: [
            {
              kind: "paragraph",
              text: "The server reaches PostgreSQL over a UNIX socket rather than localhost TCP, which skips the network stack entirely. [Benchmarks](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) show transaction rates nearly doubling; here I have observed round trips as low as **150 µs**, against about **600 µs** over localhost. It is old-fashioned and irrelevant to most cloud deployments, and it is faster.",
            },
            {
              kind: "paragraph",
              text: "PostgreSQL 18, released in September 2025, added [asynchronous I/O](https://pganalyze.com/blog/postgres-18-async-io), and io_uring is enabled here, which noticeably speeds up reads.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Build and deployment",
          blocks: [
            {
              kind: "list",
              items: [
                "The frontend build is embedded in the backend binary together with precompressed zstd and gzip copies, so no Node.js process runs in production.",
                "Optimized builds can be trained with profile-guided optimization and post-link optimized with BOLT; see [Performance](/about-blog/performance).",
                "Credentials, certificates, and runtime data stay out of Git, and a gitleaks scan checks the tree before changes leave the machine.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Tech stack",
      title: "Backend",
      lede: "Rust on Axum and Tokio, PostgreSQL through async Diesel, and an explicit limit on everything a client can hold open.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "Runtime", items: ["Axum", "Tokio", "hyper", "axum-server", "rustls", "mimalloc"] },
                { label: "Data", items: ["PostgreSQL 18", "Diesel", "diesel-async", "tantivy", "bitcode", "zstd"] },
                { label: "Integrations", items: ["openidconnect", "lettre", "Amazon S3", "webrtc-rs", "image + AVIF"] },
                { label: "Contracts", items: ["utoipa", "OpenAPI", "Swagger UI", "Generated TypeScript"] },
              ],
            },
            {
              kind: "paragraph",
              text: "The service is a single process with shared state: in-memory sessions, startup cache synchronization, and scheduled maintenance jobs. mimalloc is the global allocator; if a machine has real cores, allocator behavior is worth taking seriously.",
            },
          ],
        },
        {
          id: "data",
          heading: "Data model",
          blocks: [
            {
              kind: "paragraph",
              text: "In the Korean startup and corporate work I have seen, MySQL and MariaDB were treated as the only serious relational databases. PostgreSQL has long since surpassed them in the areas I care about, from native UUIDs to binary JSON.",
            },
            {
              kind: "list",
              items: [
                "**Posts** carry slugs, summaries, JSONB metadata, publication state, denormalized counters, and tag mappings. Comments are threaded through a nullable parent, and post and comment votes have separate tables.",
                "**Accounts** keep country and language, role and permission tables, and versioned profile pictures in their own table rather than on the hot user row.",
                "**Keys** are time-ordered UUIDv7 values: not enumerable like sequential IDs, and much kinder to B-tree locality than random UUIDv4.",
                "**Search** uses a disk-persisted tantivy index over post titles and tags, kept coherent with PostgreSQL, which remains authoritative.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Runtime bounds",
          blocks: [
            {
              kind: "list",
              items: [
                "Connections are admitted before TLS through a shared limiter: 4,096 open connections and 64 per IPv4 address or IPv6 /64. Refused sockets close without a handshake.",
                "HTTP/1 request heads must arrive within 20 seconds; HTTP/2 connections are pinged every 30 seconds and allow 128 concurrent streams.",
                "Each cache has a fixed bound and an authority it can read through to; none grows with keys a client controls. The WebAssembly bundle cache, for example, holds at most 256 MiB.",
                "Blocking work such as image encoding runs on bounded worker pools instead of Tokio workers.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Security headers",
          blocks: [
            {
              kind: "paragraph",
              text: "The Content-Security-Policy is built once at startup. Inline scripts are allowed only by SHA-256 hashes computed from the `index.html` compiled into the binary, so the pre-paint theme script cannot drift from the policy. The main origin never instantiates WebAssembly: demos and the EU5 application run in sandboxed frames under their own policies.",
            },
            {
              kind: "paragraph",
              text: "Session cookies are `Secure` and `HttpOnly`, superuser operations check roles on the server, accounts can link OpenID Connect identities, and the Swagger UI requires authentication in production.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Contracts and assets",
          blocks: [
            {
              kind: "list",
              items: [
                "Handlers carry OpenAPI annotations, the frontend's TypeScript clients are generated from them, and cargo xtask openapi fails when the generated files drift.",
                "Vite's manifest is embedded with the assets. Only manifest-listed, fingerprinted files receive a one-year `immutable` cache policy; HTML always revalidates.",
                "Missing asset paths return 404 with `no-store` rather than the application shell, so a stale page cannot cache HTML under a script URL.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Tech stack",
      title: "Frontend",
      lede: "A client-rendered SolidJS 2 application, compiled to static assets and embedded in the server binary.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "Application", items: ["SolidJS 2", "Solid Router 2", "TypeScript", "Vite 8"] },
                { label: "Styling", items: ["Tailwind CSS 4", "IBM Plex Sans", "IBM Plex Mono", "OKLab"] },
                { label: "Components", items: ["Toast UI Editor", "Leaflet", "Chart.js", "highlight.js", "DOMPurify"] },
                { label: "Testing", items: ["Vitest", "Playwright", "axe-core"] },
              ],
            },
            {
              kind: "paragraph",
              text: "Solid's fine-grained signals update only the DOM nodes that depend on changed values, and its 2.0 release candidates let async reads suspend to the nearest loading boundary. The router lazy-loads every page, and there is no production JavaScript server: the Rust binary hands out static files.",
            },
          ],
        },
        {
          id: "loading",
          heading: "Loading",
          blocks: [
            {
              kind: "list",
              items: [
                "Pages, the home page's chat panel, and its contour shader are separate chunks, and the build reports the gzip size of what loads first.",
                "Only English interface text ships in the initial chunk. Other languages, and each language's About pages, are separate chunks loaded on selection.",
                "Throttled mobile performance tests target LCP at or under 2.5 s, interactions at or under 200 ms, and CLS at or under 0.1.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Localization",
          blocks: [
            {
              kind: "paragraph",
              text: "The interface supports English, Korean, French, Spanish, Simplified and Traditional Chinese, Japanese, and German. A catalog of roughly 740 keys is validated against the backend's registry, and placeholders must survive translation. These About pages are typed data modules, so a translation cannot silently omit a section that English has.",
            },
          ],
        },
        {
          id: "design",
          heading: "Design system",
          blocks: [
            {
              kind: "paragraph",
              text: "The look is warm and technical: cream paper or pure black, hairline borders, monospace for anything technical, and one accent color at a time. Colors are semantic CSS variables, so components never need dark-mode variants. Accent presets are tested for contrast against the page backgrounds, a visitor's own RGB color is adjusted in each mode to stay readable, and the light or dark mode and accent are applied before first paint.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Motion",
          blocks: [
            {
              kind: "list",
              items: [
                "Section reveals and the reading-progress bar use CSS scroll-driven animations, which run off the main thread where supported.",
                "Statistics count up through a registered `@property` integer, with the final value present in the markup for assistive technology.",
                "The theme switch expands from the toggle through the View Transitions API.",
                "The home page's contour field is a small WebGL shader that loads after the page is idle, runs only on capable desktop browsers, caps its frame rate, and pauses off-screen.",
                "A site setting turns all of this off. Animations default to on and do not follow the operating system's reduced-motion setting.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Tech stack",
      title: "Subsystems",
      lede: "The parts of the site that are more interesting than a blog strictly needs.",
      sections: [
        {
          id: "live-chat",
          heading: "Live chat and calls",
          blocks: [
            {
              kind: "paragraph",
              text: "Chat runs over a WebSocket with a compact binary codec, moderation, and bounded history. Voice and video calls go through a Selective Forwarding Unit that runs inside the Axum process on webrtc-rs: each browser holds one peer connection, and the SFU forwards encoded media without transcoding, through bounded per-publication channels and a participant cap. [Open live chat](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Minecraft terrain explorer",
          blocks: [
            {
              kind: "paragraph",
              text: "A Paper plugin exposes a same-user UNIX socket for bounded world queries, with one operation at a time and strict frame-size limits. The [explorer](/minecraft) draws squaremap tiles with its own Leaflet client instead of running the generated map's JavaScript.",
            },
            {
              kind: "paragraph",
              text: "Biome overlays use **CYBM**, a compact binary tile format served gzip-compressed, or a PNG whose private `cyBM` chunk carries the exact biome identities so hovering never guesses from pixel colors. Seed-based predictions for unexplored terrain come from a separate world-generation worker built on Pumpkin.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Geo-IP lookups",
          blocks: [
            {
              kind: "paragraph",
              text: "The [visitor board](/visitor-board) and [IP lookup](/geo-ip-db) locate addresses without calling an external API. A preprocessing tool turns the IP2Location LITE CSV files into `BTreeMap`s keyed by range start, encodes them with bitcode, and compresses them with zstd at level 22.",
            },
            {
              kind: "table",
              columns: ["Dataset", "CSV", "Bundle"],
              numericColumns: [1, 2],
              rows: [
                ["IPv4", "312.5 MB", "24.9 MB"],
                ["IPv6", "794.2 MB", "42.4 MB"],
              ],
            },
            {
              kind: "paragraph",
              text: "At startup the server decodes one bundle at a time to limit peak memory and interns the repeated place names. A lookup finds the last range starting at or below the address and checks its end bound.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "Uploaded [demo bundles](/projects) are staged to disk, identified by their magic bytes, limited to 50 MiB decompressed, and served from a bounded cache inside sandboxed frames. The workspace also builds Rust demos, including a block breaker and a ray tracer.",
            },
            {
              kind: "paragraph",
              text: "The [EU5 Location DB](/eu5-locations-db) is a Slint application compiled to WebAssembly. The page sends its light or dark mode through a small message protocol, so the embedded palette changes with the site.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Photography pipeline",
          blocks: [
            {
              kind: "paragraph",
              text: "Uploads are decoded within fixed pixel bounds and encoded to AVIF at the slowest preset, trading processing time for smaller files. Locations from EXIF data can be confirmed on a map before publishing, and no more than two processing jobs run at once.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Tech stack",
      title: "Performance",
      lede: "Profile-guided optimization and BOLT on the backend, measured with a load generator that is not the bottleneck.",
      sections: [
        {
          id: "approach",
          heading: "Approach",
          blocks: [
            {
              kind: "paragraph",
              text: "Optimized builds are trained on a representative workload. An instrumented binary records profiles against a disposable fixture with synthetic PostgreSQL, SMTP, S3, OpenID Connect, and Minecraft endpoints. LLVM rebuilds the server with those profiles (PGO), and BOLT then reorders the final binary's code layout from sampled execution. All variants share LTO, frame pointers, and a rebuilt standard library.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Setup",
          blocks: [
            {
              kind: "list",
              items: [
                "Server, fixture providers, and PostgreSQL pinned to four cores of an AMD Ryzen 7 9800X3D; the load client pinned to the other four.",
                "HTTPS over pooled HTTP/1.1 with 32 connections, full response reads, and two-second deadlines.",
                "Three rotated rounds with five samples per variant, each after 10,000 warm-up requests, from a fresh database snapshot.",
                "27,000,000 timed requests in total; every one returned HTTP 200.",
                "Load comes from oha; Playwright checks each route's response outside the timed window.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Results",
          blocks: [
            {
              kind: "bars",
              caption: "Median requests per second across 15 samples per variant.",
              groups: [
                {
                  label: "Ten-route public reads",
                  items: [
                    { label: "Baseline", value: 44874.12, display: "44,874" },
                    { label: "PGO", value: 52925.46, display: "52,925" },
                    { label: "PGO + BOLT", value: 53888.61, display: "53,889", emphasis: true },
                  ],
                },
                {
                  label: "Small health JSON",
                  items: [
                    { label: "Baseline", value: 272837.41, display: "272,837" },
                    { label: "PGO", value: 308685.26, display: "308,685", emphasis: true },
                    { label: "PGO + BOLT", value: 306276.64, display: "306,277" },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "Tail latency for the ten-route mix.",
              columns: ["Variant", "p95 (ms)", "p99 (ms)"],
              numericColumns: [1, 2],
              rows: [
                ["Baseline", "2.380", "2.967"],
                ["PGO", "2.020", "2.524"],
                ["PGO + BOLT", "1.987", "2.483"],
              ],
            },
            {
              kind: "paragraph",
              text: "PGO raised mixed throughput by **17.9%** and PGO with BOLT by **20.1%**, and both optimized builds won every rotated round. BOLT added 1.8% over PGO on the mixed reads and was 0.8% slower on the trivial health route. These are four-core results; production capacity also depends on the network, the database's working set, and real traffic.",
            },
          ],
        },
        {
          id: "corrections",
          heading: "Corrections",
          blocks: [
            {
              kind: "list",
              items: [
                "`TCP_NODELAY` was not set at first: axum-server's default Rustls acceptor leaves it off, so HTTP/2 responses could wait about 40 ms on a delayed acknowledgement. Every accepted socket now sets it.",
                "The first benchmark drove load from Playwright, whose client slowed down as a run went on, and wrongly showed regressions. Those numbers were discarded.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
