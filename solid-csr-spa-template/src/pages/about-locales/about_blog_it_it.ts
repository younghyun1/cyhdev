import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "novembre 2026",
  chrome: {
    overview: "Panoramica",
    updated: "Ultimo aggiornamento",
    onThisPage: "In questa pagina",
    sectionNavigation: "Pagine dello stack tecnologico",
    previous: "Precedente",
    next: "Successivo",
    readingTime: "{minutes} min di lettura",
  },
  pages: {
    overview: {
      eyebrow: "Informazioni su questo sito",
      title: "Stack tecnologico del blog",
      lede: "Un solo binario Rust, un solo database PostgreSQL, una sola macchina piccola. Queste pagine descrivono come il sito è costruito, ospitato, protetto e misurato, e perché ha questa forma.",
      sections: [
        {
          id: "at-a-glance",
          heading: "In sintesi",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53.889 req/s",
                  label: "Throughput mediano di letture pubbliche miste con PGO e BOLT su quattro core del server",
                },
                {
                  value: "~150 µs",
                  label: "Round trip del database osservato su socket UNIX, contro ~600 µs su TCP localhost",
                },
                {
                  value: "0",
                  label: "Processi Node.js in produzione; il frontend è incorporato nel binario del server",
                },
                {
                  value: "24",
                  count: 24,
                  label: "Lingue dell'interfaccia, ciascuna scaricata solo quando viene selezionata",
                },
                {
                  value: "4.096",
                  label: "Connessioni aperte ammesse prima di TLS, con al massimo 64 per indirizzo o per IPv6 /64",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Processo che serve TLS, l'API, i WebSocket, i media WebRTC e il frontend incorporato",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Percorso della richiesta",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Browser",
                  detail: "HTTP/2 o HTTP/1.1 su TLS. Il frontend è in cache sotto URL immutabili con impronta digitale.",
                },
                {
                  label: "Ammissione",
                  detail: "Le connessioni TCP sono ammesse prima di TLS: 4.096 in totale e 64 per indirizzo o per IPv6 /64.",
                },
                {
                  label: "rustls",
                  detail: "TLS termina dentro il processo Rust, con `TCP_NODELAY` su ogni socket.",
                },
                {
                  label: "Axum",
                  detail: "Header di sicurezza, sessioni, limiti di frequenza, compressione e limiti sul corpo, poi un handler della funzionalità.",
                },
                {
                  label: "Cache · Diesel",
                  detail: "Cache in memoria limitate rispondono alle letture frequenti; tutto il resto passa da Diesel asincrono.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "Sullo stesso host tramite socket UNIX, con I/O asincrono io_uring.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Capitoli",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Infrastruttura",
                  href: "/about-blog/infrastructure",
                  meta: "Host · Gentoo · rete",
                  text: "La macchina, il sistema operativo e il filesystem, la rete e il motivo per cui non c'è un reverse proxy.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "Lo stack del servizio, il modello dei dati, i limiti a runtime, gli header di sicurezza e i contratti dell'API.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "La shell dell'applicazione, il caricamento, la localizzazione, il design system e il movimento.",
                },
                {
                  title: "Sottosistemi",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "La chat dal vivo con una SFU nello stesso processo, l'esploratore di Minecraft, le ricerche Geo-IP e WebAssembly.",
                },
                {
                  title: "Prestazioni",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Build ottimizzate, come sono state misurate e i risultati.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Come si verificano le modifiche",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy sui target nativo e WebAssembly come controllo di implementazione.",
                "Controlli locali con cargo xtask per formattazione, unit test nativi, deriva del contratto OpenAPI e, per il frontend, typecheck, lint, unit test e build.",
                "Playwright su Chromium e WebKit con controlli di accessibilità axe e controlli di reflow su mobile fino a 320 pixel CSS.",
                "Test di integrazione e di rollback delle migrazioni su un'istanza PostgreSQL usa e getta, più una scansione locale dei segreti con gitleaks.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Stack tecnologico",
      title: "Infrastruttura",
      lede: "Una macchina piccola in casa, una distribuzione Linux basata sui sorgenti e nessun livello che il traffico non richieda.",
      sections: [
        {
          id: "host",
          heading: "L'host",
          blocks: [
            {
              kind: "paragraph",
              text: "Il sito gira su un [miniserver](https://store.minisforum.com/products/minisforum-um690l-slim) a casa mia: un processore mobile Ryzen 9 a otto core con boost fino a 4,9 GHz, 32 GB di RAM e un SSD NVMe da 1 TB. Con circa $400 costa meno di un anno di hardware cloud sensibilmente più debole, e riflette il mio entusiasmo per il self-hosting.",
            },
            {
              kind: "paragraph",
              text: "Esegue il server che unisce backend e frontend, PostgreSQL e un server Minecraft. Se vuoi giocare, scrivimi un'email.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo e btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) non è la scelta ovvia per un server; al lavoro sceglierei Debian stable su ext4. Qui mi diverto a compilare i pacchetti da me, con ottimizzazioni per lo specifico processore.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) offre snapshot, compressione e copy-on-write, ma il copy-on-write frammenta molto i file dei database. Le directory dei dati di PostgreSQL e di Minecraft ne sono quindi escluse. Un [benchmark dei filesystem](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) mostra btrfs con le impostazioni predefinite dietro a ext4 e XFS; sospetto che disattivare il copy-on-write colmi gran parte di quel divario, e sarebbe una misurazione interessante.",
            },
          ],
        },
        {
          id: "network",
          heading: "Rete",
          blocks: [
            {
              kind: "paragraph",
              text: "Il traffico arriva su una connessione Xfinity cablata da 1 Gbps; il modem del provider non ha Ethernet a 2,5 Gb, cosa che questo sito non nota. Route 53 ospita il DNS.",
            },
            {
              kind: "paragraph",
              text: "All'interno della macchina non ci sono reverse proxy, runtime di container o service mesh. PostgreSQL gira sul sistema operativo e un solo binario Rust serve il frontend e l'API. Le richieste HTTP in chiaro vengono reindirizzate a HTTPS dallo stesso processo.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "Socket UNIX",
          blocks: [
            {
              kind: "paragraph",
              text: "Il server raggiunge PostgreSQL tramite un socket UNIX invece che con TCP su localhost, saltando del tutto lo stack di rete. I [benchmark](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) mostrano frequenze di transazione quasi raddoppiate; qui ho osservato round trip fino a **150 µs**, contro circa **600 µs** su localhost. È un metodo all'antica e irrilevante per la maggior parte dei deployment cloud, ed è più veloce.",
            },
            {
              kind: "paragraph",
              text: "PostgreSQL 18, uscito a settembre 2025, ha introdotto l'[I/O asincrono](https://pganalyze.com/blog/postgres-18-async-io), e qui io_uring è attivo, il che accelera sensibilmente le letture.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Build e deploy",
          blocks: [
            {
              kind: "list",
              items: [
                "La build del frontend è incorporata nel binario del backend insieme a copie precompresse in zstd e gzip, quindi in produzione non gira alcun processo Node.js.",
                "Le build ottimizzate possono essere addestrate con l'ottimizzazione guidata dal profilo e ottimizzate dopo il link con BOLT; vedi [Prestazioni](/about-blog/performance).",
                "Credenziali, certificati e dati di runtime restano fuori da Git, e una scansione con gitleaks controlla l'albero prima che le modifiche lascino la macchina.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Stack tecnologico",
      title: "Backend",
      lede: "Rust su Axum e Tokio, PostgreSQL tramite Diesel asincrono e un limite esplicito su tutto ciò che un client può tenere aperto.",
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
                  label: "Dati",
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
                  label: "Integrazioni",
                  items: [
                    "openidconnect",
                    "lettre",
                    "Amazon S3",
                    "webrtc-rs",
                    "image + AVIF",
                  ],
                },
                {
                  label: "Contratti",
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
              text: "Il servizio è un unico processo con stato condiviso: sessioni in memoria, sincronizzazione della cache all'avvio e job di manutenzione pianificati. mimalloc è l'allocatore globale; se una macchina ha core veri, il comportamento dell'allocatore merita attenzione.",
            },
          ],
        },
        {
          id: "data",
          heading: "Modello dei dati",
          blocks: [
            {
              kind: "paragraph",
              text: "Nel lavoro con startup e aziende coreane che ho visto, MySQL e MariaDB erano considerati gli unici database relazionali seri. PostgreSQL li ha superati da tempo negli ambiti che mi interessano, dagli UUID nativi al JSON binario.",
            },
            {
              kind: "list",
              items: [
                "Gli **articoli** hanno slug, riassunti, metadati JSONB, stato di pubblicazione, contatori denormalizzati e mappature dei tag. I commenti sono organizzati a discussione tramite un genitore nullable, e i voti agli articoli e ai commenti hanno tabelle separate.",
                "Gli **account** conservano paese e lingua, tabelle di ruoli e permessi, e immagini del profilo versionate in una tabella propria invece che nella riga utente, che è molto letta.",
                "Le **chiavi** sono UUIDv7 ordinati nel tempo: non enumerabili come gli ID sequenziali e molto più favorevoli alla località dei B-tree rispetto agli UUIDv4 casuali.",
                "La **ricerca** usa un indice tantivy persistito su disco sui titoli e sui tag degli articoli, mantenuto coerente con PostgreSQL, che resta la fonte autorevole.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Limiti a runtime",
          blocks: [
            {
              kind: "list",
              items: [
                "Le connessioni sono ammesse prima di TLS tramite un limitatore condiviso: 4.096 connessioni aperte e 64 per indirizzo IPv4 o per IPv6 /64. I socket rifiutati si chiudono senza handshake.",
                "Le intestazioni delle richieste HTTP/1 devono arrivare entro 20 secondi; le connessioni HTTP/2 ricevono un ping ogni 30 secondi e ammettono 128 stream concorrenti.",
                "Ogni cache ha un limite fisso e un'autorità da cui può rileggere; nessuna cresce con chiavi controllate da un client. La cache dei bundle WebAssembly, per esempio, contiene al massimo 256 MiB.",
                "Il lavoro bloccante, come la codifica delle immagini, gira su pool di worker limitati invece che sui worker di Tokio.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Header di sicurezza",
          blocks: [
            {
              kind: "paragraph",
              text: "La Content-Security-Policy viene costruita una sola volta all'avvio. Gli script inline sono ammessi solo tramite hash SHA-256 calcolati dal file `index.html` compilato nel binario, così lo script del tema eseguito prima del rendering non può divergere dalla policy. L'origine principale non istanzia mai WebAssembly: le demo e l'applicazione EU5 girano in frame isolati con le proprie policy.",
            },
            {
              kind: "paragraph",
              text: "I cookie di sessione sono `Secure` e `HttpOnly`, le operazioni da superutente verificano i ruoli sul server, gli account possono collegare identità OpenID Connect e la Swagger UI richiede l'autenticazione in produzione.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Contratti e asset",
          blocks: [
            {
              kind: "list",
              items: [
                "Gli handler hanno annotazioni OpenAPI, i client TypeScript del frontend sono generati da esse e cargo xtask openapi fallisce quando i file generati divergono.",
                "Il manifest di Vite è incorporato con gli asset. Solo i file elencati nel manifest e con impronta digitale ricevono una policy di cache `immutable` di un anno; l'HTML viene sempre riconvalidato.",
                "I percorsi di asset mancanti restituiscono 404 con `no-store` invece della shell dell'applicazione, così una pagina obsoleta non può mettere in cache HTML sotto l'URL di uno script.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Stack tecnologico",
      title: "Frontend",
      lede: "Un'applicazione SolidJS 2 con rendering lato client, compilata in asset statici e incorporata nel binario del server.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Applicazione",
                  items: [
                    "SolidJS 2",
                    "Solid Router 2",
                    "TypeScript",
                    "Vite 8",
                  ],
                },
                {
                  label: "Stili",
                  items: [
                    "Tailwind CSS 4",
                    "IBM Plex Sans",
                    "IBM Plex Mono",
                    "OKLab",
                  ],
                },
                {
                  label: "Componenti",
                  items: [
                    "Toast UI Editor",
                    "Leaflet",
                    "Chart.js",
                    "highlight.js",
                    "DOMPurify",
                  ],
                },
                {
                  label: "Test",
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
              text: "I signal a grana fine di Solid aggiornano solo i nodi del DOM che dipendono dai valori cambiati, e le release candidate della versione 2.0 permettono alle letture asincrone di sospendersi fino al confine di caricamento più vicino. Il router carica ogni pagina in modo lazy e in produzione non c'è alcun server JavaScript: il binario Rust serve i file statici.",
            },
          ],
        },
        {
          id: "loading",
          heading: "Caricamento",
          blocks: [
            {
              kind: "list",
              items: [
                "Le pagine, il pannello chat della home page e il suo shader dei contorni sono chunk separati, e la build riporta la dimensione gzip di ciò che si carica per primo.",
                "Nel chunk iniziale è incluso solo il testo dell'interfaccia in inglese. Le altre lingue, e le pagine Info di ciascuna lingua, sono chunk separati caricati alla selezione.",
                "I test di prestazioni su mobile con throttling puntano a un LCP pari o inferiore a 2,5 s, a interazioni pari o inferiori a 200 ms e a un CLS pari o inferiore a 0,1.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Localizzazione",
          blocks: [
            {
              kind: "paragraph",
              text: "L'interfaccia supporta 24 lingue, inclusi i layout da destra a sinistra per arabo, persiano e urdu. Un catalogo di circa 740 chiavi viene validato rispetto al registro del backend, e i segnaposto devono sopravvivere alla traduzione. Queste pagine Info sono moduli di dati tipizzati, quindi una traduzione non può omettere in silenzio una sezione presente in inglese.",
            },
          ],
        },
        {
          id: "design",
          heading: "Design system",
          blocks: [
            {
              kind: "paragraph",
              text: "L'aspetto è caldo e tecnico: carta color crema o nero puro, bordi sottilissimi, monospazio per tutto ciò che è tecnico e un solo colore d'accento alla volta. I colori sono variabili CSS semantiche, quindi i componenti non hanno mai bisogno di varianti per la modalità scura. I preset d'accento sono testati per contrasto rispetto agli sfondi delle pagine, il colore RGB scelto da un visitatore viene regolato in ciascuna modalità per restare leggibile, e la modalità chiara o scura e l'accento vengono applicati prima del primo rendering.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Movimento",
          blocks: [
            {
              kind: "list",
              items: [
                "Le animazioni di comparsa delle sezioni e la barra di avanzamento della lettura usano animazioni CSS guidate dallo scorrimento, che dove supportato girano fuori dal thread principale.",
                "Le statistiche contano fino al valore finale tramite un intero `@property` registrato, con il valore finale presente nel markup per le tecnologie assistive.",
                "Il cambio di tema si espande dal pulsante attraverso la View Transitions API.",
                "Il campo di contorni della home page è un piccolo shader WebGL che si carica quando la pagina è inattiva, gira solo sui browser desktop adeguati, limita il frame rate e va in pausa quando è fuori schermo.",
                "Un'impostazione del sito disattiva tutto questo. Le animazioni sono attive per impostazione predefinita e non seguono l'impostazione di riduzione del movimento del sistema operativo.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Stack tecnologico",
      title: "Sottosistemi",
      lede: "Le parti del sito più interessanti di quanto un blog richieda davvero.",
      sections: [
        {
          id: "live-chat",
          heading: "Chat dal vivo e chiamate",
          blocks: [
            {
              kind: "paragraph",
              text: "La chat funziona su un WebSocket con un codec binario compatto, moderazione e cronologia limitata. Le chiamate vocali e video passano attraverso una Selective Forwarding Unit eseguita dentro il processo Axum su webrtc-rs: ogni browser mantiene una sola peer connection, e la SFU inoltra i media codificati senza transcodifica, tramite canali limitati per ogni pubblicazione e un tetto ai partecipanti. [Apri la chat dal vivo](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Esploratore del terreno di Minecraft",
          blocks: [
            {
              kind: "paragraph",
              text: "Un plugin Paper espone un socket UNIX riservato allo stesso utente per interrogazioni limitate del mondo, con un'operazione alla volta e limiti rigorosi sulla dimensione dei frame. L'[esploratore](/minecraft) disegna i tile di squaremap con un proprio client Leaflet, invece di eseguire il JavaScript della mappa generata.",
            },
            {
              kind: "paragraph",
              text: "Gli overlay dei biomi usano **CYBM**, un formato binario compatto di tile servito compresso con gzip, oppure un PNG il cui chunk privato `cyBM` contiene le identità esatte dei biomi, così il passaggio del mouse non deve mai indovinare dai colori dei pixel. Le previsioni basate sul seed per il terreno non esplorato provengono da un worker separato di generazione del mondo basato su Pumpkin.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Ricerche Geo-IP",
          blocks: [
            {
              kind: "paragraph",
              text: "La [mappa dei visitatori](/visitor-board) e la [ricerca IP](/geo-ip-db) localizzano gli indirizzi senza chiamare alcuna API esterna. Uno strumento di preelaborazione trasforma i file CSV di IP2Location LITE in `BTreeMap` indicizzate per inizio intervallo, li codifica con bitcode e li comprime con zstd al livello 22.",
            },
            {
              kind: "table",
              columns: [
                "Dataset",
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
              text: "All'avvio il server decodifica un bundle alla volta per limitare il picco di memoria e internalizza i nomi di luogo ripetuti. Una ricerca trova l'ultimo intervallo che inizia all'indirizzo o prima e ne controlla il limite finale.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "I [bundle di demo](/projects) caricati vengono messi in staging su disco, identificati dai magic byte, limitati a 50 MiB decompressi e serviti da una cache limitata all'interno di frame in sandbox. Il workspace compila anche demo in Rust, tra cui un block breaker e un ray tracer.",
            },
            {
              kind: "paragraph",
              text: "L'[EU5 Location DB](/eu5-locations-db) è un'applicazione Slint compilata in WebAssembly. La pagina le invia la modalità chiara o scura tramite un piccolo protocollo di messaggi, così la palette incorporata cambia insieme al sito.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Pipeline fotografica",
          blocks: [
            {
              kind: "paragraph",
              text: "Gli upload vengono decodificati entro limiti fissi di pixel e codificati in AVIF con il preset più lento, scambiando tempo di elaborazione con file più piccoli. Le posizioni ricavate dai dati EXIF possono essere confermate su una mappa prima della pubblicazione, e non più di due job di elaborazione girano contemporaneamente.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Stack tecnologico",
      title: "Prestazioni",
      lede: "L'ottimizzazione guidata dal profilo (PGO) e BOLT sul backend, misurati con un generatore di carico che non è il collo di bottiglia.",
      sections: [
        {
          id: "approach",
          heading: "Approccio",
          blocks: [
            {
              kind: "paragraph",
              text: "Le build ottimizzate vengono addestrate su un carico di lavoro rappresentativo. Un binario strumentato registra i profili su una fixture usa e getta con endpoint sintetici di PostgreSQL, SMTP, S3, OpenID Connect e Minecraft. LLVM ricompila il server con quei profili (PGO), poi BOLT riordina il layout del codice del binario finale a partire dall'esecuzione campionata. Tutte le varianti condividono LTO, frame pointer e una libreria standard ricompilata.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Configurazione",
          blocks: [
            {
              kind: "list",
              items: [
                "Server, provider della fixture e PostgreSQL vincolati a quattro core di un AMD Ryzen 7 9800X3D; il client di carico vincolato agli altri quattro.",
                "HTTPS su HTTP/1.1 con pool di 32 connessioni, lettura completa delle risposte e timeout di due secondi.",
                "Tre round a rotazione con cinque campioni per variante, ciascuno dopo 10.000 richieste di riscaldamento, a partire da uno snapshot nuovo del database.",
                "27.000.000 richieste cronometrate in totale; tutte hanno restituito HTTP 200.",
                "Il carico viene da oha; Playwright verifica la risposta di ogni route fuori dalla finestra cronometrata.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Risultati",
          blocks: [
            {
              kind: "bars",
              caption: "Richieste al secondo mediane su 15 campioni per variante.",
              groups: [
                {
                  label: "Letture pubbliche su dieci route",
                  items: [
                    {
                      label: "Riferimento",
                      value: 44874.12,
                      display: "44.874",
                    },
                    {
                      label: "PGO",
                      value: 52925.46,
                      display: "52.925",
                    },
                    {
                      label: "PGO + BOLT",
                      value: 53888.61,
                      display: "53.889",
                      emphasis: true,
                    },
                  ],
                },
                {
                  label: "Piccolo JSON di health",
                  items: [
                    {
                      label: "Riferimento",
                      value: 272837.41,
                      display: "272.837",
                    },
                    {
                      label: "PGO",
                      value: 308685.26,
                      display: "308.685",
                      emphasis: true,
                    },
                    {
                      label: "PGO + BOLT",
                      value: 306276.64,
                      display: "306.277",
                    },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "Latenza di coda per il mix di dieci route.",
              columns: [
                "Variante",
                "p95 (ms)",
                "p99 (ms)",
              ],
              numericColumns: [
                1,
                2,
              ],
              rows: [
                [
                  "Riferimento",
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
              text: "La PGO ha aumentato il throughput misto del **17,9%** e la PGO con BOLT del **20,1%**, ed entrambe le build ottimizzate hanno vinto ogni round a rotazione. BOLT ha aggiunto l'1,8% rispetto alla PGO sulle letture miste ed è stato più lento dello 0,8% sulla route health, che è banale. Sono risultati su quattro core; la capacità in produzione dipende anche dalla rete, dal working set del database e dal traffico reale.",
            },
          ],
        },
        {
          id: "corrections",
          heading: "Correzioni",
          blocks: [
            {
              kind: "list",
              items: [
                "`TCP_NODELAY` all'inizio non era impostato: l'acceptor Rustls predefinito di axum-server lo lascia disattivato, quindi le risposte HTTP/2 potevano attendere circa 40 ms per un acknowledgement ritardato. Ora ogni socket accettato lo imposta.",
                "Il primo benchmark generava il carico da Playwright, il cui client rallentava man mano che l'esecuzione procedeva, e mostrava regressioni errate. Quei numeri sono stati scartati.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
