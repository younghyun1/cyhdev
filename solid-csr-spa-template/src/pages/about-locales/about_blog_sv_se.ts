import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "november 2026",
  chrome: {
    overview: "Översikt",
    updated: "Senast uppdaterad",
    onThisPage: "På den här sidan",
    sectionNavigation: "Sidor om teknikstacken",
    previous: "Föregående",
    next: "Nästa",
    readingTime: "{minutes} min läsning",
  },
  pages: {
    overview: {
      eyebrow: "Om den här webbplatsen",
      title: "Bloggens teknikstack",
      lede: "En Rust-binär, en PostgreSQL-databas, en liten maskin. De här sidorna beskriver hur webbplatsen byggs, drivs, skyddas och mäts, och varför den ser ut som den gör.",
      sections: [
        {
          id: "at-a-glance",
          heading: "I korthet",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53 889 req/s",
                  label: "Mediangenomströmning för blandade publika läsningar med PGO och BOLT på fyra serverkärnor",
                },
                {
                  value: "~150 µs",
                  label: "Observerad rundtur mot databasen över en UNIX-socket, mot cirka 600 µs över TCP på localhost",
                },
                {
                  value: "0",
                  label: "Node.js-processer i produktion; frontend är inbäddad i serverbinären",
                },
                {
                  value: "24",
                  count: 24,
                  label: "Gränssnittsspråk, vart och ett hämtas först när det väljs",
                },
                {
                  value: "4 096",
                  label: "Öppna anslutningar som släpps in före TLS, högst 64 per adress eller IPv6 /64",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Process som hanterar TLS, API:et, WebSockets, WebRTC-media och den inbäddade frontenden",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Förfrågningens väg",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Webbläsare",
                  detail: "HTTP/2 eller HTTP/1.1 över TLS. Frontenden cachas under oföränderliga webbadresser med fingeravtryck.",
                },
                {
                  label: "Insläpp",
                  detail: "TCP-anslutningar släpps in före TLS: 4 096 totalt och 64 per adress eller IPv6 /64.",
                },
                {
                  label: "rustls",
                  detail: "TLS avslutas inuti Rust-processen, med `TCP_NODELAY` på varje socket.",
                },
                {
                  label: "Axum",
                  detail: "Säkerhetsheaders, sessioner, hastighetsbegränsning, komprimering och storleksgränser, därefter en funktionshanterare.",
                },
                {
                  label: "Cache · Diesel",
                  detail: "Begränsade cacher i minnet besvarar täta läsningar; allt annat går genom asynkron Diesel.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "På samma värd över en UNIX-socket, med asynkron I/O via io_uring.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Kapitel",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Infrastruktur",
                  href: "/about-blog/infrastructure",
                  meta: "Värd · Gentoo · nätverk",
                  text: "Maskinen, operativsystemet och filsystemet, nätverket och varför det inte finns någon omvänd proxy.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "Tjänstestacken, datamodellen, körtidsgränserna, säkerhetsheaderna och API-kontrakten.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "Applikationsskalet, inläsning, lokalisering, designsystemet och rörelse.",
                },
                {
                  title: "Delsystem",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "Livechatt med en SFU i samma process, Minecraft-utforskaren, Geo-IP-uppslag och WebAssembly.",
                },
                {
                  title: "Prestanda",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Optimerade byggen, hur de mättes och resultaten.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Hur ändringar kontrolleras",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy på både native- och WebAssembly-mål som grind under implementeringen.",
                "Lokala cargo xtask-kontroller för formatering, native enhetstester, avvikelser mot OpenAPI-kontraktet samt frontendens typkontroll, lint, enhetstester och bygge.",
                "Playwright på Chromium och WebKit med axe-tillgänglighetskontroller och kontroller av mobil omflödning ner till 320 CSS-pixlar.",
                "Integrationstester och tester av återställning av migreringar mot en tillfällig PostgreSQL-instans, samt en lokal hemlighetsskanning med gitleaks.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Teknikstack",
      title: "Infrastruktur",
      lede: "En liten maskin hemma, en källkodsbaserad Linuxdistribution och inget lager som trafiken inte behöver.",
      sections: [
        {
          id: "host",
          heading: "Värden",
          blocks: [
            {
              kind: "paragraph",
              text: "Webbplatsen körs på en [miniserver](https://store.minisforum.com/products/minisforum-um690l-slim) hemma hos mig: en mobil Ryzen 9-processor med åtta kärnor som boostar till 4,9 GHz, 32 GB RAM och en NVMe-SSD på 1 TB. För cirka 400 dollar kostar den mindre än ett år av märkbart svagare molnhårdvara, och den speglar mitt intresse för att drifta själv.",
            },
            {
              kind: "paragraph",
              text: "Den kör den sammanslagna backend- och frontendservern, PostgreSQL och en Minecraft-server. Vill du spela, mejla mig.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo och btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) är inte det givna valet för en server; på jobbet skulle jag välja Debian stable på ext4. Här tycker jag om att kompilera paketen själv, optimerade för just den här processorn.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) ger ögonblicksbilder, komprimering och copy-on-write, men copy-on-write fragmenterar databasfiler illa. Därför är datakatalogerna för PostgreSQL och Minecraft undantagna. En [filsystemsjämförelse](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) visar att btrfs med standardinställningar ligger efter ext4 och XFS; jag misstänker att avstängd copy-on-write stänger det mesta av glappet, vilket vore en intressant mätning.",
            },
          ],
        },
        {
          id: "network",
          heading: "Nätverk",
          blocks: [
            {
              kind: "paragraph",
              text: "Trafiken kommer in över en trådbunden Xfinity-anslutning på 1 Gbps; leverantörens modem saknar Ethernet på 2,5 Gb, vilket den här webbplatsen inte märker av. Route 53 sköter DNS.",
            },
            {
              kind: "paragraph",
              text: "Inuti maskinen finns ingen omvänd proxy, ingen containerruntime och inget tjänstenät. PostgreSQL körs direkt på operativsystemet, och en enda Rust-binär serverar frontenden och API:et. Vanliga HTTP-förfrågningar omdirigeras till HTTPS av samma process.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "UNIX-sockets",
          blocks: [
            {
              kind: "paragraph",
              text: "Servern når PostgreSQL över en UNIX-socket i stället för TCP på localhost, vilket helt hoppar över nätverksstacken. [Benchmarks](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) visar att transaktionstakten nästan fördubblas; här har jag sett rundturer så låga som **150 µs**, mot cirka **600 µs** över localhost. Det är gammalmodigt och irrelevant för de flesta molnmiljöer, och det är snabbare.",
            },
            {
              kind: "paragraph",
              text: "PostgreSQL 18, som släpptes i september 2025, fick [asynkron I/O](https://pganalyze.com/blog/postgres-18-async-io), och io_uring är aktiverat här, vilket märkbart snabbar upp läsningar.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Bygge och driftsättning",
          blocks: [
            {
              kind: "list",
              items: [
                "Frontendbygget är inbäddat i backendbinären tillsammans med förkomprimerade kopior i zstd och gzip, så ingen Node.js-process körs i produktion.",
                "Optimerade byggen kan tränas med profilstyrd optimering och efterlänkoptimeras med BOLT; se [Prestanda](/about-blog/performance).",
                "Inloggningsuppgifter, certifikat och körtidsdata hålls utanför Git, och en gitleaks-skanning kontrollerar trädet innan ändringar lämnar maskinen.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Teknikstack",
      title: "Backend",
      lede: "Rust på Axum och Tokio, PostgreSQL via asynkron Diesel och en uttrycklig gräns för allt en klient kan hålla öppet.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Körtid",
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
                  label: "Data",
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
                  label: "Integrationer",
                  items: [
                    "openidconnect",
                    "lettre",
                    "Amazon S3",
                    "webrtc-rs",
                    "image + AVIF",
                  ],
                },
                {
                  label: "Kontrakt",
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
              text: "Tjänsten är en enda process med delat tillstånd: sessioner i minnet, cachesynkronisering vid start och schemalagda underhållsjobb. mimalloc är den globala allokeraren; när en maskin har riktiga kärnor är det värt att ta allokerarens beteende på allvar.",
            },
          ],
        },
        {
          id: "data",
          heading: "Datamodell",
          blocks: [
            {
              kind: "paragraph",
              text: "I de koreanska startup- och företagsmiljöer jag har sett betraktades MySQL och MariaDB som de enda seriösa relationsdatabaserna. PostgreSQL har sedan länge gått förbi dem på de områden jag bryr mig om, från inbyggda UUID till binär JSON.",
            },
            {
              kind: "list",
              items: [
                "**Inlägg** har slugs, sammanfattningar, JSONB-metadata, publiceringsstatus, denormaliserade räknare och taggkopplingar. Kommentarer är trådade via en nullbar förälder, och röster på inlägg och kommentarer ligger i separata tabeller.",
                "**Konton** lagrar land och språk, roll- och behörighetstabeller samt versionshanterade profilbilder i en egen tabell i stället för på den hårt belastade användarraden.",
                "**Nycklar** är tidsordnade UUIDv7-värden: de går inte att räkna upp som löpande ID:n och är mycket snällare mot B-trädens lokalitet än slumpmässiga UUIDv4.",
                "**Sökning** använder ett diskbaserat tantivy-index över inläggens titlar och taggar, som hålls i takt med PostgreSQL, vilken förblir den auktoritativa källan.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Körtidsgränser",
          blocks: [
            {
              kind: "list",
              items: [
                "Anslutningar släpps in före TLS genom en delad begränsare: 4 096 öppna anslutningar och 64 per IPv4-adress eller IPv6 /64. Nekade sockets stängs utan handskakning.",
                "Huvudet i HTTP/1-förfrågningar måste ha kommit fram inom 20 sekunder; HTTP/2-anslutningar pingas var 30:e sekund och tillåter 128 samtidiga strömmar.",
                "Varje cache har en fast gräns och en källa den kan läsa igenom till; ingen växer med nycklar som en klient styr. WebAssembly-paketcachen rymmer till exempel högst 256 MiB.",
                "Blockerande arbete, som bildkodning, körs på begränsade trådpooler i stället för på Tokio-trådarna.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Säkerhetsheaders",
          blocks: [
            {
              kind: "paragraph",
              text: "Content-Security-Policy byggs en gång vid start. Inbäddade skript tillåts bara via SHA-256-hashar som beräknas från den `index.html` som är kompilerad i binären, så temaskriptet som körs före första målningen inte kan avvika från policyn. Huvudoriginen instansierar aldrig WebAssembly: demorna och EU5-applikationen körs i sandlådade ramar under egna policyer.",
            },
            {
              kind: "paragraph",
              text: "Sessionscookies är `Secure` och `HttpOnly`, superanvändaråtgärder kontrollerar roller på servern, konton kan länka OpenID Connect-identiteter och Swagger UI kräver autentisering i produktion.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Kontrakt och tillgångar",
          blocks: [
            {
              kind: "list",
              items: [
                "Hanterarna har OpenAPI-annotationer, frontendens TypeScript-klienter genereras från dem, och cargo xtask openapi misslyckas när de genererade filerna avviker.",
                "Vites manifest är inbäddat tillsammans med tillgångarna. Bara filer med fingeravtryck som finns i manifestet får en `immutable`-cachepolicy på ett år; HTML omvalideras alltid.",
                "Saknade tillgångssökvägar ger 404 med `no-store` i stället för applikationsskalet, så att en föråldrad sida inte kan cacha HTML under en skript-URL.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Teknikstack",
      title: "Frontend",
      lede: "En klientrenderad SolidJS 2-applikation som kompileras till statiska filer och bäddas in i serverns binärfil.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Applikation",
                  items: [
                    "SolidJS 2",
                    "Solid Router 2",
                    "TypeScript",
                    "Vite 8",
                  ],
                },
                {
                  label: "Stilsättning",
                  items: [
                    "Tailwind CSS 4",
                    "IBM Plex Sans",
                    "IBM Plex Mono",
                    "OKLab",
                  ],
                },
                {
                  label: "Komponenter",
                  items: [
                    "Toast UI Editor",
                    "Leaflet",
                    "Chart.js",
                    "highlight.js",
                    "DOMPurify",
                  ],
                },
                {
                  label: "Testning",
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
              text: "Solids finkorniga signaler uppdaterar bara de DOM-noder som beror på ändrade värden, och i releasekandidaterna för 2.0 kan asynkrona läsningar pausas till närmaste laddningsgräns. Routern lazy-laddar varje sida, och det finns ingen JavaScript-server i produktion: Rust-binären levererar statiska filer.",
            },
          ],
        },
        {
          id: "loading",
          heading: "Laddning",
          blocks: [
            {
              kind: "list",
              items: [
                "Sidorna, chattpanelen på startsidan och dess konturshader är separata chunkar, och bygget rapporterar gzip-storleken på det som laddas först.",
                "Bara engelsk gränssnittstext följer med i den första chunken. Övriga språk, och varje språks Om-sidor, är separata chunkar som laddas när språket väljs.",
                "Prestandatester med strypt mobilanslutning siktar på LCP på högst 2,5 s, interaktioner på högst 200 ms och CLS på högst 0,1.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Lokalisering",
          blocks: [
            {
              kind: "paragraph",
              text: "Gränssnittet stöder 24 språk, inklusive höger-till-vänster-layout för arabiska, persiska och urdu. En katalog på ungefär 740 nycklar valideras mot backendens register, och platshållare måste överleva översättningen. De här Om-sidorna är typade datamoduler, så en översättning inte kan utelämna ett avsnitt som finns på engelska utan att det märks.",
            },
          ],
        },
        {
          id: "design",
          heading: "Designsystem",
          blocks: [
            {
              kind: "paragraph",
              text: "Utseendet är varmt och tekniskt: gräddfärgat papper eller rent svart, hårfina ramar, monospace för allt tekniskt och en accentfärg i taget. Färgerna är semantiska CSS-variabler, så komponenterna behöver aldrig varianter för mörkt läge. Accentförinställningarna testas för kontrast mot sidornas bakgrunder, en besökares egen RGB-färg justeras i varje läge för att förbli läsbar, och ljust eller mörkt läge samt accent tillämpas före den första ritningen.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Rörelse",
          blocks: [
            {
              kind: "list",
              items: [
                "Avsnitt som tonas in och läsförloppsfältet använder CSS-animationer styrda av scrollning, som körs utanför huvudtråden där det stöds.",
                "Statistik räknas upp genom ett registrerat heltal av typen `@property`, och slutvärdet finns i markupen för hjälpmedel.",
                "Temaväxlingen breder ut sig från knappen med hjälp av View Transitions API.",
                "Startsidans konturfält är en liten WebGL-shader som laddas när sidan är inaktiv, körs bara i tillräckligt kapabla webbläsare på dator, begränsar sin bildfrekvens och pausas när den är utanför skärmen.",
                "En webbplatsinställning stänger av allt detta. Animationer är på som standard och följer inte operativsystemets inställning för minskad rörelse.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Teknikstack",
      title: "Delsystem",
      lede: "De delar av webbplatsen som är mer intressanta än en blogg egentligen kräver.",
      sections: [
        {
          id: "live-chat",
          heading: "Livechatt och samtal",
          blocks: [
            {
              kind: "paragraph",
              text: "Chatten går över en WebSocket med en kompakt binär kodek, moderering och begränsad historik. Röst- och videosamtal går via en Selective Forwarding Unit som körs i Axum-processen på webrtc-rs: varje webbläsare har en peer-anslutning, och SFU:n vidarebefordrar kodad media utan omkodning, genom begränsade kanaler per publicering och ett tak för antalet deltagare. [Öppna livechatten](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Utforskare för Minecraft-terräng",
          blocks: [
            {
              kind: "paragraph",
              text: "Ett Paper-plugin exponerar en UNIX-socket för samma användare för begränsade frågor mot världen, med en operation i taget och strikta gränser för ramstorlek. [Utforskaren](/minecraft) ritar squaremap-brickor med en egen Leaflet-klient i stället för att köra den genererade kartans JavaScript.",
            },
            {
              kind: "paragraph",
              text: "Biomöverlägg använder **CYBM**, ett kompakt binärt brickformat som levereras gzip-komprimerat, eller en PNG vars privata `cyBM`-chunk bär de exakta biomidentiteterna, så att hovring aldrig gissar utifrån pixelfärger. Frönbaserade förutsägelser för outforskad terräng kommer från en separat worker för världsgenerering, byggd på Pumpkin.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Geo-IP-uppslag",
          blocks: [
            {
              kind: "paragraph",
              text: "[Besökstavlan](/visitor-board) och [IP-uppslaget](/geo-ip-db) lokaliserar adresser utan att anropa något externt API. Ett förbehandlingsverktyg gör om CSV-filerna från IP2Location LITE till `BTreeMap`s indexerade på intervallets start, kodar dem med bitcode och komprimerar dem med zstd på nivå 22.",
            },
            {
              kind: "table",
              columns: [
                "Dataset",
                "CSV",
                "Paket",
              ],
              numericColumns: [
                1,
                2,
              ],
              rows: [
                [
                  "IPv4",
                  "312,5 MB",
                  "24,9 MB",
                ],
                [
                  "IPv6",
                  "794,2 MB",
                  "42,4 MB",
                ],
              ],
            },
            {
              kind: "paragraph",
              text: "Vid start avkodar servern ett paket i taget för att begränsa toppminnet och internerar de upprepade ortnamnen. Ett uppslag hittar det sista intervallet som börjar vid eller under adressen och kontrollerar dess övre gräns.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "Uppladdade [demopaket](/projects) mellanlagras på disk, identifieras med sina magiska bytes, begränsas till 50 MiB uppackade och serveras från en begränsad cache i sandlådade ramar. Arbetsytan bygger också Rust-demor, bland annat ett block breaker-spel och en ray tracer.",
            },
            {
              kind: "paragraph",
              text: "[EU5 Location DB](/eu5-locations-db) är en Slint-applikation kompilerad till WebAssembly. Sidan skickar sitt ljusa eller mörka läge genom ett litet meddelandeprotokoll, så att den inbäddade paletten ändras tillsammans med webbplatsen.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Fotopipeline",
          blocks: [
            {
              kind: "paragraph",
              text: "Uppladdningar avkodas inom fasta pixelgränser och kodas till AVIF med den långsammaste förinställningen, vilket byter bearbetningstid mot mindre filer. Platser från EXIF-data kan bekräftas på en karta före publicering, och högst två bearbetningsjobb körs samtidigt.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Teknikstack",
      title: "Prestanda",
      lede: "Profilstyrd optimering och BOLT i backend, mätt med en lastgenerator som inte är flaskhalsen.",
      sections: [
        {
          id: "approach",
          heading: "Tillvägagångssätt",
          blocks: [
            {
              kind: "paragraph",
              text: "Optimerade byggen tränas på en representativ arbetsbelastning. En instrumenterad binär spelar in profiler mot en engångsmiljö med syntetiska ändpunkter för PostgreSQL, SMTP, S3, OpenID Connect och Minecraft. LLVM bygger om servern med dessa profiler (PGO), och BOLT ordnar sedan om kodlayouten i den färdiga binären utifrån samplad körning. Alla varianter delar LTO, frame pointers och ett ombyggt standardbibliotek.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Testuppsättning",
          blocks: [
            {
              kind: "list",
              items: [
                "Server, testmiljöns tjänster och PostgreSQL är låsta till fyra kärnor på en AMD Ryzen 7 9800X3D; lastklienten är låst till de övriga fyra.",
                "HTTPS över poolad HTTP/1.1 med 32 anslutningar, fullständig läsning av svaren och tidsgränser på två sekunder.",
                "Tre roterade omgångar med fem mätningar per variant, var och en efter 10 000 uppvärmningsanrop, från en ny databasögonblicksbild.",
                "27 000 000 tidtagna anrop totalt; alla returnerade HTTP 200.",
                "Lasten kommer från oha; Playwright kontrollerar varje rutts svar utanför det tidtagna fönstret.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Resultat",
          blocks: [
            {
              kind: "bars",
              caption: "Median för antal anrop per sekund över 15 mätningar per variant.",
              groups: [
                {
                  label: "Offentliga läsningar över tio rutter",
                  items: [
                    {
                      label: "Utgångsläge",
                      value: 44874.12,
                      display: "44 874",
                    },
                    {
                      label: "PGO",
                      value: 52925.46,
                      display: "52 925",
                    },
                    {
                      label: "PGO + BOLT",
                      value: 53888.61,
                      display: "53 889",
                      emphasis: true,
                    },
                  ],
                },
                {
                  label: "Liten JSON för hälsokontroll",
                  items: [
                    {
                      label: "Utgångsläge",
                      value: 272837.41,
                      display: "272 837",
                    },
                    {
                      label: "PGO",
                      value: 308685.26,
                      display: "308 685",
                      emphasis: true,
                    },
                    {
                      label: "PGO + BOLT",
                      value: 306276.64,
                      display: "306 277",
                    },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "Svanslatens för den blandade trafiken över tio rutter.",
              columns: [
                "Byggvariant",
                "p95 (ms)",
                "p99 (ms)",
              ],
              numericColumns: [
                1,
                2,
              ],
              rows: [
                [
                  "Utgångsläge",
                  "2,380",
                  "2,967",
                ],
                [
                  "PGO",
                  "2,020",
                  "2,524",
                ],
                [
                  "PGO + BOLT",
                  "1,987",
                  "2,483",
                ],
              ],
            },
            {
              kind: "paragraph",
              text: "PGO höjde den blandade genomströmningen med **17,9 %** och PGO med BOLT med **20,1 %**, och båda optimerade byggena vann varje roterad omgång. BOLT gav 1,8 % över PGO på de blandade läsningarna och var 0,8 % långsammare på den triviala hälsorutten. Det här är resultat för fyra kärnor; produktionskapaciteten beror också på nätverket, databasens arbetsmängd och verklig trafik.",
            },
          ],
        },
        {
          id: "corrections",
          heading: "Rättelser",
          blocks: [
            {
              kind: "list",
              items: [
                "`TCP_NODELAY` var inte satt från början: axum-servers standardacceptor för Rustls lämnar det avstängt, så HTTP/2-svar kunde vänta omkring 40 ms på en fördröjd bekräftelse. Varje accepterad socket sätter det nu.",
                "Det första prestandatestet genererade last från Playwright, vars klient blev långsammare under körningens gång och felaktigt visade regressioner. De siffrorna förkastades.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
