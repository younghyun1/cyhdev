import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "november 2026",
  chrome: {
    overview: "Overzicht",
    updated: "Laatst bijgewerkt",
    onThisPage: "Op deze pagina",
    sectionNavigation: "Pagina’s over de techstack",
    previous: "Vorige",
    next: "Volgende",
    readingTime: "{minutes} min leestijd",
  },
  pages: {
    overview: {
      eyebrow: "Over deze site",
      title: "Techstack van de blog",
      lede: "Eén Rust-binary, één PostgreSQL-database, één kleine machine. Deze pagina’s beschrijven hoe de site is gebouwd, gehost, beveiligd en gemeten, en waarom hij zo in elkaar zit.",
      sections: [
        {
          id: "at-a-glance",
          heading: "In het kort",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53.889 req/s",
                  label: "Mediane doorvoer van gemengde openbare leesverzoeken met PGO en BOLT op vier servercores",
                },
                {
                  value: "~150 µs",
                  label: "Waargenomen round trip naar de database via een UNIX-socket, tegenover ~600 µs via TCP op localhost",
                },
                {
                  value: "0",
                  label: "Node.js-processen in productie; de frontend zit ingebed in de serverbinary",
                },
                {
                  value: "27",
                  count: 27,
                  label: "Interfacetalen, elk pas gedownload wanneer het wordt gekozen",
                },
                {
                  value: "4.096",
                  label: "Open verbindingen die vóór TLS worden toegelaten, met hoogstens 64 per adres of IPv6 /64",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Proces dat TLS, de API, WebSockets, WebRTC-media en de ingebedde frontend bedient",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Pad van een verzoek",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Browser",
                  detail: "HTTP/2 of HTTP/1.1 over TLS. De frontend wordt gecachet onder onveranderlijke URL’s met een fingerprint.",
                },
                {
                  label: "Toelating",
                  detail: "TCP-verbindingen worden vóór TLS toegelaten: 4.096 in totaal en 64 per adres of IPv6 /64.",
                },
                {
                  label: "rustls",
                  detail: "TLS wordt afgehandeld binnen het Rust-proces, met `TCP_NODELAY` op elke socket.",
                },
                {
                  label: "Axum",
                  detail: "Beveiligingsheaders, sessies, rate limits, compressie en groottelimieten voor de body, daarna een handler van de betreffende functie.",
                },
                {
                  label: "Cache · Diesel",
                  detail: "Begrensde caches in het geheugen beantwoorden veelgevraagde leesverzoeken; al het andere gaat via asynchrone Diesel.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "Op dezelfde host via een UNIX-socket, met asynchrone I/O via io_uring.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Hoofdstukken",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Infrastructuur",
                  href: "/about-blog/infrastructure",
                  meta: "Host · Gentoo · netwerk",
                  text: "De machine, het besturingssysteem en het bestandssysteem, het netwerk en waarom er geen reverse proxy is.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "De servicestack, het datamodel, de grenzen van de runtime, beveiligingsheaders en API-contracten.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "De applicatieshell, het laden, lokalisatie, het designsysteem en animaties.",
                },
                {
                  title: "Subsystemen",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "Livechat met een SFU binnen het proces, de Minecraft-verkenner, Geo-IP-opzoekingen en WebAssembly.",
                },
                {
                  title: "Prestaties",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Geoptimaliseerde builds, hoe ze zijn gemeten en wat dat opleverde.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Hoe wijzigingen worden gecontroleerd",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy op native targets en WebAssembly-targets als poort voor de implementatie.",
                "Lokale cargo xtask-controles voor opmaak, native unittests, afwijkingen in het OpenAPI-contract en de typecontrole, lint, unittests en build van de frontend.",
                "Playwright op Chromium en WebKit met toegankelijkheidscontroles van axe en controles op mobiele reflow tot 320 CSS-pixels.",
                "Integratietests en migratierollbacktests op een wegwerpinstantie van PostgreSQL, plus een lokale gitleaks-scan op geheimen.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Techstack",
      title: "Infrastructuur",
      lede: "Een kleine machine thuis, een Linuxdistributie op basis van broncode en geen enkele laag die het verkeer niet nodig heeft.",
      sections: [
        {
          id: "host",
          heading: "De host",
          blocks: [
            {
              kind: "paragraph",
              text: "De site draait op een [miniserver](https://store.minisforum.com/products/minisforum-um690l-slim) bij mij thuis: een mobiele Ryzen 9-processor met acht cores die tot 4,9 GHz boost, 32 GB RAM en een NVMe-SSD van 1 TB. Voor ongeveer $400 kost hij minder dan een jaar aan merkbaar zwakkere cloudhardware, en hij laat zien hoe graag ik zelf host.",
            },
            {
              kind: "paragraph",
              text: "Daarop draaien de gecombineerde backend- en frontendserver, PostgreSQL en een Minecraft-server. Wil je meespelen, stuur me dan een e-mail.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo en btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) is niet de voor de hand liggende keuze voor een server; op mijn werk zou ik Debian stable op ext4 kiezen. Hier vind ik het leuk om pakketten zelf te compileren, met optimalisaties voor precies deze processor.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) biedt snapshots, compressie en copy-on-write, maar copy-on-write fragmenteert databasebestanden sterk. De datadirectory’s van PostgreSQL en Minecraft zijn er daarom van uitgesloten. Een [bestandssysteembenchmark](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) laat zien dat btrfs met standaardinstellingen achterblijft bij ext4 en XFS; ik vermoed dat het uitschakelen van copy-on-write het grootste deel van dat verschil wegneemt, en dat zou een interessante meting zijn.",
            },
          ],
        },
        {
          id: "network",
          heading: "Netwerk",
          blocks: [
            {
              kind: "paragraph",
              text: "Het verkeer komt binnen via een bekabelde Xfinity-verbinding van 1 Gbps; de modem van de provider heeft geen 2,5 Gb-ethernet, wat deze site niet zal merken. Route 53 host de DNS.",
            },
            {
              kind: "paragraph",
              text: "Binnen de machine is er geen reverse proxy, containerruntime of service mesh. PostgreSQL draait direct op het besturingssysteem en één Rust-binary levert de frontend en de API. Gewone HTTP-verzoeken worden door hetzelfde proces doorgestuurd naar HTTPS.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "UNIX-sockets",
          blocks: [
            {
              kind: "paragraph",
              text: "De server bereikt PostgreSQL via een UNIX-socket in plaats van via TCP op localhost, waardoor de netwerkstack helemaal wordt overgeslagen. [Benchmarks](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) laten zien dat het aantal transacties per seconde bijna verdubbelt; hier heb ik round trips tot **150 µs** waargenomen, tegenover ongeveer **600 µs** via localhost. Het is ouderwets en voor de meeste clouddeployments irrelevant, en het is sneller.",
            },
            {
              kind: "paragraph",
              text: "PostgreSQL 18, uitgebracht in september 2025, kreeg [asynchrone I/O](https://pganalyze.com/blog/postgres-18-async-io) en hier is io_uring ingeschakeld, wat het lezen merkbaar versnelt.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Build en deployment",
          blocks: [
            {
              kind: "list",
              items: [
                "De build van de frontend zit samen met vooraf gecomprimeerde zstd- en gzip-kopieën ingebed in de backendbinary, zodat er in productie geen Node.js-proces draait.",
                "Geoptimaliseerde builds kunnen worden getraind met profielgestuurde optimalisatie en na het linken worden geoptimaliseerd met BOLT; zie [Prestaties](/about-blog/performance).",
                "Inloggegevens, certificaten en runtimegegevens blijven buiten Git en een gitleaks-scan controleert de boomstructuur voordat wijzigingen de machine verlaten.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Techstack",
      title: "Backend",
      lede: "Rust op Axum en Tokio, PostgreSQL via asynchrone Diesel en een expliciete limiet op alles wat een client open kan houden.",
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
                { label: "Integraties", items: ["openidconnect", "lettre", "Amazon S3", "webrtc-rs", "image + AVIF"] },
                { label: "Contracten", items: ["utoipa", "OpenAPI", "Swagger UI", "Generated TypeScript"] },
              ],
            },
            {
              kind: "paragraph",
              text: "De dienst is één proces met gedeelde status: sessies in het geheugen, synchronisatie van caches bij het opstarten en geplande onderhoudstaken. mimalloc is de globale allocator; als een machine echte cores heeft, is het gedrag van de allocator het waard om serieus te nemen.",
            },
          ],
        },
        {
          id: "data",
          heading: "Datamodel",
          blocks: [
            {
              kind: "paragraph",
              text: "Bij het Koreaanse startup- en bedrijfswerk dat ik heb gezien golden MySQL en MariaDB als de enige serieuze relationele databases. PostgreSQL heeft ze op de punten waar ik om geef al lang ingehaald, van native UUID’s tot binaire JSON.",
            },
            {
              kind: "list",
              items: [
                "**Berichten** hebben slugs, samenvattingen, JSONB-metadata, publicatiestatus, gedenormaliseerde tellers en koppelingen met trefwoorden. Reacties vormen draden via een optionele ouder en stemmen op berichten en reacties staan in aparte tabellen.",
                "**Accounts** bewaren land en taal, tabellen voor rollen en rechten, en profielfoto’s met versies in een eigen tabel in plaats van op de veelgebruikte gebruikersrij.",
                "**Sleutels** zijn op tijd gesorteerde UUIDv7-waarden: niet op te sommen zoals opeenvolgende ID’s en veel gunstiger voor de B-tree-lokaliteit dan willekeurige UUIDv4’s.",
                "**Zoeken** gebruikt een op schijf bewaarde tantivy-index over titels en trefwoorden van berichten, die consistent wordt gehouden met PostgreSQL, dat leidend blijft.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Grenzen van de runtime",
          blocks: [
            {
              kind: "list",
              items: [
                "Verbindingen worden vóór TLS toegelaten via een gedeelde limiter: 4.096 open verbindingen en 64 per IPv4-adres of IPv6 /64. Geweigerde sockets worden gesloten zonder handshake.",
                "De headers van HTTP/1-verzoeken moeten binnen 20 seconden binnenkomen; HTTP/2-verbindingen krijgen elke 30 seconden een ping en staan 128 gelijktijdige streams toe.",
                "Elke cache heeft een vaste grens en een bron waaruit hij kan doorlezen; geen enkele groeit mee met sleutels die een client bepaalt. De cache voor WebAssembly-bundels bevat bijvoorbeeld hoogstens 256 MiB.",
                "Blokkerend werk, zoals het coderen van afbeeldingen, draait op begrensde workerpools in plaats van op Tokio-workers.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Beveiligingsheaders",
          blocks: [
            {
              kind: "paragraph",
              text: "De Content-Security-Policy wordt eenmalig bij het opstarten opgebouwd. Inline scripts zijn alleen toegestaan via SHA-256-hashes die worden berekend uit de `index.html` die in de binary is gecompileerd, zodat het themascript dat vóór het tekenen draait niet van het beleid kan afwijken. De hoofdorigin instantieert nooit WebAssembly: demo’s en de EU5-applicatie draaien in sandboxed frames met hun eigen beleid.",
            },
            {
              kind: "paragraph",
              text: "Sessiecookies zijn `Secure` en `HttpOnly`, handelingen van superusers controleren rollen op de server, accounts kunnen OpenID Connect-identiteiten koppelen en de Swagger UI vereist in productie authenticatie.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Contracten en assets",
          blocks: [
            {
              kind: "list",
              items: [
                "Handlers hebben OpenAPI-annotaties, de TypeScript-clients van de frontend worden daaruit gegenereerd en cargo xtask openapi mislukt wanneer de gegenereerde bestanden afwijken.",
                "Het manifest van Vite wordt samen met de assets ingebed. Alleen bestanden met een fingerprint die in het manifest staan, krijgen een cachebeleid van een jaar met `immutable`; HTML wordt altijd opnieuw gevalideerd.",
                "Ontbrekende assetpaden geven 404 met `no-store` terug in plaats van de applicatieshell, zodat een verouderde pagina geen HTML kan cachen onder een script-URL.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Techstack",
      title: "Frontend",
      lede: "Een aan de clientzijde gerenderde SolidJS 2-applicatie, gecompileerd naar statische assets en ingebed in de serverbinary.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "Applicatie", items: ["SolidJS 2", "Solid Router 2", "TypeScript", "Vite 8"] },
                { label: "Styling", items: ["Tailwind CSS 4", "IBM Plex Sans", "IBM Plex Mono", "OKLab"] },
                { label: "Componenten", items: ["Toast UI Editor", "Leaflet", "Chart.js", "highlight.js", "DOMPurify"] },
                { label: "Testen", items: ["Vitest", "Playwright", "axe-core"] },
              ],
            },
            {
              kind: "paragraph",
              text: "De fijnmazige signalen van Solid werken alleen de DOM-nodes bij die afhangen van gewijzigde waarden, en met de 2.0-releasekandidaten kunnen asynchrone leesacties worden opgeschort tot aan de dichtstbijzijnde laadgrens. De router laadt elke pagina lazy en er is geen JavaScript-server in productie: de Rust-binary levert statische bestanden.",
            },
          ],
        },
        {
          id: "loading",
          heading: "Laden",
          blocks: [
            {
              kind: "list",
              items: [
                "Pagina’s, het chatpaneel van de startpagina en de contourshader ervan zijn aparte chunks en de build rapporteert de gzip-grootte van wat als eerste wordt geladen.",
                "Alleen Engelse interfaceteksten zitten in de eerste chunk. Andere talen, en de Over-pagina’s van elke taal, zijn aparte chunks die bij het kiezen worden geladen.",
                "Mobiele prestatietests met throttling mikken op een LCP van hoogstens 2,5 s, interacties van hoogstens 200 ms en een CLS van hoogstens 0,1.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Lokalisatie",
          blocks: [
            {
              kind: "paragraph",
              text: "De interface ondersteunt 27 talen, waaronder rechts-naar-links-layouts voor Arabisch, Perzisch en Urdu. Een catalogus van ongeveer 830 sleutels wordt gevalideerd tegen het register van de backend en placeholders moeten een vertaling overleven. Deze Over-pagina’s zijn getypeerde datamodules, zodat een vertaling niet ongemerkt een sectie kan weglaten die in het Engels wel bestaat.",
            },
          ],
        },
        {
          id: "design",
          heading: "Designsysteem",
          blocks: [
            {
              kind: "paragraph",
              text: "De uitstraling is warm en technisch: crèmekleurig papier of puur zwart, haarfijne randen, monospace voor alles wat technisch is en telkens één accentkleur. Kleuren zijn semantische CSS-variabelen, zodat componenten nooit varianten voor de donkere modus nodig hebben. Accentpresets worden getest op contrast met de paginaachtergronden, de eigen RGB-kleur van een bezoeker wordt in elke modus aangepast zodat die leesbaar blijft, en de lichte of donkere modus en het accent worden toegepast vóór de eerste weergave.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Animaties",
          blocks: [
            {
              kind: "list",
              items: [
                "Het verschijnen van secties en de voortgangsbalk voor het lezen gebruiken CSS-animaties die door scrollen worden aangestuurd en waar ondersteund buiten de hoofdthread draaien.",
                "Statistieken tellen op via een geregistreerd `@property`-geheel getal, waarbij de eindwaarde voor hulptechnologie in de markup staat.",
                "De themaschakelaar breidt zich vanaf de knop uit via de View Transitions API.",
                "Het contourveld van de startpagina is een kleine WebGL-shader die laadt zodra de pagina inactief is, alleen draait in geschikte desktopbrowsers, zijn framerate begrenst en pauzeert buiten beeld.",
                "Een instelling van de site schakelt dit alles uit. Animaties staan standaard aan en volgen niet de instelling voor verminderde beweging van het besturingssysteem.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Techstack",
      title: "Subsystemen",
      lede: "De onderdelen van de site die interessanter zijn dan een blog strikt nodig heeft.",
      sections: [
        {
          id: "live-chat",
          heading: "Livechat en gesprekken",
          blocks: [
            {
              kind: "paragraph",
              text: "Chat loopt via een WebSocket met een compacte binaire codec, moderatie en een begrensde geschiedenis. Spraak- en videogesprekken gaan via een Selective Forwarding Unit die binnen het Axum-proces op webrtc-rs draait: elke browser heeft één peerverbinding en de SFU stuurt gecodeerde media door zonder transcodering, via begrensde kanalen per publicatie en een maximum aantal deelnemers. [Livechat openen](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Minecraft-terreinverkenner",
          blocks: [
            {
              kind: "paragraph",
              text: "Een Paper-plugin biedt een UNIX-socket voor dezelfde gebruiker voor begrensde wereldqueries, met telkens één bewerking en strikte limieten op de framegrootte. De [verkenner](/minecraft) tekent squaremap-tegels met een eigen Leaflet-client in plaats van de JavaScript van de gegenereerde kaart uit te voeren.",
            },
            {
              kind: "paragraph",
              text: "Biome-overlays gebruiken **CYBM**, een compact binair tegelformaat dat gzip-gecomprimeerd wordt geleverd, of een PNG waarvan het privé-chunk `cyBM` de exacte biome-identiteiten bevat, zodat bij het aanwijzen nooit wordt geraden op basis van pixelkleuren. Voorspellingen op basis van de seed voor onverkend terrein komen van een aparte wereldgeneratieworker die op Pumpkin is gebouwd.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Geo-IP-opzoekingen",
          blocks: [
            {
              kind: "paragraph",
              text: "De [bezoekerskaart](/visitor-board) en de [IP-opzoeking](/geo-ip-db) lokaliseren adressen zonder een externe API aan te roepen. Een preprocessingtool zet de CSV-bestanden van IP2Location LITE om in `BTreeMap`s met het begin van het bereik als sleutel, codeert ze met bitcode en comprimeert ze met zstd op niveau 22.",
            },
            {
              kind: "table",
              columns: ["Dataset", "CSV", "Bundel"],
              numericColumns: [1, 2],
              rows: [
                ["IPv4", "312,5 MB", "24,9 MB"],
                ["IPv6", "794,2 MB", "42,4 MB"],
              ],
            },
            {
              kind: "paragraph",
              text: "Bij het opstarten decodeert de server telkens één bundel om het piekgeheugen te beperken en dedupliceert hij de herhaalde plaatsnamen. Een opzoeking vindt het laatste bereik dat op of onder het adres begint en controleert de bovengrens ervan.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "Geüploade [demobundels](/projects) worden op schijf klaargezet, herkend aan hun magic bytes, beperkt tot 50 MiB na decompressie en geleverd vanuit een begrensde cache binnen sandboxed frames. De workspace bouwt ook Rust-demo’s, waaronder een block breaker en een raytracer.",
            },
            {
              kind: "paragraph",
              text: "De [EU5 Location DB](/eu5-locations-db) is een Slint-applicatie die naar WebAssembly is gecompileerd. De pagina stuurt haar lichte of donkere modus via een klein berichtenprotocol door, zodat het ingebedde palet met de site meeverandert.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Fotopijplijn",
          blocks: [
            {
              kind: "paragraph",
              text: "Uploads worden binnen vaste pixelgrenzen gedecodeerd en met de langzaamste preset naar AVIF gecodeerd, waarbij verwerkingstijd wordt ingeruild voor kleinere bestanden. Locaties uit EXIF-gegevens kunnen vóór het publiceren op een kaart worden bevestigd en er draaien nooit meer dan twee verwerkingstaken tegelijk.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Techstack",
      title: "Prestaties",
      lede: "Profielgestuurde optimalisatie en BOLT op de backend, gemeten met een loadgenerator die niet de bottleneck is.",
      sections: [
        {
          id: "approach",
          heading: "Aanpak",
          blocks: [
            {
              kind: "paragraph",
              text: "Geoptimaliseerde builds worden getraind op een representatieve workload. Een geïnstrumenteerde binary legt profielen vast tegen een wegwerpfixture met synthetische endpoints voor PostgreSQL, SMTP, S3, OpenID Connect en Minecraft. LLVM bouwt de server opnieuw met die profielen (PGO) en BOLT herschikt daarna de codeindeling van de uiteindelijke binary op basis van bemonsterde uitvoering. Alle varianten delen LTO, frame pointers en een opnieuw gebouwde standaardbibliotheek.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Opzet",
          blocks: [
            {
              kind: "list",
              items: [
                "Server, fixtureproviders en PostgreSQL vastgezet op vier cores van een AMD Ryzen 7 9800X3D; de loadclient vastgezet op de andere vier.",
                "HTTPS over gepoolde HTTP/1.1 met 32 verbindingen, volledig gelezen antwoorden en deadlines van twee seconden.",
                "Drie geroteerde rondes met vijf metingen per variant, elk na 10.000 warm-upverzoeken, vanaf een verse databasesnapshot.",
                "In totaal 27.000.000 getimede verzoeken; elk gaf HTTP 200 terug.",
                "De load komt van oha; Playwright controleert het antwoord van elke route buiten het getimede venster.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Resultaten",
          blocks: [
            {
              kind: "bars",
              caption: "Mediaan van verzoeken per seconde over 15 metingen per variant.",
              groups: [
                {
                  label: "Openbare leesverzoeken, tien routes",
                  items: [
                    { label: "Basislijn", value: 44874.12, display: "44.874" },
                    { label: "PGO", value: 52925.46, display: "52.925" },
                    { label: "PGO + BOLT", value: 53888.61, display: "53.889", emphasis: true },
                  ],
                },
                {
                  label: "Kleine health-JSON",
                  items: [
                    { label: "Basislijn", value: 272837.41, display: "272.837" },
                    { label: "PGO", value: 308685.26, display: "308.685", emphasis: true },
                    { label: "PGO + BOLT", value: 306276.64, display: "306.277" },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "Staartlatentie voor de mix van tien routes.",
              columns: ["Variant", "p95 (ms)", "p99 (ms)"],
              numericColumns: [1, 2],
              rows: [
                ["Basislijn", "2,380", "2,967"],
                ["PGO", "2,020", "2,524"],
                ["PGO + BOLT", "1,987", "2,483"],
              ],
            },
            {
              kind: "paragraph",
              text: "PGO verhoogde de gemengde doorvoer met **17,9%** en PGO met BOLT met **20,1%**, en beide geoptimaliseerde builds wonnen elke geroteerde ronde. BOLT voegde 1,8% toe aan PGO bij de gemengde leesverzoeken en was 0,8% trager op de triviale health-route. Dit zijn resultaten met vier cores; de productiecapaciteit hangt ook af van het netwerk, de werkset van de database en echt verkeer.",
            },
          ],
        },
        {
          id: "corrections",
          heading: "Correcties",
          blocks: [
            {
              kind: "list",
              items: [
                "`TCP_NODELAY` was in het begin niet ingesteld: de standaard Rustls-acceptor van axum-server laat de optie uit, waardoor HTTP/2-antwoorden ongeveer 40 ms konden wachten op een vertraagde bevestiging. Nu stelt elke geaccepteerde socket hem in.",
                "De eerste benchmark gaf de load vanuit Playwright, waarvan de client in de loop van een run trager werd, en liet ten onrechte regressies zien. Die cijfers zijn verworpen.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
