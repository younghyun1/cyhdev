import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "listopad 2026",
  chrome: {
    overview: "Przegląd",
    updated: "Ostatnia aktualizacja",
    onThisPage: "Na tej stronie",
    sectionNavigation: "Strony o stosie technologicznym",
    previous: "Poprzednia",
    next: "Następna",
    readingTime: "{minutes} min czytania",
  },
  pages: {
    overview: {
      eyebrow: "O tej witrynie",
      title: "Stos technologiczny bloga",
      lede: "Jeden plik binarny w Rust, jedna baza PostgreSQL, jedna mała maszyna. Te strony opisują, jak witryna jest zbudowana, hostowana, zabezpieczona i mierzona oraz dlaczego ma taki kształt.",
      sections: [
        {
          id: "at-a-glance",
          heading: "W skrócie",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53 889 żądań/s",
                  label: "Mediana przepustowości mieszanych odczytów publicznych z PGO i BOLT na czterech rdzeniach serwera",
                },
                {
                  value: "~150 µs",
                  label: "Zaobserwowany czas obiegu zapytania do bazy danych przez gniazdo UNIX, wobec ok. 600 µs przez TCP na localhost",
                },
                {
                  value: "0",
                  label: "Procesów Node.js w produkcji; frontend jest osadzony w pliku binarnym serwera",
                },
                {
                  value: "24",
                  count: 24,
                  label: "Języków interfejsu, każdy pobierany tylko po wybraniu",
                },
                {
                  value: "4096",
                  label: "Otwartych połączeń dopuszczanych przed TLS, z co najwyżej 64 na adres lub IPv6 /64",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Proces obsługujący TLS, API, WebSockets, multimedia WebRTC i osadzony frontend",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Ścieżka żądania",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Przeglądarka",
                  detail: "HTTP/2 lub HTTP/1.1 przez TLS. Frontend jest buforowany pod niezmiennymi adresami URL z odciskiem treści.",
                },
                {
                  label: "Dopuszczanie",
                  detail: "Połączenia TCP są dopuszczane przed TLS: 4096 łącznie i 64 na adres lub IPv6 /64.",
                },
                {
                  label: "rustls",
                  detail: "TLS kończy się wewnątrz procesu Rust, z `TCP_NODELAY` na każdym gnieździe.",
                },
                {
                  label: "Axum",
                  detail: "Nagłówki bezpieczeństwa, sesje, limity częstotliwości żądań, kompresja i limity rozmiaru treści, a potem handler danej funkcji.",
                },
                {
                  label: "Pamięć podręczna · Diesel",
                  detail: "Ograniczone pamięci podręczne w RAM odpowiadają na często wykonywane odczyty; wszystko inne przechodzi przez asynchroniczny Diesel.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "Na tym samym hoście przez gniazdo UNIX, z asynchronicznym wejściem/wyjściem io_uring.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Rozdziały",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Infrastruktura",
                  href: "/about-blog/infrastructure",
                  meta: "Host · Gentoo · sieć",
                  text: "Maszyna, system operacyjny i system plików, sieć oraz powód, dla którego nie ma reverse proxy.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "Stos usług, model danych, limity działania, nagłówki bezpieczeństwa i kontrakty API.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "Szkielet aplikacji, ładowanie, lokalizacja, system projektowy i animacje.",
                },
                {
                  title: "Podsystemy",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "Czat na żywo z SFU w procesie, eksplorator Minecraft, wyszukiwanie Geo-IP i WebAssembly.",
                },
                {
                  title: "Wydajność",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Zoptymalizowane kompilacje, sposób ich pomiaru i wyniki.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Jak sprawdzane są zmiany",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy na celach natywnym i WebAssembly jako bramka kontrolna na etapie implementacji.",
                "Lokalne kontrole cargo xtask: formatowanie, natywne testy jednostkowe, rozbieżność kontraktu OpenAPI oraz sprawdzenie typów, lint, testy jednostkowe i kompilacja frontendu.",
                "Playwright na Chromium i WebKit z kontrolami dostępności axe oraz kontrolami zawijania na urządzeniach mobilnych do 320 pikseli CSS.",
                "Testy integracyjne i wycofywania migracji na jednorazowej instancji PostgreSQL oraz lokalne skanowanie sekretów gitleaks.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Stos technologiczny",
      title: "Infrastruktura",
      lede: "Mała maszyna w domu, dystrybucja Linuksa oparta na kodzie źródłowym i żadnej warstwy, której ruch nie potrzebuje.",
      sections: [
        {
          id: "host",
          heading: "Host",
          blocks: [
            {
              kind: "paragraph",
              text: "Witryna działa na [miniserwerze](https://store.minisforum.com/products/minisforum-um690l-slim) w moim mieszkaniu: ośmiordzeniowy mobilny procesor Ryzen 9 podbijany do 4,9 GHz, 32 GB RAM i dysk NVMe SSD o pojemności 1 TB. Za około 400 USD kosztuje mniej niż rok wyraźnie słabszego sprzętu w chmurze i odzwierciedla mój entuzjazm dla samodzielnego hostingu.",
            },
            {
              kind: "paragraph",
              text: "Działa na nim połączony serwer backendu i frontendu, PostgreSQL oraz serwer Minecraft. Jeśli chcesz zagrać, napisz do mnie e-mail.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo i btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) nie jest oczywistym wyborem na serwer; w pracy wybrałbym Debiana stable na ext4. Tutaj lubię sam kompilować pakiety z optymalizacjami dla konkretnego procesora.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) daje migawki, kompresję i copy-on-write, ale copy-on-write mocno fragmentuje pliki bazy danych. Dlatego katalogi danych PostgreSQL i Minecraft są z niego wyłączone. [Test wydajnościowy systemów plików](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) pokazuje, że domyślny btrfs ustępuje ext4 i XFS; podejrzewam, że wyłączenie copy-on-write zamyka większość tej różnicy, co byłoby ciekawym pomiarem.",
            },
          ],
        },
        {
          id: "network",
          heading: "Sieć",
          blocks: [
            {
              kind: "paragraph",
              text: "Ruch przychodzi przez przewodowe łącze Xfinity o przepustowości 1 Gb/s; modem dostawcy nie ma Ethernetu 2,5 Gb, czego ta witryna nie odczuje. DNS hostuje Route 53.",
            },
            {
              kind: "paragraph",
              text: "Wewnątrz maszyny nie ma reverse proxy, środowiska uruchomieniowego kontenerów ani service mesh. PostgreSQL działa bezpośrednio w systemie operacyjnym, a jeden plik binarny w Rust serwuje frontend i API. Zwykłe żądania HTTP są przekierowywane na HTTPS przez ten sam proces.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "Gniazda UNIX",
          blocks: [
            {
              kind: "paragraph",
              text: "Serwer łączy się z PostgreSQL przez gniazdo UNIX, a nie przez TCP na localhost, co całkowicie omija stos sieciowy. [Testy wydajnościowe](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) pokazują prawie dwukrotnie wyższą liczbę transakcji; tutaj zaobserwowałem czasy obiegu zapytania nawet **150 µs**, wobec około **600 µs** przez localhost. To rozwiązanie staromodne i nieistotne dla większości wdrożeń w chmurze, a jest szybsze.",
            },
            {
              kind: "paragraph",
              text: "PostgreSQL 18, wydany we wrześniu 2025, dodał [asynchroniczne wejście/wyjście](https://pganalyze.com/blog/postgres-18-async-io), a io_uring jest tu włączony, co wyraźnie przyspiesza odczyty.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Kompilacja i wdrożenie",
          blocks: [
            {
              kind: "list",
              items: [
                "Kompilacja frontendu jest osadzona w pliku binarnym backendu razem ze skompresowanymi wcześniej kopiami zstd i gzip, więc w produkcji nie działa żaden proces Node.js.",
                "Zoptymalizowane kompilacje mogą być trenowane optymalizacją sterowaną profilem, a po zlinkowaniu optymalizowane za pomocą BOLT; zobacz [Wydajność](/about-blog/performance).",
                "Poświadczenia, certyfikaty i dane uruchomieniowe nie trafiają do Git, a skan gitleaks sprawdza drzewo, zanim zmiany opuszczą maszynę.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Stos technologiczny",
      title: "Backend",
      lede: "Rust na Axum i Tokio, PostgreSQL przez asynchroniczny Diesel oraz jawny limit na wszystko, co klient może trzymać otwarte.",
      sections: [
        {
          id: "stack",
          heading: "Stos",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Środowisko uruchomieniowe",
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
                  label: "Dane",
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
                  label: "Integracje",
                  items: [
                    "openidconnect",
                    "lettre",
                    "Amazon S3",
                    "webrtc-rs",
                    "image + AVIF",
                  ],
                },
                {
                  label: "Kontrakty",
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
              text: "Usługa to jeden proces ze współdzielonym stanem: sesjami w pamięci, synchronizacją pamięci podręcznej przy starcie i zaplanowanymi zadaniami konserwacyjnymi. mimalloc jest globalnym alokatorem; jeśli maszyna ma prawdziwe rdzenie, zachowanie alokatora warto traktować poważnie.",
            },
          ],
        },
        {
          id: "data",
          heading: "Model danych",
          blocks: [
            {
              kind: "paragraph",
              text: "W koreańskich startupach i firmach, które widziałem, MySQL i MariaDB uchodziły za jedyne poważne relacyjne bazy danych. PostgreSQL dawno je wyprzedził w obszarach, na których mi zależy, od natywnych UUID po binarny JSON.",
            },
            {
              kind: "list",
              items: [
                "**Wpisy** mają slugi, streszczenia, metadane JSONB, stan publikacji, zdenormalizowane liczniki i mapowania tagów. Komentarze tworzą wątki przez opcjonalny element nadrzędny, a głosy na wpisy i komentarze mają osobne tabele.",
                "**Konta** przechowują kraj i język, tabele ról i uprawnień oraz wersjonowane zdjęcia profilowe we własnej tabeli, a nie w często odczytywanym wierszu użytkownika.",
                "**Klucze** to uporządkowane w czasie wartości UUIDv7: nie da się ich wyliczyć jak kolejnych identyfikatorów i znacznie lepiej służą lokalności B-drzewa niż losowe UUIDv4.",
                "**Wyszukiwanie** korzysta z indeksu tantivy zapisanego na dysku, obejmującego tytuły wpisów i tagi, utrzymywanego spójnie z PostgreSQL, który pozostaje źródłem prawdy.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Limity działania",
          blocks: [
            {
              kind: "list",
              items: [
                "Połączenia są dopuszczane przed TLS przez wspólny ogranicznik: 4096 otwartych połączeń i 64 na adres IPv4 lub IPv6 /64. Odrzucone gniazda są zamykane bez uzgadniania.",
                "Nagłówki żądań HTTP/1 muszą dotrzeć w ciągu 20 sekund; połączenia HTTP/2 są pingowane co 30 sekund i dopuszczają 128 równoczesnych strumieni.",
                "Każda pamięć podręczna ma stały limit i źródło prawdy, z którego może odczytywać; żadna nie rośnie wraz z kluczami kontrolowanymi przez klienta. Na przykład pamięć podręczna paczek WebAssembly mieści co najwyżej 256 MiB.",
                "Praca blokująca, taka jak kodowanie obrazów, działa na ograniczonych pulach wątków roboczych, a nie na wątkach Tokio.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Nagłówki bezpieczeństwa",
          blocks: [
            {
              kind: "paragraph",
              text: "Content-Security-Policy jest budowany raz przy starcie. Skrypty inline są dozwolone wyłącznie przez skróty SHA-256 obliczone z `index.html` skompilowanego do pliku binarnego, więc skrypt motywu uruchamiany przed pierwszym renderowaniem nie może rozejść się z polityką. Główny origin nigdy nie tworzy instancji WebAssembly: dema i aplikacja EU5 działają w ramkach z piaskownicą, pod własnymi politykami.",
            },
            {
              kind: "paragraph",
              text: "Ciasteczka sesji mają atrybuty `Secure` i `HttpOnly`, operacje administratora sprawdzają role po stronie serwera, konta mogą łączyć tożsamości OpenID Connect, a Swagger UI w produkcji wymaga uwierzytelnienia.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Kontrakty i zasoby",
          blocks: [
            {
              kind: "list",
              items: [
                "Handlery mają adnotacje OpenAPI, klienty TypeScript frontendu są z nich generowane, a cargo xtask openapi kończy się błędem, gdy wygenerowane pliki są nieaktualne.",
                "Manifest Vite jest osadzony razem z zasobami. Tylko pliki z manifestu, z odciskiem treści, dostają roczną politykę pamięci podręcznej `immutable`; HTML zawsze jest rewalidowany.",
                "Brakujące ścieżki zasobów zwracają 404 z `no-store` zamiast szkieletu aplikacji, więc nieaktualna strona nie może zbuforować HTML pod adresem URL skryptu.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Stos technologiczny",
      title: "Frontend",
      lede: "Aplikacja SolidJS 2 renderowana po stronie klienta, kompilowana do statycznych zasobów i osadzona w pliku binarnym serwera.",
      sections: [
        {
          id: "stack",
          heading: "Stos",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Aplikacja",
                  items: [
                    "SolidJS 2",
                    "Solid Router 2",
                    "TypeScript",
                    "Vite 8",
                  ],
                },
                {
                  label: "Style",
                  items: [
                    "Tailwind CSS 4",
                    "IBM Plex Sans",
                    "IBM Plex Mono",
                    "OKLab",
                  ],
                },
                {
                  label: "Komponenty",
                  items: [
                    "Toast UI Editor",
                    "Leaflet",
                    "Chart.js",
                    "highlight.js",
                    "DOMPurify",
                  ],
                },
                {
                  label: "Testy",
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
              text: "Drobnoziarniste sygnały Solida aktualizują tylko te węzły DOM, które zależą od zmienionych wartości, a kandydaci do wydania 2.0 pozwalają asynchronicznym odczytom zawieszać renderowanie do najbliższej granicy ładowania. Router ładuje leniwie każdą stronę, a produkcyjnego serwera JavaScript nie ma: plik binarny w Rust serwuje pliki statyczne.",
            },
          ],
        },
        {
          id: "loading",
          heading: "Ładowanie",
          blocks: [
            {
              kind: "list",
              items: [
                "Strony, panel czatu na stronie głównej i jego shader konturów to osobne paczki, a kompilacja raportuje rozmiar po kompresji gzip tego, co ładuje się najpierw.",
                "W początkowej paczce znajduje się tylko angielski tekst interfejsu. Pozostałe języki oraz strony O mnie w każdym języku to osobne paczki ładowane po wyborze.",
                "Testy wydajnościowe na ograniczonym łączu mobilnym przyjmują za cel LCP do 2,5 s, interakcje do 200 ms i CLS do 0,1.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Lokalizacja",
          blocks: [
            {
              kind: "paragraph",
              text: "Interfejs obsługuje 24 języki, w tym układy od prawej do lewej dla arabskiego, perskiego i urdu. Katalog około 740 kluczy jest walidowany względem rejestru backendu, a znaczniki zastępcze muszą przetrwać tłumaczenie. Te strony O mnie są typowanymi modułami danych, więc tłumaczenie nie może po cichu pominąć sekcji, którą ma wersja angielska.",
            },
          ],
        },
        {
          id: "design",
          heading: "System projektowy",
          blocks: [
            {
              kind: "paragraph",
              text: "Wygląd jest ciepły i techniczny: kremowy papier lub czysta czerń, cienkie obramowania, czcionka o stałej szerokości dla wszystkiego, co techniczne, i jeden kolor akcentu naraz. Kolory to semantyczne zmienne CSS, więc komponenty nie potrzebują wariantów dla trybu ciemnego. Predefiniowane akcenty są testowane pod kątem kontrastu z tłem strony, własny kolor RGB odwiedzającego jest korygowany w każdym trybie, by pozostał czytelny, a tryb jasny lub ciemny oraz akcent są stosowane przed pierwszym wyrenderowaniem.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Animacje",
          blocks: [
            {
              kind: "list",
              items: [
                "Odsłanianie sekcji i pasek postępu czytania używają animacji CSS sterowanych przewijaniem, które tam, gdzie są obsługiwane, działają poza głównym wątkiem.",
                "Statystyki odliczają w górę przez zarejestrowaną liczbę całkowitą `@property`, a wartość końcowa jest obecna w znacznikach dla technologii asystujących.",
                "Przełączenie motywu rozchodzi się od przełącznika przez View Transitions API.",
                "Pole konturów na stronie głównej to mały shader WebGL, który ładuje się, gdy strona jest bezczynna, działa tylko w wydajnych przeglądarkach desktopowych, ogranicza liczbę klatek na sekundę i zatrzymuje się poza ekranem.",
                "Ustawienie witryny wyłącza to wszystko. Animacje są domyślnie włączone i nie uwzględniają systemowego ustawienia ograniczenia animacji.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Stos technologiczny",
      title: "Podsystemy",
      lede: "Części witryny ciekawsze, niż blog ściśle by wymagał.",
      sections: [
        {
          id: "live-chat",
          heading: "Czat na żywo i połączenia",
          blocks: [
            {
              kind: "paragraph",
              text: "Czat działa przez WebSocket z kompaktowym kodekiem binarnym, moderacją i ograniczoną historią. Połączenia głosowe i wideo przechodzą przez Selective Forwarding Unit działającą wewnątrz procesu Axum na webrtc-rs: każda przeglądarka utrzymuje jedno połączenie peer, a SFU przekazuje zakodowane media bez transkodowania, przez ograniczone kanały na publikację i limit uczestników. [Otwórz czat na żywo](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Eksplorator terenu Minecrafta",
          blocks: [
            {
              kind: "paragraph",
              text: "Wtyczka Paper udostępnia gniazdo UNIX tego samego użytkownika do ograniczonych zapytań o świat, z jedną operacją naraz i ścisłymi limitami rozmiaru ramki. [Eksplorator](/minecraft) rysuje kafelki squaremap własnym klientem Leaflet zamiast uruchamiać JavaScript wygenerowanej mapy.",
            },
            {
              kind: "paragraph",
              text: "Nakładki biomów używają **CYBM**, kompaktowego binarnego formatu kafelków serwowanego w kompresji gzip, albo PNG, którego prywatny fragment `cyBM` niesie dokładne identyfikatory biomów, więc najechanie kursorem nigdy nie zgaduje na podstawie kolorów pikseli. Przewidywania oparte na ziarnie dla niezbadanego terenu pochodzą z osobnego procesu roboczego generującego świat, zbudowanego na Pumpkin.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Wyszukiwanie Geo-IP",
          blocks: [
            {
              kind: "paragraph",
              text: "[Tablica odwiedzających](/visitor-board) i [wyszukiwarka IP](/geo-ip-db) lokalizują adresy bez wywoływania zewnętrznego API. Narzędzie przetwarzające zamienia pliki CSV IP2Location LITE w `BTreeMap` z kluczem w postaci początku zakresu, koduje je za pomocą bitcode i kompresuje zstd na poziomie 22.",
            },
            {
              kind: "table",
              columns: [
                "Zbiór danych",
                "CSV",
                "Paczka",
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
              text: "Przy starcie serwer dekoduje jedną paczkę naraz, by ograniczyć szczytowe zużycie pamięci, i deduplikuje powtarzające się nazwy miejsc. Wyszukiwanie znajduje ostatni zakres zaczynający się w adresie lub poniżej niego i sprawdza jego górną granicę.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "Przesłane [paczki dem](/projects) są najpierw zapisywane na dysku, rozpoznawane po sygnaturze bajtowej (magic bytes), ograniczone do 50 MiB po dekompresji i serwowane z ograniczonej pamięci podręcznej wewnątrz ramek w piaskownicy. Przestrzeń robocza buduje też dema w Ruście, w tym grę block breaker i ray tracer.",
            },
            {
              kind: "paragraph",
              text: "[EU5 Location DB](/eu5-locations-db) to aplikacja Slint skompilowana do WebAssembly. Strona przekazuje jej tryb jasny lub ciemny przez mały protokół komunikatów, dzięki czemu osadzona paleta zmienia się razem z witryną.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Potok fotografii",
          blocks: [
            {
              kind: "paragraph",
              text: "Przesłane pliki są dekodowane w stałych granicach pikseli i kodowane do AVIF z najwolniejszym ustawieniem wstępnym, co zamienia czas przetwarzania na mniejsze pliki. Lokalizacje z danych EXIF można potwierdzić na mapie przed publikacją, a jednocześnie działają co najwyżej dwa zadania przetwarzania.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Stos technologiczny",
      title: "Wydajność",
      lede: "Optymalizacja sterowana profilem i BOLT w backendzie, mierzone generatorem obciążenia, który nie jest wąskim gardłem.",
      sections: [
        {
          id: "approach",
          heading: "Podejście",
          blocks: [
            {
              kind: "paragraph",
              text: "Zoptymalizowane kompilacje są trenowane na reprezentatywnym obciążeniu. Plik binarny z instrumentacją zapisuje profile na jednorazowym środowisku testowym z syntetycznymi punktami końcowymi PostgreSQL, SMTP, S3, OpenID Connect i Minecraft. LLVM buduje serwer ponownie z tymi profilami (PGO), a następnie BOLT zmienia układ kodu końcowej binarki na podstawie próbkowanego wykonania. Wszystkie warianty używają LTO, wskaźników ramki i przebudowanej biblioteki standardowej.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Konfiguracja",
          blocks: [
            {
              kind: "list",
              items: [
                "Serwer, dostawcy środowiska testowego i PostgreSQL przypięte do czterech rdzeni procesora AMD Ryzen 7 9800X3D; klient generujący obciążenie przypięty do pozostałych czterech.",
                "HTTPS przez pulę połączeń HTTP/1.1 z 32 połączeniami, pełnym odczytem odpowiedzi i limitem czasu dwóch sekund.",
                "Trzy rotowane rundy z pięcioma próbkami na wariant, każda po 10 000 żądań rozgrzewkowych, ze świeżej migawki bazy danych.",
                "Łącznie 27 000 000 mierzonych żądań; każde zwróciło HTTP 200.",
                "Obciążenie pochodzi z oha; Playwright sprawdza odpowiedź każdej trasy poza mierzonym oknem.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Wyniki",
          blocks: [
            {
              kind: "bars",
              caption: "Mediana żądań na sekundę z 15 próbek na wariant.",
              groups: [
                {
                  label: "Publiczne odczyty z dziesięciu tras",
                  items: [
                    {
                      label: "Punkt odniesienia",
                      value: 44874.12,
                      display: "44 874",
                    },
                    {
                      label: "PGO",
                      value: 52925.46,
                      display: "52 925",
                    },
                    {
                      label: "PGO + BOLT",
                      value: 53888.61,
                      display: "53 889",
                      emphasis: true,
                    },
                  ],
                },
                {
                  label: "Mały JSON ze stanem działania",
                  items: [
                    {
                      label: "Punkt odniesienia",
                      value: 272837.41,
                      display: "272 837",
                    },
                    {
                      label: "PGO",
                      value: 308685.26,
                      display: "308 685",
                      emphasis: true,
                    },
                    {
                      label: "PGO + BOLT",
                      value: 306276.64,
                      display: "306 277",
                    },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "Opóźnienie ogona rozkładu dla mieszanki dziesięciu tras.",
              columns: [
                "Wariant",
                "p95 (ms)",
                "p99 (ms)",
              ],
              numericColumns: [
                1,
                2,
              ],
              rows: [
                [
                  "Punkt odniesienia",
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
              text: "PGO zwiększyło przepustowość mieszanego obciążenia o **17,9%**, a PGO z BOLT o **20,1%**, i obie zoptymalizowane kompilacje wygrały każdą rotowaną rundę. BOLT dodał 1,8% względem PGO przy mieszanych odczytach i był o 0,8% wolniejszy na trywialnej trasie stanu działania. To wyniki dla czterech rdzeni; wydajność produkcyjna zależy też od sieci, zbioru roboczego bazy danych i rzeczywistego ruchu.",
            },
          ],
        },
        {
          id: "corrections",
          heading: "Poprawki",
          blocks: [
            {
              kind: "list",
              items: [
                "`TCP_NODELAY` nie było początkowo ustawione: domyślny akceptor Rustls w axum-server zostawia tę opcję wyłączoną, więc odpowiedzi HTTP/2 mogły czekać około 40 ms na opóźnione potwierdzenie. Teraz każde przyjęte gniazdo ją ustawia.",
                "Pierwszy test wydajnościowy generował obciążenie z Playwright, którego klient zwalniał w trakcie przebiegu i błędnie pokazywał regresje. Te liczby zostały odrzucone.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
