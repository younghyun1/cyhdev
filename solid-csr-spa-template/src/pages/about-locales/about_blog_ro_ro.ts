import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "noiembrie 2026",
  chrome: {
    overview: "Prezentare generală",
    updated: "Ultima actualizare",
    onThisPage: "Pe această pagină",
    sectionNavigation: "Pagini despre stack-ul tehnic",
    previous: "Anterior",
    next: "Următor",
    readingTime: "{minutes} min de citit",
  },
  pages: {
    overview: {
      eyebrow: "Despre acest site",
      title: "Stack-ul tehnic al blogului",
      lede: "Un singur binar Rust, o singură bază de date PostgreSQL, o singură mașină mică. Aceste pagini descriu cum este construit, găzduit, securizat și măsurat site-ul și de ce are această formă.",
      sections: [
        {
          id: "at-a-glance",
          heading: "Pe scurt",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53.889 req/s",
                  label: "Debit median al citirilor publice mixte, cu PGO și BOLT, pe patru nuclee de server",
                },
                {
                  value: "~150 µs",
                  label: "Durata observată a unui drum dus-întors către baza de date printr-un socket UNIX, față de aproximativ 600 µs prin TCP pe localhost",
                },
                {
                  value: "0",
                  label: "Procese Node.js în producție; frontend-ul este încorporat în binarul serverului",
                },
                {
                  value: "24",
                  count: 24,
                  label: "Limbi ale interfeței, fiecare descărcată doar când este selectată",
                },
                {
                  value: "4.096",
                  label: "Conexiuni deschise admise înainte de TLS, cu cel mult 64 per adresă sau per IPv6 /64",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Proces care servește TLS, API-ul, WebSockets, fluxurile media WebRTC și frontend-ul încorporat",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Calea unei cereri",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Browser",
                  detail: "HTTP/2 sau HTTP/1.1 peste TLS. Frontend-ul este păstrat în cache sub URL-uri cu amprentă, imuabile.",
                },
                {
                  label: "Admitere",
                  detail: "Conexiunile TCP sunt admise înainte de TLS: 4.096 în total și 64 per adresă sau per IPv6 /64.",
                },
                {
                  label: "rustls",
                  detail: "TLS se termină în interiorul procesului Rust, cu `TCP_NODELAY` pe fiecare socket.",
                },
                {
                  label: "Axum",
                  detail: "Antete de securitate, sesiuni, limite de rată, compresie și limite de dimensiune a corpului, apoi un handler de funcționalitate.",
                },
                {
                  label: "Cache · Diesel",
                  detail: "Cache-uri în memorie, cu dimensiune limitată, răspund la citirile frecvente; tot restul trece prin Diesel asincron.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "Pe aceeași gazdă, printr-un socket UNIX, cu I/O asincron io_uring.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Capitole",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Infrastructură",
                  href: "/about-blog/infrastructure",
                  meta: "Gazdă · Gentoo · rețea",
                  text: "Mașina, sistemul de operare și sistemul de fișiere, rețeaua și motivul pentru care nu există un reverse proxy.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "Stack-ul serviciului, modelul de date, limitele de execuție, antetele de securitate și contractele API.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "Structura aplicației, încărcarea, localizarea, sistemul de design și animațiile.",
                },
                {
                  title: "Subsisteme",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "Chat live cu un SFU în același proces, exploratorul Minecraft, interogări Geo-IP și WebAssembly.",
                },
                {
                  title: "Performanță",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Compilări optimizate, cum au fost măsurate și rezultatele.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Cum sunt verificate modificările",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy pe ținte native și WebAssembly, ca poartă de verificare la implementare.",
                "Verificări locale cargo xtask pentru formatare, teste unitare native, divergența contractului OpenAPI și, pentru frontend, verificarea tipurilor, lint, teste unitare și compilare.",
                "Playwright pe Chromium și WebKit, cu verificări de accesibilitate axe și verificări de reorganizare a paginii pe mobil până la 320 de pixeli CSS.",
                "Teste de integrare și de revenire a migrărilor pe o instanță PostgreSQL de unică folosință, plus o scanare locală a secretelor cu gitleaks.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Stack tehnic",
      title: "Infrastructură",
      lede: "O mașină mică acasă, o distribuție Linux construită din surse și niciun strat de care traficul să nu aibă nevoie.",
      sections: [
        {
          id: "host",
          heading: "Gazda",
          blocks: [
            {
              kind: "paragraph",
              text: "Site-ul rulează pe un [miniserver](https://store.minisforum.com/products/minisforum-um690l-slim) la reședința mea: un procesor mobil Ryzen 9 cu opt nuclee, care ajunge la 4,9 GHz, 32 GB de RAM și un SSD NVMe de 1 TB. La aproximativ $400, costă mai puțin decât un an de hardware cloud sensibil mai slab și reflectă entuziasmul meu pentru autogăzduire.",
            },
            {
              kind: "paragraph",
              text: "Pe el rulează serverul combinat de backend și frontend, PostgreSQL și un server Minecraft. Dacă doriți să jucați, scrieți-mi un e-mail.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo și btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) nu este alegerea evidentă pentru un server; la serviciu aș alege Debian stable pe ext4. Aici îmi place să compilez singur pachetele, cu optimizări pentru procesorul exact.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) oferă instantanee, compresie și copy-on-write, dar copy-on-write fragmentează puternic fișierele bazelor de date. De aceea, directoarele de date PostgreSQL și Minecraft sunt excluse. Un [benchmark al sistemelor de fișiere](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) arată btrfs cu setări implicite în urma ext4 și XFS; bănuiesc că dezactivarea copy-on-write acoperă cea mai mare parte din diferență, ceea ce ar fi o măsurătoare interesantă.",
            },
          ],
        },
        {
          id: "network",
          heading: "Rețea",
          blocks: [
            {
              kind: "paragraph",
              text: "Traficul sosește printr-o conexiune Xfinity prin cablu, de 1 Gbps; modemul furnizorului nu are Ethernet de 2,5 Gb, lucru pe care acest site nu îl va observa. Route 53 găzduiește DNS-ul.",
            },
            {
              kind: "paragraph",
              text: "În interiorul mașinii nu există reverse proxy, runtime de containere sau service mesh. PostgreSQL rulează direct pe sistemul de operare, iar un singur binar Rust servește frontend-ul și API-ul. Cererile HTTP simple sunt redirecționate către HTTPS de același proces.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "Socketuri UNIX",
          blocks: [
            {
              kind: "paragraph",
              text: "Serverul ajunge la PostgreSQL printr-un socket UNIX, nu prin TCP pe localhost, ceea ce ocolește complet stiva de rețea. [Benchmark-urile](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) arată rate de tranzacții aproape dublate; aici am observat durate de drum dus-întors de până la **150 µs**, față de aproximativ **600 µs** prin localhost. Este o soluție veche și irelevantă pentru majoritatea implementărilor în cloud, dar este mai rapidă.",
            },
            {
              kind: "paragraph",
              text: "PostgreSQL 18, lansat în septembrie 2025, a adăugat [I/O asincron](https://pganalyze.com/blog/postgres-18-async-io), iar aici io_uring este activat, ceea ce accelerează sensibil citirile.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Compilare și implementare",
          blocks: [
            {
              kind: "list",
              items: [
                "Compilarea frontend-ului este încorporată în binarul backend-ului, împreună cu copii precomprimate zstd și gzip, așa că în producție nu rulează niciun proces Node.js.",
                "Compilările optimizate pot fi antrenate cu optimizare ghidată de profil și optimizate după link cu BOLT; vedeți [Performanță](/about-blog/performance).",
                "Credențialele, certificatele și datele de rulare nu ajung în Git, iar o scanare gitleaks verifică arborele înainte ca modificările să părăsească mașina.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Stack tehnic",
      title: "Backend",
      lede: "Rust pe Axum și Tokio, PostgreSQL prin Diesel asincron și o limită explicită pentru tot ce poate ține deschis un client.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Execuție",
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
                  label: "Date",
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
                  label: "Integrări",
                  items: [
                    "openidconnect",
                    "lettre",
                    "Amazon S3",
                    "webrtc-rs",
                    "image + AVIF",
                  ],
                },
                {
                  label: "Contracte",
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
              text: "Serviciul este un singur proces cu stare partajată: sesiuni în memorie, sincronizarea cache-ului la pornire și sarcini de întreținere programate. mimalloc este alocatorul global; dacă o mașină are nuclee reale, comportamentul alocatorului merită luat în serios.",
            },
          ],
        },
        {
          id: "data",
          heading: "Modelul de date",
          blocks: [
            {
              kind: "paragraph",
              text: "În lucrul din startup-urile și corporațiile coreene pe care l-am văzut, MySQL și MariaDB erau considerate singurele baze de date relaționale serioase. PostgreSQL le-a depășit de multă vreme în domeniile care mă interesează, de la UUID-uri native la JSON binar.",
            },
            {
              kind: "list",
              items: [
                "**Articolele** au slug-uri, rezumate, metadate JSONB, stare de publicare, contoare denormalizate și asocieri cu etichete. Comentariile sunt organizate în fire printr-un părinte nullable, iar voturile pentru articole și comentarii au tabele separate.",
                "**Conturile** păstrează țara și limba, tabele de roluri și permisiuni, iar pozele de profil versionate sunt într-un tabel propriu, nu pe rândul utilizatorului accesat frecvent.",
                "**Cheile** sunt valori UUIDv7 ordonate în timp: nu pot fi enumerate ca ID-urile secvențiale și sunt mult mai prietenoase cu localitatea B-tree decât UUIDv4 aleatoriu.",
                "**Căutarea** folosește un index tantivy persistat pe disc, peste titlurile și etichetele articolelor, menținut coerent cu PostgreSQL, care rămâne sursa autoritară.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Limite de execuție",
          blocks: [
            {
              kind: "list",
              items: [
                "Conexiunile sunt admise înainte de TLS printr-un limitator partajat: 4.096 de conexiuni deschise și 64 per adresă IPv4 sau per IPv6 /64. Socketurile refuzate se închid fără handshake.",
                "Antetele cererilor HTTP/1 trebuie să sosească în 20 de secunde; conexiunile HTTP/2 primesc ping la fiecare 30 de secunde și permit 128 de fluxuri simultane.",
                "Fiecare cache are o limită fixă și o sursă autoritară din care poate citi direct; niciunul nu crește odată cu chei controlate de client. De exemplu, cache-ul pachetelor WebAssembly păstrează cel mult 256 MiB.",
                "Operațiile blocante, precum codificarea imaginilor, rulează în grupuri limitate de fire de lucru, nu pe workerii Tokio.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Antete de securitate",
          blocks: [
            {
              kind: "paragraph",
              text: "Content-Security-Policy este construit o singură dată, la pornire. Scripturile inline sunt permise doar prin hash-uri SHA-256 calculate din `index.html` compilat în binar, astfel încât scriptul temei, rulat înainte de randare, nu poate diverge de politică. Originea principală nu instanțiază niciodată WebAssembly: demonstrațiile și aplicația EU5 rulează în cadre izolate (sandbox), sub propriile politici.",
            },
            {
              kind: "paragraph",
              text: "Cookie-urile de sesiune sunt `Secure` și `HttpOnly`, operațiile de superutilizator verifică rolurile pe server, conturile pot asocia identități OpenID Connect, iar Swagger UI cere autentificare în producție.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Contracte și resurse",
          blocks: [
            {
              kind: "list",
              items: [
                "Handlerele poartă adnotări OpenAPI, clienții TypeScript ai frontend-ului sunt generați din ele, iar cargo xtask openapi eșuează atunci când fișierele generate diverg.",
                "Manifestul Vite este încorporat împreună cu resursele. Doar fișierele cu amprentă, listate în manifest, primesc o politică de cache `immutable` de un an; HTML-ul se revalidează întotdeauna.",
                "Căile către resurse lipsă returnează 404 cu `no-store`, nu structura aplicației, astfel încât o pagină veche să nu poată păstra HTML în cache sub un URL de script.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Stack tehnic",
      title: "Frontend",
      lede: "O aplicație SolidJS 2 randată în client, compilată în resurse statice și încorporată în binarul serverului.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Aplicație",
                  items: [
                    "SolidJS 2",
                    "Solid Router 2",
                    "TypeScript",
                    "Vite 8",
                  ],
                },
                {
                  label: "Stilizare",
                  items: [
                    "Tailwind CSS 4",
                    "IBM Plex Sans",
                    "IBM Plex Mono",
                    "OKLab",
                  ],
                },
                {
                  label: "Componente",
                  items: [
                    "Toast UI Editor",
                    "Leaflet",
                    "Chart.js",
                    "highlight.js",
                    "DOMPurify",
                  ],
                },
                {
                  label: "Testare",
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
              text: "Semnalele cu granularitate fină din Solid actualizează doar nodurile DOM care depind de valorile modificate, iar versiunile sale candidate 2.0 permit citirilor asincrone să suspende randarea până la cea mai apropiată limită de încărcare. Routerul încarcă leneș fiecare pagină, iar în producție nu există un server JavaScript: binarul Rust servește fișierele statice.",
            },
          ],
        },
        {
          id: "loading",
          heading: "Încărcare",
          blocks: [
            {
              kind: "list",
              items: [
                "Paginile, panoul de chat de pe pagina principală și shaderul ei de contururi sunt fragmente separate, iar procesul de compilare raportează dimensiunea gzip a ceea ce se încarcă primul.",
                "Doar textul interfeței în engleză este livrat în fragmentul inițial. Celelalte limbi, precum și paginile Despre ale fiecărei limbi, sunt fragmente separate, încărcate la selectare.",
                "Testele de performanță pe mobil, cu rețea limitată, urmăresc un LCP de cel mult 2,5 s, interacțiuni de cel mult 200 ms și un CLS de cel mult 0,1.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Localizare",
          blocks: [
            {
              kind: "paragraph",
              text: "Interfața acceptă 24 de limbi, inclusiv aranjamente de la dreapta la stânga pentru arabă, persană și urdu. Un catalog de aproximativ 740 de chei este validat în raport cu registrul backendului, iar substituenții trebuie să supraviețuiască traducerii. Aceste pagini Despre sunt module de date tipizate, astfel încât o traducere nu poate omite pe tăcute o secțiune pe care o are engleza.",
            },
          ],
        },
        {
          id: "design",
          heading: "Sistem de design",
          blocks: [
            {
              kind: "paragraph",
              text: "Aspectul este cald și tehnic: hârtie crem sau negru pur, borduri subțiri, font monospațiat pentru tot ce este tehnic și o singură culoare de accent la un moment dat. Culorile sunt variabile CSS semantice, așa că componentele nu au nevoie de variante pentru modul întunecat. Presetările de accent sunt testate pentru contrast față de fundalurile paginii, culoarea RGB proprie a unui vizitator este ajustată în fiecare mod pentru a rămâne lizibilă, iar modul luminos sau întunecat și accentul sunt aplicate înainte de prima afișare.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Mișcare",
          blocks: [
            {
              kind: "list",
              items: [
                "Dezvăluirea secțiunilor și bara de progres a lecturii folosesc animații CSS conduse de derulare, care rulează în afara firului principal acolo unde sunt acceptate.",
                "Statisticile numără crescător printr-un întreg `@property` înregistrat, iar valoarea finală este prezentă în cod pentru tehnologiile asistive.",
                "Comutarea temei se extinde din buton prin API-ul View Transitions.",
                "Câmpul de contururi de pe pagina principală este un shader WebGL mic, care se încarcă după ce pagina este inactivă, rulează doar în browsere de desktop capabile, își limitează rata de cadre și se oprește când este în afara ecranului.",
                "O setare a site-ului dezactivează toate acestea. Animațiile sunt activate implicit și nu urmează setarea de mișcare redusă a sistemului de operare.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Stack tehnic",
      title: "Subsisteme",
      lede: "Părțile site-ului care sunt mai interesante decât are strict nevoie un blog.",
      sections: [
        {
          id: "live-chat",
          heading: "Chat live și apeluri",
          blocks: [
            {
              kind: "paragraph",
              text: "Chatul rulează peste un WebSocket, cu un codec binar compact, moderare și istoric limitat. Apelurile vocale și video trec printr-un Selective Forwarding Unit care rulează în procesul Axum, pe webrtc-rs: fiecare browser deține o singură conexiune peer, iar SFU retransmite media codificată fără transcodare, prin canale limitate pentru fiecare publicare și o limită de participanți. [Deschideți chatul live](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Explorator de teren Minecraft",
          blocks: [
            {
              kind: "paragraph",
              text: "Un plugin Paper expune un socket UNIX pentru același utilizator, destinat interogărilor limitate ale lumii, cu o singură operație la un moment dat și limite stricte pentru dimensiunea cadrelor. [Exploratorul](/minecraft) desenează dale squaremap cu propriul client Leaflet, în loc să ruleze JavaScript-ul hărții generate.",
            },
            {
              kind: "paragraph",
              text: "Suprapunerile de biomuri folosesc **CYBM**, un format binar compact de dale servit comprimat cu gzip, sau un PNG al cărui fragment privat `cyBM` conține identitățile exacte ale biomurilor, astfel încât trecerea cursorului să nu ghicească niciodată după culorile pixelilor. Predicțiile bazate pe seed pentru terenul neexplorat provin de la un worker separat de generare a lumii, construit pe Pumpkin.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Căutări Geo-IP",
          blocks: [
            {
              kind: "paragraph",
              text: "[Panoul de vizitatori](/visitor-board) și [căutarea IP](/geo-ip-db) localizează adrese fără a apela un API extern. Un instrument de preprocesare transformă fișierele CSV IP2Location LITE în `BTreeMap`-uri indexate după începutul intervalului, le codifică cu bitcode și le comprimă cu zstd la nivelul 22.",
            },
            {
              kind: "table",
              columns: [
                "Set de date",
                "CSV",
                "Pachet",
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
              text: "La pornire, serverul decodifică un pachet pe rând pentru a limita memoria maximă și internalizează numele de locuri repetate. O căutare găsește ultimul interval care începe la adresă sau sub ea și îi verifică limita de sfârșit.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "[Pachetele de demonstrații](/projects) încărcate sunt pregătite pe disc, identificate după octeții lor magici, limitate la 50 MiB după decomprimare și servite dintr-un cache limitat, în cadre izolate. Workspace-ul construiește și demonstrații Rust, printre care un block breaker și un ray tracer.",
            },
            {
              kind: "paragraph",
              text: "[EU5 Location DB](/eu5-locations-db) este o aplicație Slint compilată în WebAssembly. Pagina îi transmite modul luminos sau întunecat printr-un mic protocol de mesaje, astfel încât paleta încorporată se schimbă odată cu site-ul.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Pipeline foto",
          blocks: [
            {
              kind: "paragraph",
              text: "Încărcările sunt decodificate în limite fixe de pixeli și codificate în AVIF cu cea mai lentă presetare, sacrificând timp de procesare pentru fișiere mai mici. Locațiile din datele EXIF pot fi confirmate pe o hartă înainte de publicare, iar cel mult două lucrări de procesare rulează simultan.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Stack tehnic",
      title: "Performanță",
      lede: "Optimizare ghidată de profil și BOLT pe backend, măsurate cu un generator de sarcină care nu este el însuși blocajul.",
      sections: [
        {
          id: "approach",
          heading: "Abordare",
          blocks: [
            {
              kind: "paragraph",
              text: "Compilările optimizate sunt antrenate pe o sarcină reprezentativă. Un binar instrumentat înregistrează profile pe un mediu de test de unică folosință, cu endpointuri sintetice pentru PostgreSQL, SMTP, S3, OpenID Connect și Minecraft. LLVM recompilează serverul cu aceste profile (PGO), iar BOLT reordonează apoi dispunerea codului din binarul final pe baza execuției eșantionate. Toate variantele folosesc LTO, frame pointers și o bibliotecă standard recompilată.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Configurare",
          blocks: [
            {
              kind: "list",
              items: [
                "Serverul, furnizorii din mediul de test și PostgreSQL sunt fixați pe patru nuclee ale unui AMD Ryzen 7 9800X3D; clientul de încărcare este fixat pe celelalte patru.",
                "HTTPS peste HTTP/1.1 cu conexiuni reutilizate, 32 de conexiuni, citirea completă a răspunsurilor și termene de două secunde.",
                "Trei runde alternate, cu cinci eșantioane pe variantă, fiecare după 10.000 de cereri de încălzire, pornind de la un instantaneu proaspăt al bazei de date.",
                "27.000.000 de cereri cronometrate în total; toate au returnat HTTP 200.",
                "Sarcina provine de la oha; Playwright verifică răspunsul fiecărei rute în afara ferestrei cronometrate.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Rezultate",
          blocks: [
            {
              kind: "bars",
              caption: "Mediana cererilor pe secundă pentru 15 eșantioane pe variantă.",
              groups: [
                {
                  label: "Citiri publice pe zece rute",
                  items: [
                    {
                      label: "Referință",
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
                  label: "JSON mic de stare",
                  items: [
                    {
                      label: "Referință",
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
              caption: "Latența la coadă pentru amestecul de zece rute.",
              columns: [
                "Variantă",
                "p95 (ms)",
                "p99 (ms)",
              ],
              numericColumns: [
                1,
                2,
              ],
              rows: [
                [
                  "Referință",
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
              text: "PGO a crescut debitul mixt cu **17,9%**, iar PGO cu BOLT cu **20,1%**, și ambele compilări optimizate au câștigat fiecare rundă alternată. BOLT a adăugat 1,8% peste PGO la citirile mixte și a fost cu 0,8% mai lent pe ruta banală de stare. Acestea sunt rezultate pe patru nuclee; capacitatea de producție depinde și de rețea, de setul de lucru al bazei de date și de traficul real.",
            },
          ],
        },
        {
          id: "corrections",
          heading: "Corecții",
          blocks: [
            {
              kind: "list",
              items: [
                "`TCP_NODELAY` nu era setat la început: acceptorul Rustls implicit din axum-server îl lasă dezactivat, așa că răspunsurile HTTP/2 puteau aștepta circa 40 ms după o confirmare întârziată. Acum fiecare socket acceptat îl setează.",
                "Primul benchmark genera sarcina din Playwright, al cărui client încetinea pe parcursul unei rulări și arăta în mod greșit regresii. Acele cifre au fost eliminate.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
