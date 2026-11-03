import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "November 2026",
  chrome: {
    overview: "Überblick",
    updated: "Zuletzt aktualisiert",
    onThisPage: "Auf dieser Seite",
    sectionNavigation: "Über-mich-Seiten",
    previous: "Zurück",
    next: "Weiter",
    readingTime: "{minutes} Min. Lesezeit",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "Über mich",
      lede: "Backend- und Infrastrukturentwickler mit Schwerpunkt auf Rust, PostgreSQL und Cloud-Plattformen. Ich entwickle Dienste, die die darunterliegende Hardware effektiv nutzen, und kümmere mich um Bereitstellung, Monitoring und Sicherheit, die sie am Laufen halten.",
      sections: [
        {
          id: "introduction",
          heading: "Einleitung",
          blocks: [
            {
              kind: "paragraph",
              text: "Ich bin koreanischer Softwareentwickler und lebe in Colorado. Bei **Soundpatrol** arbeite ich als Rust-Entwickler an Cybersicherheit, DevOps, Webdiensten und Entwicklerwerkzeugen, vor allem mit Rust, Python, PostgreSQL, Kubernetes und Google Cloud.",
            },
            {
              kind: "paragraph",
              text: "Davor leitete ich Backend und Infrastruktur für einen digitalen Gebäudezwilling von Samsung C&T, wirkte an den Anwendungs-Backends von Hyundai, Kia und Genesis mit und baute die Datenerfassung für eine YouTube-Analyseplattform auf. Seit 2023 konzentriert sich meine Arbeit auf Backend-Dienste, Datenpipelines und die Infrastruktur darunter.",
            },
            {
              kind: "paragraph",
              text: "Ich bin überzeugt, dass Dienste moderne Hardware und Programmiersprachen effektiv nutzen sollten. In der Praxis heißt das: sorgfältiges Schemadesign, begrenzte Warteschlangen und Caches, explizite Fehlerbehandlung und Bereitstellungen, die sich prüfen und wiederherstellen lassen. Ich helfe Kolleginnen und Kollegen gern beim Einstieg in Rust und möchte mit Menschen arbeiten, die stolz auf ihr Handwerk sind.",
            },
          ],
        },
        {
          id: "results",
          heading: "Ausgewählte Ergebnisse",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97 %",
                  count: 97,
                  suffix: " %",
                  label: "Geringere monatliche Cloud-Kosten beim digitalen Zwilling von Samsung C&T, von etwa 5.000 auf 150 US-Dollar",
                },
                {
                  value: "80",
                  count: 80,
                  label: "REST-Endpunkte für den digitalen Zwilling, mit Fachlogik aus dem Bauingenieurwesen",
                },
                {
                  value: "9 Monate",
                  count: 9,
                  suffix: " Monate",
                  label: "Produktivbetrieb des digitalen Zwillings ohne Laufzeitabbruch",
                },
                {
                  value: "+20 %",
                  count: 20,
                  prefix: "+",
                  suffix: " %",
                  label: "Mehr Durchsatz bei gemischten Lesezugriffen im Backend dieser Website durch [PGO und BOLT](/about-blog/performance)",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Statisch gelinkte Service-Container aus mehrstufigen Builds",
                },
                {
                  value: "30.000",
                  label: "Codezeilen für das Backend des digitalen Zwillings, rund 75 % des Codes",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Werkzeugkasten",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Sprachen",
                  items: ["Rust", "Python", "TypeScript", "SQL", "Java", "C", "C++"],
                },
                {
                  label: "Backend",
                  items: ["Axum", "Tokio", "Diesel", "tokio-postgres", "Spring Boot", "FastAPI", "WebSockets", "gRPC", "Protocol Buffers"],
                },
                {
                  label: "Daten",
                  items: ["PostgreSQL", "Valkey / Redis", "SQLite / Turso", "MySQL", "FAISS", "tantivy"],
                },
                {
                  label: "Infrastruktur",
                  items: ["Kubernetes", "Helm", "Docker", "OpenTofu / Terraform", "GitHub Actions", "Google Cloud", "AWS", "Linux"],
                },
                {
                  label: "Zuverlässigkeit und Sicherheit",
                  items: ["OpenTelemetry", "IAM", "TLS", "Network policy", "Credential lifecycle", "Incident response"],
                },
                {
                  label: "Oberflächen und Geräte",
                  items: ["SolidJS", "Leptos", "Slint", "WebAssembly", "WebRTC", "BACnet", "Modbus"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "Mehr über mich",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Berufserfahrung",
                  href: "/about/experience",
                  meta: "2016 - heute",
                  text: "Stationen von Soundpatrol bis zu meinem ersten Praktikum, jeweils mit den zugehörigen Projekten und Ergebnissen.",
                },
                {
                  title: "Projekte",
                  href: "/about/projects",
                  meta: "Rust · Open Source",
                  text: "Sillok, EU5 Location DB, diese Website und weitere unabhängige und Open-Source-Arbeiten.",
                },
                {
                  title: "Werdegang",
                  href: "/about/background",
                  meta: "Studium · Forschung",
                  text: "Studium, veröffentlichte Forschung, Auszeichnungen, Sprachen und Tätigkeiten außerhalb der Softwareentwicklung.",
                },
                {
                  title: "So funktioniert diese Website",
                  href: "/about-blog",
                  meta: "Technologie-Stack",
                  text: "Host, Backend, Frontend, Subsysteme und Leistungsmessungen hinter cyhdev.com.",
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
                "**E-Mail**: [younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**: [github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**: [linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "Per E-Mail bin ich am zuverlässigsten erreichbar. Wer dem Minecraft-Server beitreten möchte, kann das gern in derselben Nachricht erwähnen.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "Über mich",
      title: "Berufserfahrung",
      lede: "Backend-, Infrastruktur- und Sicherheitsarbeit bei einem US-Start-up, für koreanische Großunternehmen und in einem frühen Praktikum, gefolgt von den Tätigkeiten vor der Softwareentwicklung.",
      sections: [
        {
          id: "software",
          heading: "Softwareentwicklung",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Rust-Entwickler",
                  period: "Mai 2026 - heute",
                  location: "Vereinigte Staaten · Remote",
                  summary: "Cybersicherheit, DevOps, Webdienstentwicklung und Entwicklerwerkzeuge für ein Unternehmen zur Musik- und Audioerkennung.",
                  highlights: [
                    "Wartung der Kubernetes-Infrastruktur auf Google Cloud, von Infrastructure as Code und geprüften Bereitstellungsabläufen mit kurzlebigen, auf die jeweilige Umgebung beschränkten Zugangsdaten.",
                    "Planung und Durchführung von Migrationen produktiver Datenspeicher, mit Prüfung des migrierten Zustands und geprobter Wiederherstellung vor jeder Umstellung.",
                    "Konsolidierung von Metriken, Logs und Traces auf OpenTelemetry sowie Entwicklung von Rust-Werkzeugen für Dienstzustand, Diagnose und Kostentransparenz.",
                    "Arbeit an der Sicherheitsbereitschaft: Zugriffsverwaltung, Rotation von Zugangsdaten und Überwachung ihres Ablaufs, Richtlinien, Reaktion auf Sicherheitsvorfälle und Audit-Nachweise.",
                    "Entwicklung interner Dienste und Werkzeuge in Rust und Python für Betriebs- und Supportabläufe.",
                  ],
                  tags: ["Rust", "Python", "PostgreSQL", "Kubernetes", "Google Cloud", "OpenTofu", "OpenTelemetry"],
                },
                {
                  organization: "GenesisNest",
                  title: "Softwareentwickler",
                  period: "Jan. 2025 - Juli 2025",
                  location: "Seongnam, Südkorea",
                  summary: "Backend-Dienste und Infrastrukturwerkzeuge für interne Plattformen und die offizielle Fan-App einer K-Pop-Gruppe.",
                  highlights: [
                    "Steuerte APIs, Fehlerbehebungen und Integrationen zu den Backends der offiziellen Apps von Hyundai, Kia und Genesis bei, arbeitete in einer großen Java-Codebasis und stimmte mich mit Dienstleistern für Services ab, die zig Millionen Menschen in Asien und Europa nutzen.",
                    "Baute Internationalisierungspipelines in Rust und Python, die übersetzte Inhalte transformieren und in Unternehmensdatenbanken laden, und stimmte mich dabei mit den Datenbankadministratoren ab, die für mit Fabriksystemen verbundene Datensätze zuständig sind.",
                    "Entwickelte Werkzeuge zur Infrastrukturverwaltung für interne Dienste.",
                  ],
                  tags: ["Java", "Spring Boot", "Rust", "Python"],
                },
                {
                  organization: "pampam Inc.",
                  title: "Softwareentwickler auf Vertragsbasis",
                  period: "Nov. 2024 - Dez. 2024",
                  location: "Seoul, Südkorea",
                  highlights: [
                    "Entwarf das PostgreSQL-Schema und das Rust-Backend einer KI-gestützten YouTube-Analyseplattform und leitete die Bereitstellung auf AWS und Google Cloud.",
                    "Baute Erfassungsabläufe über die YouTube-API für zig Millionen Kommentare und Nutzer, die Kanaldaten, Transkripte und Seiteninhalte zusammenführten.",
                    "Integrierte gehostete Sprachmodelle und auf dem Server betriebene Open-Source-Modelle, um Zusammenfassungen und Marketing-Erkenntnisse zu erzeugen.",
                  ],
                  tags: ["Rust", "PostgreSQL", "AWS", "Google Cloud"],
                },
                {
                  organization: "EAN Technology Co. Ltd.",
                  title: "Leitender Backend-Softwareentwickler",
                  period: "Aug. 2023 - Aug. 2024",
                  location: "Seoul, Südkorea",
                  summary: "Leitete die Entwicklung von Backend, Datenbank und Cloud-Infrastruktur für zwei Gebäude- und Energieplattformen.",
                  highlights: [],
                  projects: [
                    {
                      name: "Digitaler Zwilling des Global Engineering Center von Samsung C&T",
                      highlights: [
                        "Ersetzte ein Express-Backend und eine Cloud-Microservice-Struktur durch einen optimierten Rust/Axum-Monolithen und senkte die monatlichen Infrastrukturkosten von etwa **5.000 auf 150 US-Dollar**.",
                        "Schrieb etwa **30.000 Zeilen**, rund 75 % des Backends, darunter **80 REST-Endpunkte** mit Fachlogik aus dem Bauingenieurwesen, Datenintegration und Echtzeitfunktionen.",
                        "Senkte die P99-Latenz vieler Anfragepfade von mehreren Sekunden auf ein- oder zweistellige Millisekundenwerte, auf einer Umgebung mit zwei Kernen und 4 GB, durch asynchrones Rust, Multithreading, Caching sowie Optimierung von PostgreSQL-Schema und -Abfragen.",
                        "Integrierte vor Ort in der Zentrale von Samsung C&T mehrere Tausend Sensoren, Zähler sowie HLK- und Wassersystemsteuerungen über BACnet und Modbus.",
                        "Neun Monate Produktivbetrieb ohne Laufzeitabbruch, bei einer Abdeckung der Fehlerbehandlung von rund 90 %.",
                        "Baute CI/CD mit GitHub-Runnern, AWS CodeDeploy und Docker auf; statische, auf den Prozessor abgestimmte Builds ergaben portable Container von etwa 20 MB.",
                        "Band die Inferenz von Vorhersagemodellen über ECS, EC2, SQS, SNS, S3 und SageMaker an, setzte die Sicherheitsanforderungen von Samsung SDS um und integrierte das Backend mit einer Unreal-Engine-4-Oberfläche eines Partnerstudios.",
                      ],
                    },
                    {
                      name: "KI-gestützter Energiemanagementdienst",
                      highlights: [
                        "Entwarf das PostgreSQL-Schema und die Rust/Axum-API von Grund auf und schrieb etwa **25.000 Zeilen**, rund 70 % des Backends.",
                        "Entschied mich für direktes SQL über tokio-postgres statt eines ORM, um Abfragen und Kompilierzeiten im Griff zu behalten, und nutzte dabei Erfahrungen aus dem digitalen Zwilling.",
                        "Half zwei Entwicklern mit Node.js- und Java-Hintergrund, innerhalb von etwa einem Monat in Rust produktiv zu werden, in einem neunköpfigen Team aus Backend-, Frontend- und Data-Science-Entwicklern.",
                      ],
                    },
                  ],
                  tags: ["Rust", "Axum", "PostgreSQL", "AWS", "Docker", "BACnet", "Modbus"],
                },
                {
                  organization: "Artifyc Inc.",
                  title: "Softwareentwickler im Praktikum",
                  period: "Aug. 2022 - März 2023",
                  location: "Austin, Texas · Remote",
                  highlights: [
                    "Entwickelte einen Python-Discord-Bot, der über die Reddit-API Anfragen für Kunstaufträge fand und in einem Community-Kanal veröffentlichte, alle fünf Minuten auf AWS Lambda ausgeführt.",
                    "Implementierte Lambda-Logik für Währungsumrechnung, Abrechnung und Aufschlagsberechnung über verschiedene Rechtsräume hinweg.",
                    "Trug Überarbeitungen am React-Frontend zu Layout, Assets und Texten bei.",
                  ],
                  tags: ["Python", "AWS Lambda", "React"],
                },
              ],
            },
          ],
        },
        {
          id: "other",
          heading: "Vor der Softwareentwicklung",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Korea Institute of Maritime Science and Technology",
                  title: "Übersetzer und Dolmetscher",
                  period: "Juni - Juli 2023, Aug. 2024",
                  location: "Seoul, Südkorea · Auf Vertragsbasis",
                  highlights: [
                    "Dolmetschte auf Englisch, Koreanisch und Norwegisch bei einer Konferenz zur maritimen Forschung, an der der norwegische Botschafter in Korea, ein norwegischer Staatssekretär, die Institutsleitung und der Vorsitzende der Dongwon Group teilnahmen.",
                    "Übersetzte Reden und bereitete Mitarbeitende und den Direktor des Instituts auf eine internationale Konferenz sowie auf eine Veranstaltung des US-Energieministeriums zu einem gemeinsamen Projekt vor.",
                  ],
                },
                {
                  organization: "Heer der Republik Korea · 2. US-Infanteriedivision",
                  title: "Sergeant, KATUSA-Verbindungssoldat und Ausbilder",
                  period: "März 2016 - Dez. 2017",
                  location: "Dongducheon, Südkorea",
                  highlights: [
                    "Diente im Stabsbataillon der Division im Rahmen des Programms [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army), in einem gemeinsamen koreanisch-amerikanischen Umfeld.",
                    "Bildete rund 6.000 amerikanische und koreanische Armeeangehörige in Kursen zu Fitness, Kultur, Geschichte und Prävention aus und wickelte ihre Aufnahme ab.",
                    "Führte ein 13-köpfiges Team und hielt die Einsatzbereitschaft während der [Krise von 2017](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) auf der Halbinsel aufrecht.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Lagerarbeiter",
                  period: "2020 - 2022",
                  location: "Südkorea · Gelegentliche Verträge",
                  highlights: [
                    "Nachtschichten im Lager neben dem Studium.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "Über mich",
      title: "Projekte",
      lede: "Unabhängige und Open-Source-Arbeiten, überwiegend in Rust. Interaktive Demos gibt es auf der [Projektseite](/projects).",
      sections: [
        {
          id: "featured",
          heading: "Ausgewählte Projekte",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "Eine Arbeitschronik für die Kommandozeile. Ziele, erledigte Aufgaben, Korrekturen und Widerrufe werden als Append-only-Ereignisse mit indizierten aktuellen Projektionen erfasst, und ein Git-gestütztes Archiv führt Verläufe über mehrere Rechner hinweg zusammen.",
                  tags: ["Event Sourcing", "Synchronisierung"],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Eine Desktop- und Browseranwendung für Kartendaten aus Europa Universalis V, mit eingebetteten komprimierten Datensätzen, vorberechneten Such- und Sortierindizes, kategorialen Filtern und anpassbaren Tabellen. [Hier ausprobieren](/eu5-locations-db).",
                  tags: ["Nativ + Web", "Indizes"],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Diese Website: Veröffentlichungen, Fotografie, ein Forum, Livechat mit Anrufen, ein Minecraft-Geländeexplorer und WebAssembly-Demos, ausgeliefert von einem einzigen Rust-Binary. [So ist sie aufgebaut](/about-blog).",
                  tags: ["Monorepo", "Selbst gehostet"],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "Ein UUIDv4-Generator und -Formatierer mit Ausgabe im Terminal und in Dateien, optionaler Duplikatprüfung und Python- sowie JSON-kompatibler Formatierung; in einem frühen lokalen Benchmark etwa dreimal schneller als uuidgen.",
                  tags: ["Durchsatz"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "Weitere Arbeiten",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: ein [Upstream-Fix](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) für den Rust-Minecraft-Server, der Paper-Einstellungen für die Oberwelt beim Import und Speichern erhält; derzeit in Prüfung.",
                "**Minecraft-Kartenwerkzeuge**: ein Paper-Plugin und Rust-Dienste hinter dem [Geländeexplorer](/minecraft) dieser Website, einschließlich eines kompakten binären Kachelformats für Biome und seedbasierter Biomvorhersagen.",
                "**Dateisystem-Synchronisierung**: ein Rust-Werkzeug für die bidirektionale Ordnersynchronisierung und das Aufräumen von Build-Artefakten über fremde Dateisysteme hinweg, mit Probeläufen, Operationsprotokollen, optionalen Prüfsummen und Schutz vor großen unbeabsichtigten Löschungen.",
                "**impulsr**: ein Werkzeug zur YouTube-Transkription und Kommentarsammlung, das mit Sprachmodellen Zusammenfassungen für Marketingzwecke erstellt.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Wie ich entwickle",
          blocks: [
            {
              kind: "paragraph",
              text: "Ich bevorzuge kleine, überprüfbare Systeme: ein Binary, wo ein Binary genügt, eine explizite Grenze für jede Warteschlange und jeden Cache und Datenformate, die nach dem Zugriffsmuster statt aus Gewohnheit gewählt werden.",
            },
            {
              kind: "paragraph",
              text: "Messungen kommen vor Behauptungen. Wenn sich herausstellt, dass ein Benchmark seinen eigenen Client statt des Servers misst, wird auch die Korrektur festgehalten; die [Leistungsnotizen](/about-blog/performance) zu dieser Website sind ein Beispiel dafür.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "Über mich",
      title: "Werdegang",
      lede: "Studium, Forschung, Qualifikationen und Interessen außerhalb der Softwareentwicklung.",
      sections: [
        {
          id: "education",
          heading: "Studium",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Sungkyunkwan University",
                  title: "B.Eng. in Software Engineering",
                  period: "März 2015 - Aug. 2023",
                  location: "Suwon, Südkorea",
                  summary: "Fachbereich Informatik und Ingenieurwesen. Der Studienzeitraum umfasst den verpflichtenden Wehrdienst.",
                  highlights: [
                    "Lehrveranstaltungen zu eingebetteter Software, Logikschaltungen, Betriebssystemen, Netzwerken, hardwarenaher Programmierung und Elektronik, ergänzt durch Methoden der Softwareentwicklung.",
                    "Die Sungkyunkwan University geht auf das Jahr 1398 als königliche Akademie der Joseon-Dynastie zurück und zählt zu den führenden Forschungsuniversitäten Koreas.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Forschung",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Emotionsanalyse sozialer Mediengemeinschaften mit einem NLP-Modell für die koreanische Sprache.** Anwendung von KoBERT zur Sentimentanalyse und Kategorisierung koreanischsprachiger sozialer Medien, mit Blick auf emotionale Veränderungen in der frühen Phase der COVID-19-Pandemie.",
            },
            {
              kind: "paragraph",
              text: "Das Projekt erhielt bei den Preisen der Universität für Abschlussarbeiten und -projekte den Bronze-Preis, unter rund 60 Einreichungen.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Auszeichnungen und Qualifikationen",
          blocks: [
            {
              kind: "table",
              columns: ["Jahr", "Auszeichnung", "Details"],
              rows: [
                ["2023", "**Bronze-Preis**", "Preise der Sungkyunkwan University für Abschlussarbeiten und -projekte"],
                ["2023", "**IELTS 8,5 / 9,0**", "GER C2"],
                ["2021", "**7. Platz**", "Capstone-Design- und Ideen-Hackathon, Korean Society for Internet Information"],
                ["2020", "**3. Platz**", "Fintech-Hackathon, Seoul National University of Science and Technology"],
                ["2018", "**TOEFL 117 / 120**", "GER C2"],
              ],
            },
          ],
        },
        {
          id: "languages",
          heading: "Sprachen",
          blocks: [
            {
              kind: "list",
              items: [
                "**Koreanisch**: Muttersprache.",
                "**Englisch**: verhandlungssicher (C2).",
                "**Französisch, Deutsch, Spanisch und Mandarin**: Grundkenntnisse.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "Außerhalb der Softwareentwicklung",
          blocks: [
            {
              kind: "list",
              items: [
                "**Fotografie**: Landschafts- und Porträtfotografie seit 2010, mit rund 30.000 archivierten Aufnahmen. Eine Auswahl findet sich in der [Galerie](/photographs).",
                "**Journalismus**: studentischer Reporter für die Sungkyun Times, das englischsprachige Magazin der Universität, mit Beiträgen zu Technologie, Sicherheit und internationaler Politik.",
                "**Unterricht und Ehrenamt**: Englischunterricht in einem Gemeindezentrum und ehrenamtliche Mitarbeit in einem Secondhandladen, der von Menschen mit Behinderungen betrieben wird.",
                "**Übersetzung**: Dolmetschen und Übersetzen für öffentliche Einrichtungen; siehe [Berufserfahrung](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
