import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "novembre 2026",
  chrome: {
    overview: "Panoramica",
    updated: "Ultimo aggiornamento",
    onThisPage: "In questa pagina",
    sectionNavigation: "Pagine Info",
    previous: "Precedente",
    next: "Successivo",
    readingTime: "{minutes} min di lettura",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "Info",
      lede: "Ingegnere backend e infrastrutture, lavoro con Rust, PostgreSQL e piattaforme cloud. Realizzo servizi che sfruttano bene l'hardware sottostante, insieme al lavoro di deploy, monitoraggio e sicurezza che li mantiene in funzione.",
      sections: [
        {
          id: "introduction",
          heading: "Introduzione",
          blocks: [
            {
              kind: "paragraph",
              text: "Sono un ingegnere del software coreano che vive in Colorado. In **Soundpatrol** lavoro come Rust Engineer su cybersecurity, DevOps, servizi web e strumenti per sviluppatori, soprattutto con Rust, Python, PostgreSQL, Kubernetes e Google Cloud.",
            },
            {
              kind: "paragraph",
              text: "In precedenza ho guidato il lavoro su backend e infrastrutture di un digital twin di un edificio per Samsung C&T, ho contribuito ai backend delle applicazioni di Hyundai, Kia e Genesis e ho realizzato la raccolta dati per una piattaforma di analisi di YouTube. Dal 2023 il mio lavoro ruota attorno a servizi backend, pipeline di dati e all'infrastruttura che li sostiene.",
            },
            {
              kind: "paragraph",
              text: "Credo nel costruire servizi che sfruttino bene l'hardware e i linguaggi di programmazione moderni. In pratica significa progettare con cura gli schemi, usare code e cache limitate, gestire gli errori in modo esplicito e fare deploy che si possano ispezionare e ripristinare. Mi piace aiutare i colleghi a imparare Rust e voglio lavorare con persone che hanno cura del proprio mestiere.",
            },
          ],
        },
        {
          id: "results",
          heading: "Risultati scelti",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "Di spesa cloud mensile in meno per il digital twin di Samsung C&T, da circa $5.000 a $150",
                },
                {
                  value: "80",
                  count: 80,
                  label: "Endpoint REST realizzati per il digital twin, con logica di dominio dell'ingegneria edile",
                },
                {
                  value: "9 mesi",
                  count: 9,
                  suffix: " mesi",
                  label: "Di esercizio in produzione del digital twin senza alcun arresto a runtime",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "Di throughput in più sulle letture miste del backend di questo sito grazie a [PGO e BOLT](/about-blog/performance)",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Container di servizi collegati staticamente, ottenuti con build multi-stage",
                },
                {
                  value: "30.000",
                  label: "Righe scritte per il backend del digital twin, circa il 75% del suo codice",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Strumenti",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Linguaggi",
                  items: [
                    "Rust",
                    "Python",
                    "TypeScript",
                    "SQL",
                    "Java",
                    "C",
                    "C++",
                  ],
                },
                {
                  label: "Backend",
                  items: [
                    "Axum",
                    "Tokio",
                    "Diesel",
                    "tokio-postgres",
                    "Spring Boot",
                    "FastAPI",
                    "WebSockets",
                    "gRPC",
                    "Protocol Buffers",
                  ],
                },
                {
                  label: "Dati",
                  items: [
                    "PostgreSQL",
                    "Valkey / Redis",
                    "SQLite / Turso",
                    "MySQL",
                    "FAISS",
                    "tantivy",
                  ],
                },
                {
                  label: "Infrastruttura",
                  items: [
                    "Kubernetes",
                    "Helm",
                    "Docker",
                    "OpenTofu / Terraform",
                    "GitHub Actions",
                    "Google Cloud",
                    "AWS",
                    "Linux",
                  ],
                },
                {
                  label: "Affidabilità e sicurezza",
                  items: [
                    "OpenTelemetry",
                    "IAM",
                    "TLS",
                    "Network policy",
                    "Credential lifecycle",
                    "Incident response",
                  ],
                },
                {
                  label: "Interfacce e dispositivi",
                  items: [
                    "SolidJS",
                    "Leptos",
                    "Slint",
                    "WebAssembly",
                    "WebRTC",
                    "BACnet",
                    "Modbus",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "Altro su di me",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Esperienza",
                  href: "/about/experience",
                  meta: "2016 - oggi",
                  text: "I ruoli da Soundpatrol fino al mio primo stage, con i progetti e i risultati di ciascuno.",
                },
                {
                  title: "Progetti",
                  href: "/about/projects",
                  meta: "Rust · open source",
                  text: "Sillok, EU5 Location DB, questo sito e altri lavori indipendenti e open source.",
                },
                {
                  title: "Percorso",
                  href: "/about/background",
                  meta: "Studi · ricerca",
                  text: "Formazione, ricerca pubblicata, premi, lingue e lavori fuori dal software.",
                },
                {
                  title: "Come funziona questo sito",
                  href: "/about-blog",
                  meta: "Stack tecnologico",
                  text: "L'host, il backend, il frontend, i sottosistemi e le misure di prestazioni dietro cyhdev.com.",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "Contatti",
          blocks: [
            {
              kind: "list",
              items: [
                "**Email**: [younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**: [github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**: [linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "L'email è il modo più affidabile per raggiungermi. Se vuoi entrare nel server Minecraft, scrivilo lì.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "Info",
      title: "Esperienza",
      lede: "Lavoro su backend, infrastrutture e sicurezza in una startup statunitense, per clienti aziendali coreani e in uno stage iniziale, seguito dai lavori precedenti al software.",
      sections: [
        {
          id: "software",
          heading: "Ingegneria del software",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Rust Engineer",
                  period: "maggio 2026 - oggi",
                  location: "Stati Uniti · Da remoto",
                  summary: "Cybersecurity, DevOps, sviluppo di servizi web e strumenti per sviluppatori per un'azienda di rilevamento musicale e audio.",
                  highlights: [
                    "Mantengo l'infrastruttura Kubernetes su Google Cloud, l'infrastruttura come codice e flussi di deploy revisionati che usano credenziali di breve durata, limitate all'ambiente.",
                    "Pianifico ed eseguo migrazioni di data store in produzione, verificando lo stato migrato e provando il ripristino prima di ogni passaggio.",
                    "Unifico metriche, log e trace su OpenTelemetry e realizzo strumenti in Rust per stato dei servizi, diagnostica e visibilità dei costi.",
                    "Mi occupo della prontezza in materia di sicurezza: amministrazione degli accessi, rotazione e tracciamento della scadenza delle credenziali, policy, risposta agli incidenti ed evidenze per gli audit.",
                    "Realizzo servizi e strumenti interni in Rust e Python per i flussi di operazioni e supporto.",
                  ],
                  tags: [
                    "Rust",
                    "Python",
                    "PostgreSQL",
                    "Kubernetes",
                    "Google Cloud",
                    "OpenTofu",
                    "OpenTelemetry",
                  ],
                },
                {
                  organization: "GenesisNest",
                  title: "Software Engineer",
                  period: "gen 2025 - lug 2025",
                  location: "Seongnam, Corea del Sud",
                  summary: "Servizi backend e strumenti per l'infrastruttura di piattaforme interne e dell'applicazione ufficiale per i fan di un gruppo K-pop.",
                  highlights: [
                    "Ho contribuito con API, correzioni di bug e integrazioni ai backend delle applicazioni ufficiali di Hyundai, Kia e Genesis, lavorando su una grande base di codice Java e coordinandomi con i fornitori per servizi usati da decine di milioni di persone in Asia e in Europa.",
                    "Ho realizzato pipeline di internazionalizzazione in Rust e Python che trasformano e caricano contenuti tradotti nei database aziendali, coordinandomi con gli amministratori di database responsabili dei dataset collegati ai sistemi di fabbrica.",
                    "Ho sviluppato strumenti di gestione dell'infrastruttura per i servizi interni.",
                  ],
                  tags: [
                    "Java",
                    "Spring Boot",
                    "Rust",
                    "Python",
                  ],
                },
                {
                  organization: "pampam Inc.",
                  title: "Software Engineer a contratto",
                  period: "nov 2024 - dic 2024",
                  location: "Seul, Corea del Sud",
                  highlights: [
                    "Ho progettato lo schema PostgreSQL e il backend in Rust di una piattaforma di analisi di YouTube assistita dall'IA e ne ho guidato il deploy su AWS e Google Cloud.",
                    "Ho realizzato flussi di raccolta tramite l'API di YouTube che coprono decine di milioni di commenti e utenti, combinando dati dei canali, trascrizioni e contenuti delle pagine.",
                    "Ho integrato modelli linguistici in hosting e modelli open source sul server per generare riassunti e spunti di marketing.",
                  ],
                  tags: [
                    "Rust",
                    "PostgreSQL",
                    "AWS",
                    "Google Cloud",
                  ],
                },
                {
                  organization: "EAN Technology Co. Ltd.",
                  title: "Backend Software Engineer (Lead)",
                  period: "ago 2023 - ago 2024",
                  location: "Seul, Corea del Sud",
                  summary: "Ho guidato lo sviluppo di backend, database e infrastruttura cloud per due piattaforme su edifici ed energia.",
                  highlights: [],
                  projects: [
                    {
                      name: "Digital twin del Global Engineering Center di Samsung C&T",
                      highlights: [
                        "Ho sostituito un backend Express e un insieme di microservizi cloud con un monolite Rust/Axum ottimizzato, riducendo la spesa mensile per l'infrastruttura da circa **$5.000 a $150**.",
                        "Ho scritto circa **30.000 righe**, all'incirca il 75% del backend, tra cui **80 endpoint REST** con logica di dominio dell'ingegneria edile, integrazione di dati e operazioni in tempo reale.",
                        "Ho portato molti percorsi di richiesta con P99 di diversi secondi a una o due cifre di millisecondi su un deploy da due core e 4 GB, con Rust asincrono, multithreading, cache e ottimizzazione di schema e query PostgreSQL.",
                        "Ho integrato diverse migliaia di sensori, contatori e controllori di impianti HVAC e idrici tramite BACnet e Modbus, in loco presso la sede di Samsung C&T.",
                        "È rimasto in produzione per nove mesi senza alcun arresto a runtime, con una copertura della gestione degli errori di circa il 90%.",
                        "Ho costruito CI/CD con runner GitHub, AWS CodeDeploy e Docker; le build statiche, ottimizzate per il processore, hanno prodotto container portabili di circa 20 MB.",
                        "Ho collegato l'inferenza dei modelli predittivi tramite ECS, EC2, SQS, SNS, S3 e SageMaker, implementato i requisiti di sicurezza di Samsung SDS e integrato il backend con un'interfaccia Unreal Engine 4 realizzata da uno studio partner.",
                      ],
                    },
                    {
                      name: "Servizio di gestione energetica con IA",
                      highlights: [
                        "Ho progettato da zero lo schema PostgreSQL e l'API Rust/Axum e scritto circa **25.000 righe**, all'incirca il 70% del backend.",
                        "Ho scelto SQL diretto tramite tokio-postgres al posto di un ORM per avere controllo su query e tempi di compilazione, applicando quanto imparato con il digital twin.",
                        "Ho aiutato due ingegneri provenienti da Node.js e Java a diventare produttivi in Rust in circa un mese, in un team di nove persone tra ingegneri backend, frontend e data science.",
                      ],
                    },
                  ],
                  tags: [
                    "Rust",
                    "Axum",
                    "PostgreSQL",
                    "AWS",
                    "Docker",
                    "BACnet",
                    "Modbus",
                  ],
                },
                {
                  organization: "Artifyc Inc.",
                  title: "Stagista di ingegneria del software",
                  period: "ago 2022 - mar 2023",
                  location: "Austin, Texas · Da remoto",
                  highlights: [
                    "Ho realizzato un bot Discord in Python che usava l'API di Reddit per trovare richieste di commissioni artistiche e pubblicarle in un canale della community, eseguito ogni cinque minuti su AWS Lambda.",
                    "Ho implementato in Lambda la logica per conversione di valuta, addebiti e calcolo dei supplementi tra giurisdizioni diverse.",
                    "Ho contribuito con modifiche al frontend React su layout, risorse grafiche e testi.",
                  ],
                  tags: [
                    "Python",
                    "AWS Lambda",
                    "React",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "other",
          heading: "Prima del software",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Korea Institute of Maritime Science and Technology",
                  title: "Traduttore e interprete",
                  period: "giu - lug 2023, ago 2024",
                  location: "Seul, Corea del Sud · A contratto",
                  highlights: [
                    "Ho fatto da interprete in inglese, coreano e norvegese a una conferenza di ricerca marittima a cui hanno partecipato l'ambasciatore norvegese in Corea, un sottosegretario di Stato norvegese, i vertici dell'istituto e il presidente di Dongwon Group.",
                    "Ho tradotto discorsi e preparato il personale dell'istituto e il suo direttore per una conferenza internazionale e per un evento del Dipartimento dell'Energia degli Stati Uniti su un progetto congiunto.",
                  ],
                },
                {
                  organization: "Esercito della Repubblica di Corea · 2ª Divisione di fanteria dell'esercito statunitense",
                  title: "Sergente, addetto di collegamento e istruttore KATUSA",
                  period: "mar 2016 - dic 2017",
                  location: "Dongducheon, Corea del Sud",
                  highlights: [
                    "Ho prestato servizio nel Battaglione di comando della divisione tramite il programma [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army), in un ambiente congiunto coreano-americano.",
                    "Ho istruito e accolto circa 6.000 militari americani e coreani in corsi di preparazione fisica, cultura, storia e prevenzione.",
                    "Ho guidato una squadra di 13 persone e mantenuto la prontezza operativa durante la [crisi del 2017](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) nella penisola.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Addetto al carico logistico",
                  period: "2020 - 2022",
                  location: "Corea del Sud · Contratti occasionali",
                  highlights: [
                    "Lavoro di magazzino a turni notturni, insieme agli studi universitari.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "Info",
      title: "Progetti",
      lede: "Lavori indipendenti e open source, soprattutto in Rust. Le demo interattive si trovano nella [pagina dei progetti](/projects).",
      sections: [
        {
          id: "featured",
          heading: "In evidenza",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "Un registro di lavoro da riga di comando. Obiettivi, attività completate, correzioni e ritrattazioni vengono registrati come eventi append-only con proiezioni correnti indicizzate, e un archivio basato su Git unisce le cronologie tra più macchine.",
                  tags: [
                    "Event sourcing",
                    "Sync",
                  ],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Un'applicazione desktop e browser per i dati delle mappe di Europa Universalis V, con dataset compressi incorporati, indici di ricerca e ordinamento precalcolati, filtri per categoria e tabelle ridimensionabili. [Provala qui](/eu5-locations-db).",
                  tags: [
                    "Native + web",
                    "Indexes",
                  ],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Questo sito: pubblicazione, fotografia, un forum, chat dal vivo con chiamate, un esploratore del terreno di Minecraft e demo WebAssembly, serviti da un unico binario Rust. [Come è fatto](/about-blog).",
                  tags: [
                    "Monorepo",
                    "Self-hosted",
                  ],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "Un generatore e formattatore di UUIDv4 con output su terminale e su file, controllo opzionale dei duplicati e formattazione compatibile con Python e JSON; circa tre volte più veloce di uuidgen in un primo benchmark locale.",
                  tags: [
                    "Throughput",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "Altri lavori",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: una [correzione upstream](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) per il server Minecraft in Rust che preserva le impostazioni dell'overworld di Paper durante l'importazione e il salvataggio; in attesa di revisione.",
                "**Strumenti per le mappe di Minecraft**: un plugin Paper e servizi Rust alla base dell'[esploratore del terreno](/minecraft) di questo sito, tra cui un formato binario compatto per i tile dei biomi e previsioni dei biomi basate sul seed.",
                "**Sincronizzazione del filesystem**: un'utility Rust per la sincronizzazione bidirezionale di cartelle e la pulizia degli artefatti di build tra filesystem eterogenei, con dry run, journal delle operazioni, checksum opzionali e protezioni contro cancellazioni massicce involontarie.",
                "**impulsr**: uno strumento di trascrizione e raccolta dei commenti di YouTube che produce riassunti generati da modelli linguistici per il lavoro di marketing.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Come lavoro",
          blocks: [
            {
              kind: "paragraph",
              text: "Preferisco sistemi piccoli e ispezionabili: un solo binario dove un solo binario basta, un limite esplicito per ogni coda e cache, e formati di dati scelti in base al pattern di accesso e non per abitudine.",
            },
            {
              kind: "paragraph",
              text: "Le misurazioni vengono prima delle affermazioni; le [note sulle prestazioni](/about-blog/performance) di questo sito mostrano come vengono effettuate.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "Info",
      title: "Percorso",
      lede: "Formazione, ricerca, qualifiche e interessi al di fuori del software.",
      sections: [
        {
          id: "education",
          heading: "Formazione",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Sungkyunkwan University",
                  title: "Laurea in Ingegneria del software (B.Eng.)",
                  period: "mar 2015 - ago 2023",
                  location: "Suwon, Corea del Sud",
                  summary: "Dipartimento di Informatica e Ingegneria. Il percorso di laurea comprende il servizio militare obbligatorio.",
                  highlights: [
                    "Corsi di software embedded, circuiti logici, sistemi operativi, reti, programmazione a basso livello ed elettronica, accanto alla pratica dell'ingegneria del software.",
                    "La Sungkyunkwan University fa risalire le proprie origini al 1398, come accademia reale della dinastia Joseon, ed è una delle principali università di ricerca della Corea.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Ricerca",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Analisi emotiva delle community sui social media basata su un modello NLP per la lingua coreana.** Ho applicato KoBERT all'analisi del sentimento e alla categorizzazione di contenuti social in coreano, esaminando il cambiamento emotivo nelle prime fasi del COVID-19.",
            },
            {
              kind: "paragraph",
              text: "Il progetto ha ricevuto il Premio di bronzo nell'ambito dei premi per tesi e progetti di laurea dell'università, tra circa 60 candidature.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Premi e qualifiche",
          blocks: [
            {
              kind: "table",
              columns: [
                "Anno",
                "Riconoscimento",
                "Dettaglio",
              ],
              rows: [
                [
                  "2023",
                  "**Premio di bronzo**",
                  "Premi per tesi e progetti di laurea della Sungkyunkwan University",
                ],
                [
                  "2023",
                  "**IELTS 8.5 / 9.0**",
                  "CEFR C2",
                ],
                [
                  "2021",
                  "**7º posto**",
                  "Hackathon di Capstone Design e idee, Korean Society for Internet Information",
                ],
                [
                  "2020",
                  "**3º posto**",
                  "Hackathon Fintech, Seoul National University of Science and Technology",
                ],
                [
                  "2018",
                  "**TOEFL 117 / 120**",
                  "CEFR C2",
                ],
              ],
            },
          ],
        },
        {
          id: "languages",
          heading: "Lingue",
          blocks: [
            {
              kind: "list",
              items: [
                "**Coreano**: madrelingua.",
                "**Inglese**: padronanza professionale (C2).",
                "**Francese, tedesco, spagnolo e mandarino**: conoscenza di base.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "Fuori dal software",
          blocks: [
            {
              kind: "list",
              items: [
                "**Fotografia**: fotografia di paesaggio e ritratto dal 2010, con circa 30.000 scatti archiviati. Una selezione è nella [galleria](/photographs).",
                "**Giornalismo**: giornalista studentesco per il Sungkyun Times, la rivista in lingua inglese dell'università, su tecnologia, sicurezza e affari internazionali.",
                "**Insegnamento e volontariato**: ho insegnato inglese in un centro comunitario e ho fatto volontariato in un negozio dell'usato gestito da persone con disabilità.",
                "**Traduzione**: interpretariato e traduzione per enti pubblici; vedi [Esperienza](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
