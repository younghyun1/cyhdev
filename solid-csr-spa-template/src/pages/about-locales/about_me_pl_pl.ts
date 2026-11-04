import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "listopad 2026",
  chrome: {
    overview: "Przegląd",
    updated: "Ostatnia aktualizacja",
    onThisPage: "Na tej stronie",
    sectionNavigation: "Strony O mnie",
    previous: "Poprzednia",
    next: "Następna",
    readingTime: "{minutes} min czytania",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "O mnie",
      lede: "Inżynier backendu i infrastruktury, pracuję w Rust, PostgreSQL i platformach chmurowych. Buduję usługi, które dobrze wykorzystują sprzęt pod spodem, a także zajmuję się wdrożeniami, monitoringiem i bezpieczeństwem, dzięki którym te usługi działają.",
      sections: [
        {
          id: "introduction",
          heading: "Wprowadzenie",
          blocks: [
            {
              kind: "paragraph",
              text: "Jestem koreańskim inżynierem oprogramowania mieszkającym w Kolorado. W **Soundpatrol** pracuję jako inżynier Rust nad cyberbezpieczeństwem, DevOps, usługami webowymi i narzędziami dla programistów, głównie w Rust, Python, PostgreSQL, Kubernetes i Google Cloud.",
            },
            {
              kind: "paragraph",
              text: "Wcześniej kierowałem pracami nad backendem i infrastrukturą cyfrowego bliźniaka budynku dla Samsung C&T, współtworzyłem backendy aplikacji Hyundai, Kia i Genesis oraz zbudowałem zbieranie danych dla platformy analitycznej YouTube. Od 2023 roku moja praca skupia się na usługach backendowych, potokach danych i infrastrukturze pod nimi.",
            },
            {
              kind: "paragraph",
              text: "Wierzę w budowanie usług, które dobrze wykorzystują nowoczesny sprzęt i języki programowania. W praktyce oznacza to staranne projektowanie schematów, ograniczone kolejki i pamięci podręczne, jawną obsługę błędów oraz wdrożenia, które można zbadać i przywrócić. Lubię pomagać współpracownikom w nauce Rusta i chcę pracować z ludźmi, którzy dbają o swoje rzemiosło.",
            },
          ],
        },
        {
          id: "results",
          heading: "Wybrane wyniki",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "Niższe miesięczne koszty chmury cyfrowego bliźniaka Samsung C&T, z około 5000 USD do 150 USD",
                },
                {
                  value: "80",
                  count: 80,
                  label: "Endpointów REST zaimplementowanych dla cyfrowego bliźniaka, z logiką dziedzinową inżynierii budowlanej",
                },
                {
                  value: "9 mies.",
                  count: 9,
                  suffix: " mies.",
                  label: "Produkcyjna eksploatacja cyfrowego bliźniaka bez przestoju w czasie działania",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "Wzrost przepustowości mieszanych odczytów w backendzie tej strony dzięki [PGO i BOLT](/about-blog/performance)",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Statycznie zlinkowane kontenery usług z kompilacji wieloetapowych",
                },
                {
                  value: "30,000",
                  label: "Linii napisanych dla backendu cyfrowego bliźniaka, około 75% jego kodu",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Narzędzia",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Języki",
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
                  label: "Dane",
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
                  label: "Infrastruktura",
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
                  label: "Niezawodność i bezpieczeństwo",
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
                  label: "Interfejsy i urządzenia",
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
          heading: "Więcej o mnie",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Doświadczenie",
                  href: "/about/experience",
                  meta: "2016 - obecnie",
                  text: "Role od Soundpatrol aż do pierwszego stażu, wraz z projektami i wynikami każdej z nich.",
                },
                {
                  title: "Projekty",
                  href: "/about/projects",
                  meta: "Rust · open source",
                  text: "Sillok, EU5 Location DB, ta strona i inne niezależne projekty oraz projekty open source.",
                },
                {
                  title: "Życiorys",
                  href: "/about/background",
                  meta: "Wykształcenie · badania",
                  text: "Wykształcenie, publikacje naukowe, nagrody, języki i praca poza branżą oprogramowania.",
                },
                {
                  title: "Jak działa ta strona",
                  href: "/about-blog",
                  meta: "Stos technologiczny",
                  text: "Hosting, backend, frontend, podsystemy i pomiary wydajności stojące za cyhdev.com.",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "Kontakt",
          blocks: [
            {
              kind: "list",
              items: [
                "**E-mail**: [younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**: [github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**: [linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "E-mail to najpewniejszy sposób, aby się ze mną skontaktować. Jeśli chcesz dołączyć do serwera Minecraft, napisz o tym również tam.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "O mnie",
      title: "Doświadczenie",
      lede: "Praca nad backendem, infrastrukturą i bezpieczeństwem w amerykańskim startupie, dla koreańskich klientów korporacyjnych i na wczesnym stażu, a także praca sprzed czasów programowania.",
      sections: [
        {
          id: "software",
          heading: "Inżynieria oprogramowania",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Inżynier Rust",
                  period: "maj 2026 - obecnie",
                  location: "Stany Zjednoczone · Zdalnie",
                  summary: "Cyberbezpieczeństwo, DevOps, tworzenie usług webowych i narzędzia dla programistów w firmie wykrywającej muzykę i dźwięk.",
                  highlights: [
                    "Utrzymuję infrastrukturę Kubernetes w Google Cloud, infrastrukturę jako kod oraz recenzowane procesy wdrożeniowe, które używają krótkotrwałych poświadczeń ograniczonych do środowiska.",
                    "Planuję i przeprowadzam produkcyjne migracje magazynów danych, sprawdzając zmigrowany stan i ćwicząc odtwarzanie przed każdym przełączeniem.",
                    "Konsoliduję metryki, logi i ślady w OpenTelemetry oraz buduję w Rust narzędzia do monitorowania stanu usług, diagnostyki i widoczności kosztów.",
                    "Pracuję nad gotowością w zakresie bezpieczeństwa: administracja dostępem, rotacja poświadczeń i śledzenie ich wygaśnięcia, polityki, reagowanie na incydenty i materiały dowodowe do audytów.",
                    "Buduję wewnętrzne usługi i narzędzia w Rust i Python dla procesów operacyjnych i wsparcia.",
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
                  title: "Inżynier oprogramowania",
                  period: "sty 2025 - lip 2025",
                  location: "Seongnam, Korea Południowa",
                  summary: "Usługi backendowe i narzędzia infrastrukturalne dla wewnętrznych platform oraz oficjalnej aplikacji dla fanów grupy K-pop.",
                  highlights: [
                    "Dodawałem API, poprawki błędów i integracje do oficjalnych backendów aplikacji Hyundai, Kia i Genesis, pracując w dużej bazie kodu w Javie i koordynując działania z dostawcami usług używanych przez dziesiątki milionów ludzi w Azji i Europie.",
                    "Zbudowałem w Rust i Python potoki internacjonalizacji, które przekształcają przetłumaczone treści i ładują je do korporacyjnych baz danych, we współpracy z administratorami baz odpowiedzialnymi za zbiory danych połączone z systemami fabrycznymi.",
                    "Opracowałem narzędzia do zarządzania infrastrukturą wewnętrznych usług.",
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
                  title: "Inżynier oprogramowania (kontrakt)",
                  period: "lis 2024 - gru 2024",
                  location: "Seul, Korea Południowa",
                  highlights: [
                    "Zaprojektowałem schemat PostgreSQL i backend w Rust dla wspieranej przez AI platformy analitycznej YouTube oraz kierowałem jej wdrożeniem w AWS i Google Cloud.",
                    "Zbudowałem procesy zbierania danych z API YouTube obejmujące dziesiątki milionów komentarzy i użytkowników, łączące dane kanałów, transkrypcje i treść stron.",
                    "Zintegrowałem hostowane modele językowe oraz modele open source uruchamiane na serwerze, aby generować streszczenia i wnioski marketingowe.",
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
                  title: "Inżynier oprogramowania backendu (lider)",
                  period: "sie 2023 - sie 2024",
                  location: "Seul, Korea Południowa",
                  summary: "Kierowałem rozwojem backendu, baz danych i infrastruktury chmurowej dla dwóch platform dla budynków i energetyki.",
                  highlights: [],
                  projects: [
                    {
                      name: "Cyfrowy bliźniak Samsung C&T Global Engineering Center",
                      highlights: [
                        "Zastąpiłem backend w Express i układ mikrousług w chmurze zoptymalizowanym monolitem w Rust/Axum, obniżając miesięczne koszty infrastruktury z około **5000 do 150 USD**.",
                        "Napisałem około **30 000 linii**, czyli mniej więcej 75% backendu, w tym **80 endpointów REST** z logiką dziedzinową inżynierii budowlanej, integracją danych i operacjami w czasie rzeczywistym.",
                        "Skróciłem wiele ścieżek żądań z wielosekundowym P99 do jedno- lub dwucyfrowych milisekund we wdrożeniu z dwoma rdzeniami i 4 GB pamięci, dzięki asynchronicznemu Rust, wielowątkowości, pamięci podręcznej oraz strojeniu schematu i zapytań PostgreSQL.",
                        "Zintegrowałem kilka tysięcy czujników, liczników oraz sterowników HVAC i systemów wodnych przez BACnet i Modbus, na miejscu w siedzibie Samsung C&T.",
                        "System działał produkcyjnie przez dziewięć miesięcy bez przestoju w czasie działania, z około 90% pokryciem obsługi błędów.",
                        "Zbudowałem CI/CD z runnerami GitHub, AWS CodeDeploy i Dockerem; statyczne kompilacje dostrojone do procesora dawały przenośne kontenery o rozmiarze około 20 MB.",
                        "Połączyłem inferencję modeli predykcyjnych przez ECS, EC2, SQS, SNS, S3 i SageMaker, wdrożyłem wymagania bezpieczeństwa Samsung SDS i zintegrowałem backend z interfejsem w Unreal Engine 4 zbudowanym przez studio partnerskie.",
                      ],
                    },
                    {
                      name: "Usługa zarządzania energią z AI",
                      highlights: [
                        "Zaprojektowałem od podstaw schemat PostgreSQL i API w Rust/Axum oraz napisałem około **25 000 linii**, czyli mniej więcej 70% backendu.",
                        "Wybrałem bezpośredni SQL przez tokio-postgres zamiast ORM, aby mieć kontrolę nad zapytaniami i czasem kompilacji, stosując doświadczenia z cyfrowego bliźniaka.",
                        "Pomogłem dwóm inżynierom pochodzącym z Node.js i Javy stać się produktywnymi w Rust w około miesiąc, w dziewięcioosobowym zespole inżynierów backendu, frontendu i data science.",
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
                  title: "Stażysta inżynierii oprogramowania",
                  period: "sie 2022 - mar 2023",
                  location: "Austin, Teksas · Zdalnie",
                  highlights: [
                    "Zbudowałem bota Discord w Pythonie, który przez API Reddita wyszukiwał prośby o zlecenia artystyczne i publikował je na kanale społeczności, uruchamiany w AWS Lambda co pięć minut.",
                    "Zaimplementowałem w Lambdzie logikę przeliczania walut, pobierania opłat i naliczania dopłat w różnych jurysdykcjach.",
                    "Wprowadzałem poprawki we frontendzie w React dotyczące układu, zasobów i treści.",
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
          heading: "Przed oprogramowaniem",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Korea Institute of Maritime Science and Technology",
                  title: "Tłumacz pisemny i ustny",
                  period: "cze - lip 2023, sie 2024",
                  location: "Seul, Korea Południowa · Kontrakt",
                  highlights: [
                    "Tłumaczyłem ustnie w językach angielskim, koreańskim i norweskim na konferencji poświęconej badaniom morskim, w której uczestniczyli ambasador Norwegii w Korei, norweski sekretarz stanu, kierownictwo instytutu i prezes Dongwon Group.",
                    "Tłumaczyłem przemówienia i szkoliłem pracowników instytutu oraz jego dyrektora przed międzynarodową konferencją i wydarzeniem Departamentu Energii USA poświęconym wspólnemu projektowi.",
                  ],
                },
                {
                  organization: "Armia Republiki Korei · 2. Dywizja Piechoty Armii USA",
                  title: "Sierżant, łącznik i instruktor KATUSA",
                  period: "mar 2016 - gru 2017",
                  location: "Dongducheon, Korea Południowa",
                  highlights: [
                    "Służyłem w batalionie dowodzenia dywizji w ramach programu [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army), we wspólnym środowisku koreańsko-amerykańskim.",
                    "Szkoliłem i wprowadzałem do służby około 6000 amerykańskich i koreańskich żołnierzy na kursach z zakresu sprawności fizycznej, kultury, historii i profilaktyki.",
                    "Kierowałem 13-osobowym zespołem i utrzymywałem gotowość podczas [kryzysu w 2017 roku](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) na półwyspie.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Pracownik załadunku w logistyce",
                  period: "2020 - 2022",
                  location: "Korea Południowa · Zlecenia dorywcze",
                  highlights: [
                    "Praca nocna w magazynie równolegle ze studiami.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "O mnie",
      title: "Projekty",
      lede: "Niezależne projekty i prace open source, głównie w Rust. Interaktywne dema są na [stronie projektów](/projects).",
      sections: [
        {
          id: "featured",
          heading: "Wyróżnione",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "Dziennik pracy w wierszu poleceń. Cele, wykonane zadania, korekty i wycofania są zapisywane jako zdarzenia tylko do dopisywania, z indeksowanymi bieżącymi projekcjami, a archiwum oparte na Gicie scala historie z różnych maszyn.",
                  tags: [
                    "Event sourcing",
                    "Sync",
                  ],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Aplikacja desktopowa i przeglądarkowa do danych mapy Europa Universalis V: wbudowane skompresowane zbiory danych, wstępnie obliczone indeksy wyszukiwania i sortowania, filtry kategorii oraz tabele ze zmienianym rozmiarem. [Wypróbuj tutaj](/eu5-locations-db).",
                  tags: [
                    "Native + web",
                    "Indexes",
                  ],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Ta strona: publikacje, fotografia, forum, czat na żywo z połączeniami, eksplorator terenu Minecrafta i dema WebAssembly, obsługiwane przez jeden program w Rust. [Jak jest zbudowana](/about-blog).",
                  tags: [
                    "Monorepo",
                    "Self-hosted",
                  ],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "Generator i formater UUIDv4 z wyjściem do terminala i pliku, opcjonalnym sprawdzaniem duplikatów oraz formatowaniem zgodnym z Pythonem i JSON; we wstępnym lokalnym teście około trzy razy szybszy niż uuidgen.",
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
          heading: "Pozostałe prace",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: [poprawka do projektu źródłowego](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) serwera Minecraft napisanego w Rust, zachowująca ustawienia świata głównego z Paper przy imporcie i zapisie; czeka na recenzję.",
                "**Narzędzia do map Minecrafta**: wtyczka Paper i usługi w Rust stojące za [eksploratorem terenu](/minecraft) na tej stronie, w tym zwarty binarny format kafelków biomów i przewidywanie biomów na podstawie ziarna świata.",
                "**Synchronizacja systemów plików**: program w Rust do dwukierunkowej synchronizacji katalogów i czyszczenia artefaktów kompilacji między różnymi systemami plików, z próbnymi przebiegami, dziennikami operacji, opcjonalnymi sumami kontrolnymi i zabezpieczeniami przed niezamierzonym masowym usuwaniem.",
                "**impulsr**: narzędzie do transkrypcji YouTube i zbierania komentarzy, które tworzy streszczenia modelem językowym na potrzeby marketingu.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Jak buduję",
          blocks: [
            {
              kind: "paragraph",
              text: "Wolę małe systemy, które można zbadać: jeden program tam, gdzie jeden wystarczy, jawny limit każdej kolejki i pamięci podręcznej oraz formaty danych dobrane do wzorca dostępu, a nie z przyzwyczajenia.",
            },
            {
              kind: "paragraph",
              text: "Pomiary przed twierdzeniami; [notatki o wydajności](/about-blog/performance) tej strony pokazują, jak są wykonywane.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "O mnie",
      title: "Życiorys",
      lede: "Wykształcenie, badania, kwalifikacje i zainteresowania poza oprogramowaniem.",
      sections: [
        {
          id: "education",
          heading: "Wykształcenie",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Uniwersytet Sungkyunkwan",
                  title: "Dyplom inżyniera, inżynieria oprogramowania",
                  period: "mar 2015 - sie 2023",
                  location: "Suwon, Korea Południowa",
                  summary: "Wydział informatyki i inżynierii komputerowej. W okresie studiów mieści się obowiązkowa służba wojskowa.",
                  highlights: [
                    "Zajęcia z oprogramowania wbudowanego, układów logicznych, systemów operacyjnych, sieci, programowania niskopoziomowego i elektroniki, obok praktyki inżynierii oprogramowania.",
                    "Uniwersytet Sungkyunkwan wywodzi się z 1398 roku jako królewska akademia dynastii Joseon i jest jednym z czołowych uniwersytetów badawczych Korei.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Badania",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Analiza emocji w koreańskich społecznościach internetowych z użyciem modelu NLP dla języka koreańskiego.** Zastosowałem KoBERT do analizy sentymentu i kategoryzacji treści z koreańskich mediów społecznościowych, badając zmiany emocji na początku pandemii COVID-19.",
            },
            {
              kind: "paragraph",
              text: "Projekt otrzymał Brązową nagrodę w uniwersyteckim konkursie prac dyplomowych i projektów, wśród około 60 zgłoszeń.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Nagrody i kwalifikacje",
          blocks: [
            {
              kind: "table",
              columns: [
                "Rok",
                "Wyróżnienie",
                "Szczegóły",
              ],
              rows: [
                [
                  "2023",
                  "**Brązowa nagroda**",
                  "Konkurs prac dyplomowych i projektów Uniwersytetu Sungkyunkwan",
                ],
                [
                  "2023",
                  "**IELTS 8.5 / 9.0**",
                  "CEFR C2",
                ],
                [
                  "2021",
                  "**7. miejsce**",
                  "Hackathon projektów dyplomowych i pomysłów, Koreańskie Towarzystwo Informacji Internetowej",
                ],
                [
                  "2020",
                  "**3. miejsce**",
                  "Hackathon fintech, Narodowy Uniwersytet Nauki i Technologii w Seulu",
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
          heading: "Języki",
          blocks: [
            {
              kind: "list",
              items: [
                "**Koreański**: ojczysty.",
                "**Angielski**: biegłość zawodowa (C2).",
                "**Francuski, niemiecki, hiszpański i mandaryński**: podstawowa znajomość.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "Poza oprogramowaniem",
          blocks: [
            {
              kind: "list",
              items: [
                "**Fotografia**: fotografia krajobrazowa i portretowa od 2010 roku, około 30 000 zarchiwizowanych klatek. Wybór jest w [galerii](/photographs).",
                "**Dziennikarstwo**: reporter studencki w Sungkyun Times, anglojęzycznym magazynie uniwersytetu, piszący o technologii, bezpieczeństwie i sprawach międzynarodowych.",
                "**Nauczanie i wolontariat**: uczyłem angielskiego w ośrodku społecznościowym i byłem wolontariuszem w sklepie z używanymi rzeczami prowadzonym przez osoby z niepełnosprawnościami.",
                "**Tłumaczenia**: tłumaczenia ustne i pisemne dla instytucji publicznych; zobacz [Doświadczenie](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
