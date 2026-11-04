import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "Nobyembre 2026",
  chrome: {
    overview: "Pangkalahatang-ideya",
    updated: "Huling na-update",
    onThisPage: "Sa pahinang ito",
    sectionNavigation: "Mga pahina ng tech stack",
    previous: "Nakaraan",
    next: "Susunod",
    readingTime: "{minutes} min na pagbasa",
  },
  pages: {
    overview: {
      eyebrow: "Tungkol sa site na ito",
      title: "Tech Stack ng Blog",
      lede: "Isang Rust binary, isang PostgreSQL database, isang maliit na makina. Inilalarawan ng mga pahinang ito kung paano binuo, pinapatakbo, sinisiguro, at sinusukat ang site, at kung bakit ganito ang hugis nito.",
      sections: [
        {
          id: "at-a-glance",
          heading: "Sa isang tingin",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53,889 req/s",
                  label: "Median na throughput ng pinaghalong pampublikong pagbasa gamit ang PGO at BOLT sa apat na core ng server",
                },
                {
                  value: "~150 µs",
                  label: "Naobserbahang round trip ng database sa UNIX socket, kumpara sa ~600 µs sa localhost TCP",
                },
                {
                  value: "0",
                  label: "Mga proseso ng Node.js sa production; naka-embed ang frontend sa binary ng server",
                },
                {
                  value: "24",
                  count: 24,
                  label: "Mga wika ng interface, na bawat isa ay dina-download lamang kapag pinili",
                },
                {
                  value: "4,096",
                  label: "Mga bukas na koneksyong tinatanggap bago ang TLS, hanggang 64 kada address o IPv6 /64",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Prosesong naghahatid ng TLS, ng API, ng mga WebSocket, ng WebRTC media, at ng naka-embed na frontend",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Daan ng request",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Browser",
                  detail: "HTTP/2 o HTTP/1.1 sa ibabaw ng TLS. Naka-cache ang frontend sa ilalim ng mga fingerprinted at immutable na URL.",
                },
                {
                  label: "Pagtanggap",
                  detail: "Tinatanggap ang mga koneksyong TCP bago ang TLS: 4,096 sa kabuuan at 64 kada address o IPv6 /64.",
                },
                {
                  label: "rustls",
                  detail: "Nagtatapos ang TLS sa loob ng prosesong Rust, na may `TCP_NODELAY` sa bawat socket.",
                },
                {
                  label: "Axum",
                  detail: "Mga security header, session, rate limit, compression, at limitasyon sa body, saka ang feature handler.",
                },
                {
                  label: "Cache · Diesel",
                  detail: "Sinasagot ng mga bounded na in-memory cache ang madalas na pagbasa; ang iba pa ay dumadaan sa async Diesel.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "Sa parehong host sa pamamagitan ng UNIX socket, gamit ang asynchronous I/O ng io_uring.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Mga kabanata",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Imprastraktura",
                  href: "/about-blog/infrastructure",
                  meta: "Host · Gentoo · network",
                  text: "Ang makina, ang operating system at filesystem, ang network, at kung bakit walang reverse proxy.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "Ang service stack, data model, mga hangganan sa runtime, mga security header, at mga kontrata ng API.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "Ang application shell, paglo-load, lokalisasyon, design system, at galaw.",
                },
                {
                  title: "Mga subsystem",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "Live chat na may in-process na SFU, ang Minecraft explorer, mga Geo-IP lookup, at WebAssembly.",
                },
                {
                  title: "Pagganap",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Mga optimized na build, kung paano ito sinukat, at ang mga resulta.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Paano sinusuri ang mga pagbabago",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy sa mga target na native at WebAssembly bilang gate ng implementasyon.",
                "Mga lokal na pagsusuri ng cargo xtask para sa formatting, mga native unit test, drift ng kontrata ng OpenAPI, at ang typecheck, lint, unit test, at build ng frontend.",
                "Playwright sa Chromium at WebKit na may mga pagsusuri sa accessibility ng axe at mga pagsusuri ng mobile reflow hanggang 320 CSS pixel.",
                "Mga integration test at migration rollback test laban sa disposable na instance ng PostgreSQL, kasama ang lokal na secret scan ng gitleaks.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Tech stack",
      title: "Imprastraktura",
      lede: "Isang maliit na makina sa bahay, isang Linux distribution na batay sa source, at walang layer na hindi kailangan ng trapiko.",
      sections: [
        {
          id: "host",
          heading: "Ang host",
          blocks: [
            {
              kind: "paragraph",
              text: "Tumatakbo ang site sa isang [miniserver](https://store.minisforum.com/products/minisforum-um690l-slim) sa tirahan ko: isang Ryzen 9 mobile processor na may walong core na umaabot sa 4.9 GHz, 32 GB ng RAM, at 1 TB na NVMe SSD. Sa halagang humigit-kumulang $400, mas mura ito kaysa sa isang taon ng mas mahinang cloud hardware, at ipinapakita nito ang hilig ko sa self-hosting.",
            },
            {
              kind: "paragraph",
              text: "Dito tumatakbo ang pinagsamang backend at frontend server, ang PostgreSQL, at isang Minecraft server. Kung gusto mong maglaro, i-email mo ako.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo at btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "Hindi halatang pagpipilian ang [Gentoo](https://www.gentoo.org/) para sa server; sa trabaho, Debian stable sa ext4 ang pipiliin ko. Dito, nasisiyahan akong mag-compile ng mga package nang ako mismo, na may mga optimization para sa eksaktong processor.",
            },
            {
              kind: "paragraph",
              text: "Nagdadala ang [btrfs](https://en.wikipedia.org/wiki/Btrfs) ng mga snapshot, compression, at copy-on-write, ngunit malubhang pinapa-fragment ng copy-on-write ang mga file ng database. Kaya hindi isinasama rito ang mga data directory ng PostgreSQL at Minecraft. Ipinapakita ng isang [benchmark ng filesystem](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) na nahuhuli ang default na btrfs sa ext4 at XFS; hinihinala kong halos mapupuno ng pag-disable ng copy-on-write ang agwat na iyon, na magiging kawili-wiling sukatin.",
            },
          ],
        },
        {
          id: "network",
          heading: "Network",
          blocks: [
            {
              kind: "paragraph",
              text: "Dumarating ang trapiko sa pamamagitan ng wired na 1 Gbps na koneksyon ng Xfinity; walang 2.5 Gb Ethernet ang modem ng provider, na hindi mapapansin ng site na ito. Ang Route 53 ang nagho-host ng DNS.",
            },
            {
              kind: "paragraph",
              text: "Sa loob ng makina, walang reverse proxy, container runtime, o service mesh. Tumatakbo ang PostgreSQL sa operating system, at isang Rust binary ang naghahatid ng frontend at ng API. Ang parehong proseso rin ang nagre-redirect ng mga plain HTTP request papuntang HTTPS.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "Mga UNIX socket",
          blocks: [
            {
              kind: "paragraph",
              text: "Kumokonekta ang server sa PostgreSQL sa pamamagitan ng UNIX socket sa halip na localhost TCP, na lumalaktaw sa buong network stack. Ipinapakita ng mga [benchmark](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) na halos dumodoble ang transaction rate; dito, nakaobserba ako ng mga round trip na kasing baba ng **150 µs**, kumpara sa humigit-kumulang **600 µs** sa localhost. Makaluma ito at walang kaugnayan sa karamihan ng cloud deployment, at mas mabilis ito.",
            },
            {
              kind: "paragraph",
              text: "Nagdagdag ang PostgreSQL 18, na inilabas noong Setyembre 2025, ng [asynchronous I/O](https://pganalyze.com/blog/postgres-18-async-io), at naka-enable dito ang io_uring, na kapansin-pansing nagpapabilis ng mga pagbasa.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Build at deployment",
          blocks: [
            {
              kind: "list",
              items: [
                "Naka-embed ang build ng frontend sa binary ng backend kasama ang mga precompressed na kopya sa zstd at gzip, kaya walang prosesong Node.js na tumatakbo sa production.",
                "Maaaring sanayin ang mga optimized na build gamit ang profile-guided optimization at i-post-link optimize gamit ang BOLT; tingnan ang [Pagganap](/about-blog/performance).",
                "Hindi isinasama sa Git ang mga credential, certificate, at runtime data, at sinusuri ng gitleaks scan ang tree bago umalis sa makina ang mga pagbabago.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Tech stack",
      title: "Backend",
      lede: "Rust sa Axum at Tokio, PostgreSQL sa pamamagitan ng async Diesel, at tahasang limitasyon sa lahat ng maaaring panatilihing bukas ng isang kliyente.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Runtime",
                  items: [
                    "Axum",
                    "Tokio",
                    "hyper",
                    "axum-server",
                    "rustls",
                    "mimalloc",
                  ],
                },
                {
                  label: "Datos",
                  items: [
                    "PostgreSQL 18",
                    "Diesel",
                    "diesel-async",
                    "tantivy",
                    "bitcode",
                    "zstd",
                  ],
                },
                {
                  label: "Mga integrasyon",
                  items: [
                    "openidconnect",
                    "lettre",
                    "Amazon S3",
                    "webrtc-rs",
                    "image + AVIF",
                  ],
                },
                {
                  label: "Mga kontrata",
                  items: [
                    "utoipa",
                    "OpenAPI",
                    "Swagger UI",
                    "Generated TypeScript",
                  ],
                },
              ],
            },
            {
              kind: "paragraph",
              text: "Iisang proseso ang serbisyo na may shared state: mga in-memory na session, cache synchronization sa startup, at mga nakaiskedyul na maintenance job. Ang mimalloc ang global allocator; kung may totoong mga core ang makina, dapat seryosohin ang asal ng allocator.",
            },
          ],
        },
        {
          id: "data",
          heading: "Modelo ng data",
          blocks: [
            {
              kind: "paragraph",
              text: "Sa mga startup at korporasyon sa Korea na nakita kong pinagtrabahuhan, itinuring na tanging seryosong relational database ang MySQL at MariaDB. Matagal nang nalampasan ng PostgreSQL ang mga ito sa mga larangang mahalaga sa akin, mula sa native na UUID hanggang sa binary JSON.",
            },
            {
              kind: "list",
              items: [
                "Ang **mga post** ay may slug, buod, JSONB metadata, estado ng publikasyon, denormalized na counter, at mga mapping ng tag. May thread ang mga komento sa pamamagitan ng nullable na parent, at magkahiwalay na table ang mga boto sa post at komento.",
                "Ang **mga account** ay nag-iingat ng bansa at wika, mga table ng role at permission, at mga bersyon ng larawan ng profile sa sarili nilang table sa halip na sa madalas gamiting row ng user.",
                "Ang **mga key** ay mga UUIDv7 na nakaayos ayon sa oras: hindi mabibilang tulad ng sunud-sunod na ID, at mas mabuti para sa locality ng B-tree kaysa sa random na UUIDv4.",
                "Ang **paghahanap** ay gumagamit ng tantivy index na nakaimbak sa disk para sa mga pamagat ng post at mga tag, na pinananatiling tugma sa PostgreSQL, na siya pa ring batayan.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Mga hangganan sa runtime",
          blocks: [
            {
              kind: "list",
              items: [
                "Tinatanggap ang mga koneksyon bago ang TLS sa pamamagitan ng shared na limiter: 4,096 na bukas na koneksyon at 64 kada IPv4 address o IPv6 /64. Isinasara nang walang handshake ang mga tinanggihang socket.",
                "Dapat dumating sa loob ng 20 segundo ang head ng mga HTTP/1 request; pina-ping ang mga koneksyong HTTP/2 kada 30 segundo at pinapayagan ang 128 sabay-sabay na stream.",
                "May takdang hangganan ang bawat cache at isang awtoridad na mapagbabasahan nito; walang lumalaki kasabay ng mga key na kontrolado ng kliyente. Halimbawa, hanggang 256 MiB lamang ang laman ng cache ng WebAssembly bundle.",
                "Ang mga blocking na gawain tulad ng image encoding ay tumatakbo sa mga bounded na worker pool sa halip na sa mga Tokio worker.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Mga security header",
          blocks: [
            {
              kind: "paragraph",
              text: "Binubuo nang minsan lamang sa startup ang Content-Security-Policy. Pinapayagan lamang ang mga inline script sa pamamagitan ng mga SHA-256 hash na na-compute mula sa `index.html` na na-compile sa binary, kaya hindi maaaring lumihis sa polisiya ang theme script bago ang paint. Hindi kailanman nag-iinstantiate ng WebAssembly ang pangunahing origin: tumatakbo ang mga demo at ang EU5 application sa mga sandboxed na frame sa ilalim ng sarili nilang polisiya.",
            },
            {
              kind: "paragraph",
              text: "Ang mga session cookie ay `Secure` at `HttpOnly`, sinusuri ng server ang mga role para sa mga operasyon ng superuser, maaaring i-link ng mga account ang mga identidad ng OpenID Connect, at nangangailangan ng authentication ang Swagger UI sa production.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Mga kontrata at asset",
          blocks: [
            {
              kind: "list",
              items: [
                "May mga anotasyon ng OpenAPI ang mga handler, nabubuo mula sa mga ito ang mga TypeScript client ng frontend, at pumapalya ang cargo xtask openapi kapag lumihis ang mga nabuong file.",
                "Naka-embed kasama ng mga asset ang manifest ng Vite. Tanging mga fingerprinted na file na nakalista sa manifest ang tumatanggap ng isang-taong `immutable` na cache policy; laging nagre-revalidate ang HTML.",
                "Ang mga nawawalang landas ng asset ay nagbabalik ng 404 na may `no-store` sa halip na ang application shell, kaya hindi makakapag-cache ng HTML sa ilalim ng URL ng script ang lumang pahina.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Tech stack",
      title: "Frontend",
      lede: "Isang SolidJS 2 application na nagre-render sa client, kino-compile bilang mga static asset, at naka-embed sa binary ng server.",
      sections: [
        {
          id: "stack",
          heading: "Mga teknolohiya",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Aplikasyon",
                  items: [
                    "SolidJS 2",
                    "Solid Router 2",
                    "TypeScript",
                    "Vite 8",
                  ],
                },
                {
                  label: "Estilo",
                  items: [
                    "Tailwind CSS 4",
                    "IBM Plex Sans",
                    "IBM Plex Mono",
                    "OKLab",
                  ],
                },
                {
                  label: "Mga component",
                  items: [
                    "Toast UI Editor",
                    "Leaflet",
                    "Chart.js",
                    "highlight.js",
                    "DOMPurify",
                  ],
                },
                {
                  label: "Pagsubok",
                  items: [
                    "Vitest",
                    "Playwright",
                    "axe-core",
                  ],
                },
              ],
            },
            {
              kind: "paragraph",
              text: "Ina-update lang ng fine-grained signals ng Solid ang mga DOM node na umaasa sa nagbagong value, at sa mga 2.0 release candidate nito, maaaring mag-suspend ang mga async na pagbasa hanggang sa pinakamalapit na loading boundary. Lazy-load ng router ang bawat pahina, at walang JavaScript server sa production: ang Rust binary mismo ang naghahatid ng mga static file.",
            },
          ],
        },
        {
          id: "loading",
          heading: "Pag-load",
          blocks: [
            {
              kind: "list",
              items: [
                "Magkakahiwalay na chunk ang mga pahina, ang chat panel ng home page, at ang contour shader nito, at iniuulat ng build ang gzip size ng unang nilo-load.",
                "Tekstong pang-interface sa Ingles lang ang kasama sa unang chunk. Ang ibang wika, at ang mga About page ng bawat wika, ay magkakahiwalay na chunk na nilo-load kapag pinili.",
                "Ang mga throttled na pagsubok sa performance ng mobile ay may target na LCP na 2.5 s pababa, mga interaksyon na 200 ms pababa, at CLS na 0.1 pababa.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Lokalisasyon",
          blocks: [
            {
              kind: "paragraph",
              text: "Sinusuportahan ng interface ang 24 na wika, kasama ang mga right-to-left na layout para sa Arabic, Persian, at Urdu. Isang catalog na may humigit-kumulang 740 key ang bina-validate laban sa registry ng backend, at dapat manatili ang mga placeholder sa pagsasalin. Ang mga About page na ito ay mga typed data module, kaya hindi basta makakalaktaw ang isang salin ng seksyong mayroon sa Ingles.",
            },
          ],
        },
        {
          id: "design",
          heading: "Sistema ng disenyo",
          blocks: [
            {
              kind: "paragraph",
              text: "Mainit at teknikal ang itsura: cream na papel o purong itim, manipis na border, monospace para sa anumang teknikal, at isang accent color lang sa bawat pagkakataon. Semantic na CSS variable ang mga kulay, kaya hindi na kailangan ng mga component ng dark-mode variant. Sinusubok ang contrast ng mga accent preset laban sa mga background ng pahina, ina-adjust sa bawat mode ang sariling RGB color ng bisita para manatiling nababasa, at inilalapat ang light o dark mode at ang accent bago ang unang paint.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Galaw",
          blocks: [
            {
              kind: "list",
              items: [
                "Gumagamit ng mga CSS scroll-driven animation ang mga section reveal at ang reading-progress bar, na tumatakbo sa labas ng main thread kung saan suportado.",
                "Bumibilang pataas ang mga istatistika gamit ang rehistradong integer na `@property`, at nasa markup na ang huling value para sa assistive technology.",
                "Lumalawak ang theme switch mula sa toggle gamit ang View Transitions API.",
                "Isang maliit na WebGL shader ang contour field ng home page; nilo-load ito kapag idle na ang pahina, tumatakbo lang sa mga desktop browser na may kakayahan, nililimitahan ang frame rate nito, at humihinto kapag wala sa screen.",
                "Isang setting ng site ang nagpapatay sa lahat ng ito. Naka-on bilang default ang mga animation at hindi sinusunod ang reduced-motion setting ng operating system.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Tech stack",
      title: "Mga subsystem",
      lede: "Ang mga bahagi ng site na mas kawili-wili kaysa sa kailangan ng isang blog.",
      sections: [
        {
          id: "live-chat",
          heading: "Live chat at mga tawag",
          blocks: [
            {
              kind: "paragraph",
              text: "Tumatakbo ang chat sa WebSocket na may compact na binary codec, moderation, at limitadong history. Dumadaan ang mga voice at video call sa isang Selective Forwarding Unit na tumatakbo sa loob ng proseso ng Axum sa webrtc-rs: may isang peer connection ang bawat browser, at ipinapasa ng SFU ang naka-encode na media nang walang transcoding, sa pamamagitan ng limitadong channel kada publication at limitasyon sa bilang ng kalahok. [Buksan ang live chat](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Terrain explorer ng Minecraft",
          blocks: [
            {
              kind: "paragraph",
              text: "Nagbubukas ang isang Paper plugin ng UNIX socket para sa parehong user na ginagamit sa mga limitadong query sa mundo, isang operasyon lang sa bawat pagkakataon at may mahigpit na limitasyon sa laki ng frame. Iginuguhit ng [explorer](/minecraft) ang mga squaremap tile gamit ang sarili nitong Leaflet client sa halip na patakbuhin ang JavaScript ng nabuong mapa.",
            },
            {
              kind: "paragraph",
              text: "Gumagamit ang mga biome overlay ng **CYBM**, isang compact na binary tile format na inihahatid na naka-gzip, o ng PNG na ang pribadong `cyBM` chunk ang may dala ng eksaktong pagkakakilanlan ng mga biome, kaya hindi kailanman hinuhulaan mula sa kulay ng pixel ang lumalabas sa hover. Ang mga prediksyon batay sa seed para sa mga lugar na hindi pa napupuntahan ay galing sa hiwalay na worker ng world-generation na binuo sa Pumpkin.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Mga lookup ng Geo-IP",
          blocks: [
            {
              kind: "paragraph",
              text: "Hinahanap ng [visitor board](/visitor-board) at [IP lookup](/geo-ip-db) ang lokasyon ng mga address nang hindi tumatawag sa external na API. Ginagawang mga `BTreeMap` ng isang preprocessing tool ang mga CSV file ng IP2Location LITE, na naka-key sa simula ng range, ine-encode gamit ang bitcode, at kino-compress gamit ang zstd sa level 22.",
            },
            {
              kind: "table",
              columns: [
                "Set ng data",
                "CSV",
                "Bundle",
              ],
              numericColumns: [
                1,
                2,
              ],
              rows: [
                [
                  "IPv4",
                  "312.5 MB",
                  "24.9 MB",
                ],
                [
                  "IPv6",
                  "794.2 MB",
                  "42.4 MB",
                ],
              ],
            },
            {
              kind: "paragraph",
              text: "Sa startup, isa-isang dine-decode ng server ang mga bundle upang limitahan ang peak na memory, at ini-intern ang mga paulit-ulit na pangalan ng lugar. Hinahanap ng lookup ang huling range na nagsisimula sa o mas mababa sa address at sinusuri ang end bound nito.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "Ang mga na-upload na [demo bundle](/projects) ay ini-stage sa disk, kinikilala sa pamamagitan ng kanilang magic bytes, nililimitahan sa 50 MiB kapag na-decompress, at inihahatid mula sa limitadong cache sa loob ng mga sandboxed na frame. Binubuo rin ng workspace ang mga Rust demo, kasama ang isang block breaker at isang ray tracer.",
            },
            {
              kind: "paragraph",
              text: "Ang [EU5 Location DB](/eu5-locations-db) ay isang Slint application na kino-compile sa WebAssembly. Ipinapadala ng pahina ang light o dark mode nito sa pamamagitan ng maliit na message protocol, kaya nagbabago ang naka-embed na palette kasabay ng site.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Pipeline ng litrato",
          blocks: [
            {
              kind: "paragraph",
              text: "Dine-decode ang mga upload sa loob ng takdang hangganan ng pixel at ine-encode sa AVIF sa pinakamabagal na preset, na ipinagpapalit ang oras ng pagproseso para sa mas maliliit na file. Maaaring kumpirmahin sa mapa ang mga lokasyon mula sa EXIF data bago mag-publish, at hindi hihigit sa dalawang processing job ang sabay na tumatakbo.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Tech stack",
      title: "Pagganap",
      lede: "Profile-guided optimization at BOLT sa backend, na sinusukat gamit ang load generator na hindi nagiging bottleneck.",
      sections: [
        {
          id: "approach",
          heading: "Paraan",
          blocks: [
            {
              kind: "paragraph",
              text: "Ang mga optimized na build ay sinasanay sa isang kinatawang workload. Ang instrumented na binary ay nagtatala ng mga profile laban sa isang disposable na fixture na may synthetic na PostgreSQL, SMTP, S3, OpenID Connect, at Minecraft endpoint. Muling binubuo ng LLVM ang server gamit ang mga profile na iyon (PGO), at pagkatapos ay inaayos ng BOLT ang code layout ng huling binary mula sa sampled na execution. Lahat ng variant ay may LTO, frame pointer, at muling binuong standard library.",
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
                "Ang server, mga fixture provider, at PostgreSQL ay naka-pin sa apat na core ng AMD Ryzen 7 9800X3D; ang load client ay naka-pin sa natitirang apat.",
                "HTTPS sa pooled na HTTP/1.1 na may 32 koneksyon, buong pagbasa ng response, at dalawang-segundong deadline.",
                "Tatlong paikot na round na may limang sample bawat variant, bawat isa pagkatapos ng 10,000 warm-up na request, mula sa bagong snapshot ng database.",
                "27,000,000 timed na request sa kabuuan; bawat isa ay nagbalik ng HTTP 200.",
                "Ang load ay galing sa oha; sinusuri ng Playwright ang response ng bawat ruta sa labas ng timed na window.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Mga resulta",
          blocks: [
            {
              kind: "bars",
              caption: "Median ng mga request kada segundo sa 15 sample bawat variant.",
              groups: [
                {
                  label: "Pampublikong pagbasa sa sampung ruta",
                  items: [
                    {
                      label: "Batayan",
                      value: 44874.12,
                      display: "44,874",
                    },
                    {
                      label: "PGO",
                      value: 52925.46,
                      display: "52,925",
                    },
                    {
                      label: "PGO + BOLT",
                      value: 53888.61,
                      display: "53,889",
                      emphasis: true,
                    },
                  ],
                },
                {
                  label: "Maliit na health JSON",
                  items: [
                    {
                      label: "Batayan",
                      value: 272837.41,
                      display: "272,837",
                    },
                    {
                      label: "PGO",
                      value: 308685.26,
                      display: "308,685",
                      emphasis: true,
                    },
                    {
                      label: "PGO + BOLT",
                      value: 306276.64,
                      display: "306,277",
                    },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "Tail latency para sa halo ng sampung ruta.",
              columns: [
                "Variant",
                "p95 (ms)",
                "p99 (ms)",
              ],
              numericColumns: [
                1,
                2,
              ],
              rows: [
                [
                  "Batayan",
                  "2.380",
                  "2.967",
                ],
                [
                  "PGO",
                  "2.020",
                  "2.524",
                ],
                [
                  "PGO + BOLT",
                  "1.987",
                  "2.483",
                ],
              ],
            },
            {
              kind: "paragraph",
              text: "Itinaas ng PGO ang pinaghalong throughput ng **17.9%** at ng PGO na may BOLT ng **20.1%**, at nanguna ang dalawang optimized na build sa bawat paikot na round. Nadagdagan ng BOLT ng 1.8% ang PGO sa pinaghalong pagbasa, ngunit 0.8% na mas mabagal ito sa simpleng health route. Ito ay mga resulta sa apat na core; ang kapasidad sa production ay nakadepende rin sa network, sa working set ng database, at sa totoong trapiko.",
            },
          ],
        },
        {
          id: "corrections",
          heading: "Mga pagwawasto",
          blocks: [
            {
              kind: "list",
              items: [
                "Hindi nakatakda ang `TCP_NODELAY` noong una: iniiwang naka-off ito ng default na Rustls acceptor ng axum-server, kaya ang mga HTTP/2 response ay maaaring maghintay ng mga 40 ms sa naantalang acknowledgement. Ngayon, itinatakda ito sa bawat tinanggap na socket.",
                "Ang unang benchmark ay nagpadala ng load mula sa Playwright, na ang client ay bumabagal habang tumatagal ang run, at maling nagpakita ng mga regression. Itinapon ang mga numerong iyon.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
