import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "novembar 2026",
  chrome: {
    overview: "Pregled",
    updated: "Posljednje ažuriranje",
    onThisPage: "Na ovoj stranici",
    sectionNavigation: "Stranice o tehnološkom steku",
    previous: "Prethodno",
    next: "Sljedeće",
    readingTime: "{minutes} min čitanja",
  },
  pages: {
    overview: {
      eyebrow: "O ovom sajtu",
      title: "Tehnološki stek bloga",
      lede: "Jedna Rust izvršna datoteka, jedna PostgreSQL baza podataka, jedan mali računar. Ove stranice opisuju kako je sajt izgrađen, postavljen, zaštićen i izmjeren, i zašto ima baš takav oblik.",
      sections: [
        {
          id: "at-a-glance",
          heading: "Ukratko",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53.889 zahtjeva/s",
                  label: "Medijan propusnosti za mješovito javno čitanje uz PGO i BOLT na četiri jezgre servera",
                },
                {
                  value: "~150 µs",
                  label: "Uočeno vrijeme odziva baze podataka preko UNIX soketa, naspram ~600 µs preko localhost TCP-a",
                },
                {
                  value: "0",
                  label: "Node.js procesa u produkciji; frontend je ugrađen u izvršnu datoteku servera",
                },
                {
                  value: "24",
                  count: 24,
                  label: "Jezika sučelja, a svaki se preuzima tek kad se odabere",
                },
                {
                  value: "4.096",
                  label: "Otvorenih veza prihvaćenih prije TLS-a, najviše 64 po adresi ili IPv6 /64",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Proces koji poslužuje TLS, API, WebSocket, WebRTC medije i ugrađeni frontend",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Putanja zahtjeva",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Preglednik",
                  detail: "HTTP/2 ili HTTP/1.1 preko TLS-a. Frontend se kešira pod nepromjenjivim URL-ovima s otiskom sadržaja.",
                },
                {
                  label: "Prihvaćanje",
                  detail: "TCP veze se prihvaćaju prije TLS-a: 4.096 ukupno i 64 po adresi ili IPv6 /64.",
                },
                {
                  label: "rustls",
                  detail: "TLS se završava unutar Rust procesa, uz `TCP_NODELAY` na svakom soketu.",
                },
                {
                  label: "Axum",
                  detail: "Sigurnosna zaglavlja, sesije, ograničenja učestalosti, kompresija i ograničenja veličine tijela, zatim obrađivač funkcije.",
                },
                {
                  label: "Keš · Diesel",
                  detail: "Ograničeni keševi u memoriji odgovaraju na česta čitanja; sve ostalo ide kroz asinhroni Diesel.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "Na istom hostu preko UNIX soketa, uz asinhroni I/O putem io_uring-a.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Poglavlja",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Infrastruktura",
                  href: "/about-blog/infrastructure",
                  meta: "Host · Gentoo · mreža",
                  text: "Računar, operativni sistem i datotečni sistem, mreža i zašto nema reverznog proksija.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "Servisni stek, model podataka, ograničenja u izvršavanju, sigurnosna zaglavlja i API ugovori.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "Ljuska aplikacije, učitavanje, lokalizacija, dizajnerski sistem i animacije.",
                },
                {
                  title: "Podsistemi",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "Chat uživo sa SFU-om unutar procesa, Minecraft istraživač, Geo-IP pretrage i WebAssembly.",
                },
                {
                  title: "Performanse",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Optimizirane izgradnje, kako su mjerene i rezultati.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Kako se provjeravaju izmjene",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy na nativnim i WebAssembly ciljevima kao obavezna provjera pri implementaciji.",
                "Lokalne cargo xtask provjere formatiranja, nativnih jediničnih testova, odstupanja OpenAPI ugovora te frontenda: provjera tipova, lint, jedinični testovi i izgradnja.",
                "Playwright na Chromiumu i WebKitu uz axe provjere pristupačnosti i provjere prelamanja na mobilnim uređajima do 320 CSS piksela.",
                "Integracijski testovi i testovi vraćanja migracija na privremenoj PostgreSQL instanci, uz lokalno gitleaks skeniranje tajni.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Tehnološki stek",
      title: "Infrastruktura",
      lede: "Mali računar kod kuće, Linux distribucija zasnovana na izvornom kodu i nijedan sloj koji mrežni promet ne treba.",
      sections: [
        {
          id: "host",
          heading: "Host",
          blocks: [
            {
              kind: "paragraph",
              text: "Sajt radi na [miniserveru](https://store.minisforum.com/products/minisforum-um690l-slim) u mom domu: osmojezgreni Ryzen 9 mobilni procesor koji ubrzava do 4,9 GHz, 32 GB RAM-a i NVMe SSD od 1 TB. Uz cijenu od oko $400 košta manje od godine dana osjetno slabijeg hardvera u oblaku, a odražava i moj entuzijazam za samostalni hosting.",
            },
            {
              kind: "paragraph",
              text: "Na njemu rade kombinirani backend i frontend server, PostgreSQL i Minecraft server. Ako želite igrati, pošaljite mi e-poštu.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo i btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) nije očigledan izbor za server; na poslu bih odabrao stabilni Debian na ext4. Ovdje uživam u samostalnom kompajliranju paketa s optimizacijama za upravo ovaj procesor.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) donosi snimke stanja, kompresiju i copy-on-write, ali copy-on-write jako fragmentira datoteke baze podataka. Zato su direktoriji s podacima za PostgreSQL i Minecraft izuzeti iz njega. [Mjerenje datotečnih sistema](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) pokazuje da zadani btrfs zaostaje za ext4 i XFS; pretpostavljam da isključivanje copy-on-write zatvara većinu tog jaza, što bi bilo zanimljivo izmjeriti.",
            },
          ],
        },
        {
          id: "network",
          heading: "Mreža",
          blocks: [
            {
              kind: "paragraph",
              text: "Mrežni promet stiže preko žičane Xfinity veze od 1 Gbps; modem provajdera nema 2.5 Gb Ethernet, što ovaj sajt neće osjetiti. DNS vodi Route 53.",
            },
            {
              kind: "paragraph",
              text: "Unutar računara nema reverznog proksija, okruženja za kontejnere ni service mesha. PostgreSQL radi direktno na operativnom sistemu, a jedna Rust izvršna datoteka poslužuje frontend i API. Obične HTTP zahtjeve isti proces preusmjerava na HTTPS.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "UNIX soketi",
          blocks: [
            {
              kind: "paragraph",
              text: "Server pristupa PostgreSQL-u preko UNIX soketa umjesto localhost TCP-a, čime se mrežni stek potpuno preskače. [Mjerenja](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) pokazuju da se broj transakcija gotovo udvostručuje; ovdje sam uočio vrijeme odziva i do **150 µs**, naspram oko **600 µs** preko localhosta. To je staromodno i nebitno za većinu isporuka u oblaku, a brže je.",
            },
            {
              kind: "paragraph",
              text: "PostgreSQL 18, objavljen u septembru 2025, dodao je [asinhroni I/O](https://pganalyze.com/blog/postgres-18-async-io), a ovdje je uključen io_uring, što osjetno ubrzava čitanja.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Izgradnja i isporuka",
          blocks: [
            {
              kind: "list",
              items: [
                "Izgradnja frontenda ugrađena je u backend izvršnu datoteku zajedno s unaprijed komprimiranim zstd i gzip kopijama, pa se u produkciji ne pokreće nijedan Node.js proces.",
                "Optimizirane izgradnje mogu se trenirati optimizacijom vođenom profilom, a nakon povezivanja optimizirati BOLT-om; vidi [Performanse](/about-blog/performance).",
                "Pristupni podaci, certifikati i podaci iz izvršavanja ostaju izvan Gita, a gitleaks skeniranje provjerava stablo prije nego što izmjene napuste računar.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Tehnološki stek",
      title: "Backend",
      lede: "Rust na Axumu i Tokiju, PostgreSQL preko asinhronog Diesela i izričito ograničenje svega što klijent može držati otvorenim.",
      sections: [
        {
          id: "stack",
          heading: "Stek",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Izvršno okruženje",
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
                  label: "Podaci",
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
                  label: "Integracije",
                  items: [
                    "openidconnect",
                    "lettre",
                    "Amazon S3",
                    "webrtc-rs",
                    "image + AVIF",
                  ],
                },
                {
                  label: "Ugovori",
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
              text: "Servis je jedan proces sa zajedničkim stanjem: sesije u memoriji, usklađivanje keša pri pokretanju i planirani poslovi održavanja. mimalloc je globalni alokator; ako računar ima prave jezgre, ponašanje alokatora vrijedi shvatiti ozbiljno.",
            },
          ],
        },
        {
          id: "data",
          heading: "Model podataka",
          blocks: [
            {
              kind: "paragraph",
              text: "U korejskim startupima i korporacijama koje sam vidio MySQL i MariaDB smatrani su jedinim ozbiljnim relacijskim bazama podataka. PostgreSQL ih je odavno prestigao u oblastima do kojih mi je stalo, od izvornih UUID-ova do binarnog JSON-a.",
            },
            {
              kind: "list",
              items: [
                "**Objave** imaju slugove, sažetke, JSONB metapodatke, stanje objavljivanja, denormalizirane brojače i veze s oznakama. Komentari su ugniježđeni preko nadređenog komentara koji može izostati, a glasovi za objave i komentare imaju zasebne tabele.",
                "**Nalozi** čuvaju državu i jezik, tabele uloga i dozvola te verzionirane profilne slike u zasebnoj tabeli, a ne u često čitanom redu korisnika.",
                "**Ključevi** su vremenski uređene UUIDv7 vrijednosti: ne mogu se nabrojati kao uzastopni ID-ovi i mnogo su povoljnije za lokalnost B-stabla od nasumičnih UUIDv4.",
                "**Pretraga** koristi tantivy indeks trajno pohranjen na disku, nad naslovima objava i oznakama, usklađen s PostgreSQL-om, koji ostaje mjerodavan.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Ograničenja u izvršavanju",
          blocks: [
            {
              kind: "list",
              items: [
                "Veze se prihvataju prije TLS-a preko zajedničkog ograničivača: 4.096 otvorenih veza i 64 po IPv4 adresi ili IPv6 /64. Odbijeni soketi zatvaraju se bez handshakea.",
                "Zaglavlja HTTP/1 zahtjeva moraju stići u roku od 20 sekundi; HTTP/2 veze se pinguju svakih 30 sekundi i dopuštaju 128 istovremenih tokova.",
                "Svaki keš ima fiksnu granicu i izvor istine iz kojeg može čitati; nijedan ne raste s ključevima koje kontrolira klijent. Keš WebAssembly paketa, na primjer, drži najviše 256 MiB.",
                "Blokirajući poslovi, poput kodiranja slika, izvršavaju se u ograničenim skupovima radnih niti, a ne na Tokio radnim nitima.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Sigurnosna zaglavlja",
          blocks: [
            {
              kind: "paragraph",
              text: "Content-Security-Policy se gradi jednom pri pokretanju. Inline skripte dopuštene su samo preko SHA-256 heševa izračunatih iz `index.html` kompajliranog u izvršnu datoteku, pa skripta za temu prije prvog iscrtavanja ne može odstupiti od politike. Glavni origin nikad ne instancira WebAssembly: demo primjeri i EU5 aplikacija rade u izoliranim okvirima pod vlastitim politikama.",
            },
            {
              kind: "paragraph",
              text: "Kolačići sesije su `Secure` i `HttpOnly`, operacije superkorisnika provjeravaju uloge na serveru, nalozi mogu povezati OpenID Connect identitete, a Swagger UI u produkciji zahtijeva autentifikaciju.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Ugovori i resursi",
          blocks: [
            {
              kind: "list",
              items: [
                "Obrađivači imaju OpenAPI anotacije, TypeScript klijenti frontenda generiraju se iz njih, a cargo xtask openapi pada kad generirane datoteke odstupe.",
                "Viteov manifest ugrađen je zajedno s resursima. Samo datoteke navedene u manifestu i označene otiskom dobijaju jednogodišnju `immutable` politiku keširanja; HTML se uvijek iznova provjerava.",
                "Putanje nepostojećih resursa vraćaju 404 s `no-store`, a ne ljusku aplikacije, pa zastarjela stranica ne može keširati HTML pod URL-om skripte.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Tehnološki stek",
      title: "Frontend",
      lede: "SolidJS 2 aplikacija koja se iscrtava na klijentu, kompajlira u statične datoteke i ugrađuje u binarnu datoteku servera.",
      sections: [
        {
          id: "stack",
          heading: "Stek",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Aplikacija",
                  items: [
                    "SolidJS 2",
                    "Solid Router 2",
                    "TypeScript",
                    "Vite 8",
                  ],
                },
                {
                  label: "Stiliziranje",
                  items: [
                    "Tailwind CSS 4",
                    "IBM Plex Sans",
                    "IBM Plex Mono",
                    "OKLab",
                  ],
                },
                {
                  label: "Komponente",
                  items: [
                    "Toast UI Editor",
                    "Leaflet",
                    "Chart.js",
                    "highlight.js",
                    "DOMPurify",
                  ],
                },
                {
                  label: "Testiranje",
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
              text: "Solidovi sitnozrnati signali ažuriraju samo one DOM čvorove koji zavise od promijenjenih vrijednosti, a njegovi kandidati za izdanje 2.0 omogućavaju da asinhrona čitanja obustave prikaz do najbliže granice učitavanja. Router svaku stranicu učitava na zahtjev, a u produkciji nema JavaScript servera: Rust binarna datoteka isporučuje statične datoteke.",
            },
          ],
        },
        {
          id: "loading",
          heading: "Učitavanje",
          blocks: [
            {
              kind: "list",
              items: [
                "Stranice, panel za chat na početnoj stranici i njegov šejder za konture su zasebni dijelovi, a izgradnja prijavljuje gzip veličinu onoga što se učitava prvo.",
                "U početnom dijelu isporučuje se samo engleski tekst sučelja. Ostali jezici i stranice O meni za svaki jezik su zasebni dijelovi koji se učitavaju pri izboru.",
                "Testovi performansi na usporenoj mobilnoj mreži ciljaju LCP do 2,5 s, interakcije do 200 ms i CLS do 0,1.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Lokalizacija",
          blocks: [
            {
              kind: "paragraph",
              text: "Sučelje podržava 24 jezika, uključujući raspored zdesna nalijevo za arapski, perzijski i urdu. Katalog od otprilike 740 ključeva provjerava se naspram registra na backendu, a rezervirana mjesta moraju preživjeti prevođenje. Ove stranice O meni su tipizirani moduli podataka, pa prijevod ne može nezapaženo izostaviti odjeljak koji postoji u engleskom.",
            },
          ],
        },
        {
          id: "design",
          heading: "Dizajnerski sistem",
          blocks: [
            {
              kind: "paragraph",
              text: "Izgled je topao i tehnički: krem papir ili čista crna, tanke linije okvira, monospace za sve tehničko i jedna akcentna boja odjednom. Boje su semantičke CSS varijable, pa komponentama nikad ne trebaju varijante za tamni režim. Unaprijed zadani akcenti testiraju se na kontrast prema pozadinama stranice, korisnikova vlastita RGB boja prilagođava se u svakom režimu da ostane čitljiva, a svijetli ili tamni režim i akcent primjenjuju se prije prvog iscrtavanja.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Animacije",
          blocks: [
            {
              kind: "list",
              items: [
                "Otkrivanje odjeljaka i traka napretka čitanja koriste CSS animacije vođene klizanjem, koje se, gdje su podržane, izvršavaju izvan glavne niti.",
                "Statistike se broje naviše preko registriranog cjelobrojnog `@property`, a konačna vrijednost je prisutna u označavanju za pomoćne tehnologije.",
                "Promjena teme širi se od prekidača pomoću View Transitions API-ja.",
                "Polje kontura na početnoj stranici je mali WebGL šejder koji se učitava kad je stranica u mirovanju, radi samo u sposobnim desktop preglednicima, ograničava broj sličica u sekundi i pauzira se kad nije vidljiv.",
                "Postavka sajta isključuje sve ovo. Animacije su podrazumijevano uključene i ne prate postavku smanjenog kretanja operativnog sistema.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Tehnološki stek",
      title: "Podsistemi",
      lede: "Dijelovi sajta koji su zanimljiviji nego što blogu strogo treba.",
      sections: [
        {
          id: "live-chat",
          heading: "Chat uživo i pozivi",
          blocks: [
            {
              kind: "paragraph",
              text: "Chat radi preko WebSocketa sa kompaktnim binarnim kodekom, moderacijom i ograničenom historijom. Glasovni i video pozivi idu preko Selective Forwarding Unit jedinice koja radi unutar Axum procesa na webrtc-rs: svaki preglednik drži jednu peer vezu, a SFU prosljeđuje kodirani medij bez transkodiranja, kroz ograničene kanale po objavi i uz ograničenje broja osoba u pozivu. [Otvori chat uživo](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Istraživač terena u Minecraftu",
          blocks: [
            {
              kind: "paragraph",
              text: "Paper dodatak izlaže UNIX soket istog korisnika za ograničene upite o svijetu, uz jednu operaciju odjednom i stroga ograničenja veličine okvira. [Istraživač](/minecraft) crta squaremap pločice vlastitim Leaflet klijentom umjesto da pokreće JavaScript generirane mape.",
            },
            {
              kind: "paragraph",
              text: "Slojevi bioma koriste **CYBM**, kompaktan binarni format pločica koji se servira gzip-komprimiran, ili PNG čiji privatni `cyBM` dio nosi precizne identitete bioma, tako da lebdenje mišem nikad ne nagađa iz boja piksela. Predviđanja zasnovana na seedu za neistražen teren dolaze iz zasebnog radnog procesa za generiranje svijeta izgrađenog na Pumpkinu.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Geo-IP pretrage",
          blocks: [
            {
              kind: "paragraph",
              text: "[Karta posjetitelja](/visitor-board) i [pretraga IP adresa](/geo-ip-db) lociraju adrese bez poziva vanjskog API-ja. Alat za predobradu pretvara IP2Location LITE CSV datoteke u `BTreeMap` strukture s ključem početka raspona, kodira ih pomoću bitcodea i komprimira pomoću zstd na razini 22.",
            },
            {
              kind: "table",
              columns: [
                "Skup podataka",
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
              text: "Pri pokretanju server dekodira po jedan paket da ograniči vršnu potrošnju memorije i internira ponovljene nazive mjesta. Pretraga nalazi posljednji raspon koji počinje na adresi ili ispod nje i provjerava njegovu gornju granicu.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "Otpremljeni [demo paketi](/projects) privremeno se pohranjuju na disk, prepoznaju po magičnim bajtovima, ograničeni su na 50 MiB nakon raspakivanja i servirani iz ograničenog keša unutar izoliranih okvira. Radni prostor također gradi Rust demo primjere, među njima block breaker i ray tracer.",
            },
            {
              kind: "paragraph",
              text: "[EU5 Location DB](/eu5-locations-db) je Slint aplikacija kompajlirana u WebAssembly. Stranica šalje svoj svijetli ili tamni režim preko malog protokola poruka, pa se ugrađena paleta mijenja zajedno sa sajtom.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Cjevovod za fotografije",
          blocks: [
            {
              kind: "paragraph",
              text: "Otpremljene slike dekodiraju se unutar fiksnih granica piksela i kodiraju u AVIF na najsporijoj postavci, čime se vrijeme obrade mijenja za manje datoteke. Lokacije iz EXIF podataka mogu se potvrditi na mapi prije objave, a istovremeno radi najviše dva zadatka obrade.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Tehnološki stek",
      title: "Performanse",
      lede: "Optimizacija vođena profilima i BOLT na backendu, mjerena generatorom opterećenja koji nije usko grlo.",
      sections: [
        {
          id: "approach",
          heading: "Pristup",
          blocks: [
            {
              kind: "paragraph",
              text: "Optimizirane izgradnje treniraju se na reprezentativnom opterećenju. Instrumentirana binarna datoteka bilježi profile na jednokratnom testnom okruženju sa sintetičkim PostgreSQL, SMTP, S3, OpenID Connect i Minecraft krajnjim tačkama. LLVM ponovo gradi server s tim profilima (PGO), a BOLT zatim preuređuje raspored koda konačne binarne datoteke na osnovu uzorkovanog izvršavanja. Sve varijante dijele LTO, frame pointere i ponovo izgrađenu standardnu biblioteku.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Postavke mjerenja",
          blocks: [
            {
              kind: "list",
              items: [
                "Server, simulirani servisi i PostgreSQL vezani su za četiri jezgre procesora AMD Ryzen 7 9800X3D; klijent za opterećenje vezan je za preostale četiri.",
                "HTTPS preko HTTP/1.1 veza iz skupa od 32 veze, uz čitanje cijelih odgovora i vremenska ograničenja od dvije sekunde.",
                "Tri rotirana kruga s pet uzoraka po varijanti, svaki nakon 10.000 zahtjeva za zagrijavanje, iz svježeg snimka baze podataka.",
                "Ukupno 27.000.000 izmjerenih zahtjeva; svaki je vratio HTTP 200.",
                "Opterećenje generira oha; Playwright provjerava odgovor svake rute izvan mjerenog vremenskog prozora.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Rezultati",
          blocks: [
            {
              kind: "bars",
              caption: "Medijan zahtjeva u sekundi na 15 uzoraka po varijanti.",
              groups: [
                {
                  label: "Javna čitanja na deset ruta",
                  items: [
                    {
                      label: "Osnovna verzija",
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
                  label: "Mali JSON za provjeru stanja",
                  items: [
                    {
                      label: "Osnovna verzija",
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
              caption: "Repna latencija za mješavinu od deset ruta.",
              columns: [
                "Varijanta",
                "p95 (ms)",
                "p99 (ms)",
              ],
              numericColumns: [
                1,
                2,
              ],
              rows: [
                [
                  "Osnovna verzija",
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
              text: "PGO je povećao mješovitu propusnost za **17,9%**, a PGO s BOLT-om za **20,1%**, i obje optimizirane izgradnje pobijedile su u svakom rotiranom krugu. BOLT je dodao 1,8% u odnosu na PGO na mješovitim čitanjima, a na trivijalnoj ruti za provjeru stanja bio je 0,8% sporiji. Ovo su rezultati na četiri jezgre; kapacitet u produkciji zavisi i od mreže, radnog skupa baze podataka i stvarnog opterećenja.",
            },
          ],
        },
        {
          id: "corrections",
          heading: "Ispravke",
          blocks: [
            {
              kind: "list",
              items: [
                "`TCP_NODELAY` isprva nije bio postavljen: zadani Rustls prihvatnik u axum-serveru ga ostavlja isključenim, pa su HTTP/2 odgovori mogli čekati oko 40 ms zbog odgođene potvrde prijema. Sada ga postavlja svaki prihvaćeni soket.",
                "Prvo mjerenje stvaralo je opterećenje iz Playwrighta, čiji se klijent usporavao kako je izvršavanje odmicalo i pogrešno je prikazivao regresije. Ti su brojevi odbačeni.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
