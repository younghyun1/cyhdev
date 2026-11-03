import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "November 2026",
  chrome: {
    overview: "Überblick",
    updated: "Zuletzt aktualisiert",
    onThisPage: "Auf dieser Seite",
    sectionNavigation: "Seiten zum Technologie-Stack",
    previous: "Zurück",
    next: "Weiter",
    readingTime: "{minutes} Min. Lesezeit",
  },
  pages: {
    overview: {
      eyebrow: "Über diese Website",
      title: "Technologie-Stack des Blogs",
      lede: "Ein Rust-Binary, eine PostgreSQL-Datenbank, ein kleiner Rechner. Diese Seiten beschreiben, wie die Website gebaut, gehostet, abgesichert und gemessen wird und warum sie so aufgebaut ist.",
      sections: [
        {
          id: "at-a-glance",
          heading: "Auf einen Blick",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53.889 req/s",
                  label: "Medianer Durchsatz bei gemischten öffentlichen Lesezugriffen mit PGO und BOLT auf vier Serverkernen",
                },
                {
                  value: "~150 µs",
                  label: "Beobachteter Datenbank-Roundtrip über einen UNIX-Socket, gegenüber ~600 µs über localhost-TCP",
                },
                {
                  value: "130 KiB",
                  count: 130,
                  suffix: " KiB",
                  label: "Gzip-Budget für die initialen Frontend-Assets, beim Erstellen des Bundles durchgesetzt",
                },
                {
                  value: "8",
                  count: 8,
                  label: "Oberflächensprachen, die jeweils erst bei Auswahl geladen werden",
                },
                {
                  value: "0,15 ms",
                  label: "Mediane Abschlusszeit einer Kachel über HTTP/2 nach Aktivierung von `TCP_NODELAY`, zuvor 41 ms",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Prozess für TLS, die API, WebSockets, WebRTC-Medien und das eingebettete Frontend",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Anfragepfad",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Browser",
                  detail: "HTTP/2 oder HTTP/1.1 über TLS. Das Frontend wird unter unveränderlichen URLs mit Fingerprint zwischengespeichert.",
                },
                {
                  label: "Zulassung",
                  detail: "TCP-Verbindungen werden vor TLS zugelassen: insgesamt 4.096 und 64 pro Adresse oder IPv6-/64.",
                },
                {
                  label: "rustls",
                  detail: "TLS endet im Rust-Prozess, mit `TCP_NODELAY` auf jedem Socket.",
                },
                {
                  label: "Axum",
                  detail: "Sicherheits-Header, Sitzungen, Ratenbegrenzung, Komprimierung und Größenlimits für den Body, danach ein Feature-Handler.",
                },
                {
                  label: "Cache · Diesel",
                  detail: "Begrenzte In-Memory-Caches beantworten häufige Lesezugriffe; alles andere läuft über asynchrones Diesel.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "Auf demselben Host über einen UNIX-Socket, mit asynchroner E/A über io_uring.",
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
                  meta: "Host · Gentoo · Netzwerk",
                  text: "Der Rechner, Betriebssystem und Dateisystem, das Netzwerk und warum es keinen Reverse-Proxy gibt.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "Der Service-Stack, das Datenmodell, Laufzeitgrenzen, Sicherheits-Header und API-Verträge.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "Die Anwendungsshell, das Ladebudget, Lokalisierung, Designsystem und Animationen.",
                },
                {
                  title: "Subsysteme",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "Livechat mit prozessinterner SFU, der Minecraft-Explorer, Geo-IP-Abfragen und WebAssembly.",
                },
                {
                  title: "Leistung",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Optimierte Builds, der Benchmark, der das Falsche gemessen hat, und die korrigierten Zahlen.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Wie Änderungen geprüft werden",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy für native und WebAssembly-Ziele als Prüfschritt während der Implementierung.",
                "Ein Validierungs-Workflow für Formatierung, native Unit-Tests, Abweichungen vom OpenAPI-Vertrag sowie Typprüfung, Linting, Unit-Tests und Build des Frontends.",
                "Playwright auf Chromium und WebKit mit axe-Barrierefreiheitsprüfungen und mobilen Reflow-Prüfungen bis hinunter zu 320 CSS-Pixeln.",
                "Integrations- und Migrations-Rollback-Tests gegen eine Wegwerf-PostgreSQL-Instanz sowie ein lokaler Secret-Scan mit gitleaks.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Technologie-Stack",
      title: "Infrastruktur",
      lede: "Ein kleiner Rechner zu Hause, eine quellbasierte Linux-Distribution und keine Schicht, die der Datenverkehr nicht braucht.",
      sections: [
        {
          id: "host",
          heading: "Der Host",
          blocks: [
            {
              kind: "paragraph",
              text: "Die Website läuft auf einem [Miniserver](https://store.minisforum.com/products/minisforum-um690l-slim) bei mir zu Hause: ein mobiler Ryzen-9-Prozessor mit acht Kernen und bis zu 4,9 GHz, 32 GB RAM und eine 1-TB-NVMe-SSD. Mit rund 400 US-Dollar kostet er weniger als ein Jahr spürbar schwächerer Cloud-Hardware und spiegelt meine Begeisterung für selbst gehostete Dienste wider.",
            },
            {
              kind: "paragraph",
              text: "Darauf laufen der kombinierte Backend- und Frontend-Server, PostgreSQL und ein Minecraft-Server. Wer mitspielen möchte, kann mir gern eine E-Mail schreiben.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo und btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) ist für einen Server nicht die naheliegende Wahl; beruflich würde ich Debian Stable auf ext4 nehmen. Hier kompiliere ich Pakete gern selbst, optimiert für genau diesen Prozessor.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) bietet Snapshots, Komprimierung und Copy-on-Write, doch Copy-on-Write fragmentiert Datenbankdateien stark. Die Datenverzeichnisse von PostgreSQL und Minecraft sind deshalb davon ausgenommen. Ein [Dateisystem-Benchmark](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) zeigt btrfs in der Standardkonfiguration hinter ext4 und XFS; ich vermute, dass das Deaktivieren von Copy-on-Write den Großteil dieses Abstands schließt, was eine interessante Messung wäre.",
            },
          ],
        },
        {
          id: "network",
          heading: "Netzwerk",
          blocks: [
            {
              kind: "paragraph",
              text: "Der Datenverkehr kommt über einen kabelgebundenen Xfinity-Anschluss mit 1 Gbit/s; dem Modem des Anbieters fehlt 2,5-Gbit-Ethernet, was diese Website nicht bemerken wird. Route 53 hostet das DNS.",
            },
            {
              kind: "paragraph",
              text: "Auf dem Rechner gibt es keinen Reverse-Proxy, keine Container-Runtime und kein Service-Mesh. PostgreSQL läuft auf dem Betriebssystem, und ein einziges Rust-Binary liefert Frontend und API aus. Unverschlüsselte HTTP-Anfragen leitet derselbe Prozess auf HTTPS um.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "UNIX-Sockets",
          blocks: [
            {
              kind: "paragraph",
              text: "Der Server erreicht PostgreSQL über einen UNIX-Socket statt über localhost-TCP und umgeht damit den Netzwerk-Stack vollständig. [Benchmarks](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) zeigen nahezu verdoppelte Transaktionsraten; hier habe ich Roundtrips von nur **150 µs** beobachtet, gegenüber etwa **600 µs** über localhost. Das ist altmodisch und für die meisten Cloud-Bereitstellungen irrelevant, und es ist schneller.",
            },
            {
              kind: "paragraph",
              text: "PostgreSQL 18, erschienen im September 2025, brachte [asynchrone E/A](https://pganalyze.com/blog/postgres-18-async-io), und io_uring ist hier aktiviert, was Lesezugriffe spürbar beschleunigt.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Build und Bereitstellung",
          blocks: [
            {
              kind: "list",
              items: [
                "Der Frontend-Build ist zusammen mit vorkomprimierten zstd- und gzip-Kopien in das Backend-Binary eingebettet, sodass in Produktion kein Node.js-Prozess läuft.",
                "Optimierte Builds lassen sich mit profilgesteuerter Optimierung trainieren und nach dem Linken mit BOLT optimieren; siehe [Leistung](/about-blog/performance).",
                "Zugangsdaten, Zertifikate und Laufzeitdaten bleiben außerhalb von Git, und ein gitleaks-Scan prüft den Arbeitsbaum, bevor Änderungen den Rechner verlassen.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Technologie-Stack",
      title: "Backend",
      lede: "Rust auf Axum und Tokio, PostgreSQL über asynchrones Diesel und eine explizite Grenze für alles, was ein Client offen halten kann.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "Laufzeit", items: ["Axum", "Tokio", "hyper", "axum-server", "rustls", "mimalloc"] },
                { label: "Daten", items: ["PostgreSQL 18", "Diesel", "diesel-async", "tantivy", "bitcode", "zstd"] },
                { label: "Integrationen", items: ["openidconnect", "lettre", "Amazon S3", "webrtc-rs", "image + AVIF"] },
                { label: "API-Verträge", items: ["utoipa", "OpenAPI", "Swagger UI", "Generated TypeScript"] },
              ],
            },
            {
              kind: "paragraph",
              text: "Der Dienst ist ein einzelner Prozess mit gemeinsamem Zustand: Sitzungen im Arbeitsspeicher, Cache-Synchronisierung beim Start und geplante Wartungsaufgaben. mimalloc ist der globale Allokator; hat ein Rechner echte Kerne, lohnt es sich, das Verhalten des Allokators ernst zu nehmen.",
            },
          ],
        },
        {
          id: "data",
          heading: "Datenmodell",
          blocks: [
            {
              kind: "paragraph",
              text: "In den koreanischen Start-ups und Unternehmen, die ich kennengelernt habe, galten MySQL und MariaDB als die einzigen ernsthaften relationalen Datenbanken. PostgreSQL hat sie in den Bereichen, die mir wichtig sind, längst überholt, von nativen UUIDs bis zu binärem JSON.",
            },
            {
              kind: "list",
              items: [
                "**Beiträge** enthalten Slugs, Zusammenfassungen, JSONB-Metadaten, Veröffentlichungsstatus, denormalisierte Zähler und Tag-Zuordnungen. Kommentare bilden Threads über einen optionalen Elternverweis, und Stimmen für Beiträge und Kommentare liegen in getrennten Tabellen.",
                "**Konten** speichern Land und Sprache, Rollen- und Berechtigungstabellen sowie versionierte Profilbilder in einer eigenen Tabelle statt in der häufig gelesenen Benutzerzeile.",
                "**Schlüssel** sind zeitlich geordnete UUIDv7-Werte: nicht aufzählbar wie fortlaufende IDs und deutlich freundlicher zur B-Baum-Lokalität als zufällige UUIDv4.",
                "Die **Suche** nutzt einen auf der Festplatte persistierten tantivy-Index über Beitragstitel und Tags, der mit PostgreSQL konsistent gehalten wird; PostgreSQL bleibt maßgeblich.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Laufzeitgrenzen",
          blocks: [
            {
              kind: "list",
              items: [
                "Verbindungen werden vor TLS über einen gemeinsamen Begrenzer zugelassen: 4.096 offene Verbindungen und 64 pro IPv4-Adresse oder IPv6-/64. Abgewiesene Sockets werden ohne Handshake geschlossen.",
                "Header von HTTP/1-Anfragen müssen innerhalb von 20 Sekunden eintreffen; HTTP/2-Verbindungen werden alle 30 Sekunden angepingt und erlauben 128 gleichzeitige Streams.",
                "Jeder Cache hat eine feste Obergrenze und eine maßgebliche Quelle, auf die er beim Lesen durchgreifen kann; keiner wächst mit Schlüsseln, die ein Client steuert. Der Cache für WebAssembly-Bundles fasst beispielsweise höchstens 256 MiB.",
                "Blockierende Arbeit wie die Bildkodierung läuft in begrenzten Worker-Pools statt auf Tokio-Workern.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Sicherheits-Header",
          blocks: [
            {
              kind: "paragraph",
              text: "Die Content-Security-Policy wird einmal beim Start erstellt. Inline-Skripte sind nur über SHA-256-Hashes erlaubt, die aus der ins Binary kompilierten `index.html` berechnet werden, sodass das Theme-Skript vor dem ersten Rendern nicht von der Richtlinie abweichen kann. Der Hauptursprung instanziiert nie WebAssembly: Demos und die EU5-Anwendung laufen in Sandbox-Frames mit eigenen Richtlinien.",
            },
            {
              kind: "paragraph",
              text: "Sitzungscookies sind `Secure` und `HttpOnly`, Superuser-Operationen prüfen Rollen auf dem Server, Konten können OpenID-Connect-Identitäten verknüpfen, und Swagger UI erfordert in Produktion eine Authentifizierung.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Verträge und Assets",
          blocks: [
            {
              kind: "list",
              items: [
                "Handler tragen OpenAPI-Annotationen, die TypeScript-Clients des Frontends werden daraus generiert, und die CI schlägt fehl, wenn die generierten Dateien abweichen.",
                "Das Vite-Manifest ist zusammen mit den Assets eingebettet. Nur im Manifest aufgeführte Dateien mit Fingerprint erhalten eine einjährige `immutable`-Cache-Richtlinie; HTML wird immer revalidiert.",
                "Fehlende Asset-Pfade liefern 404 mit `no-store` statt der Anwendungsshell, sodass eine veraltete Seite kein HTML unter einer Skript-URL zwischenspeichern kann.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Technologie-Stack",
      title: "Frontend",
      lede: "Eine clientseitig gerenderte SolidJS-2-Anwendung, kompiliert zu statischen Assets und in das Server-Binary eingebettet.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "Anwendung", items: ["SolidJS 2", "Solid Router 2", "TypeScript", "Vite 8"] },
                { label: "Styling", items: ["Tailwind CSS 4", "IBM Plex Sans", "IBM Plex Mono", "OKLCH"] },
                { label: "Komponenten", items: ["Toast UI Editor", "Leaflet", "Chart.js", "highlight.js", "DOMPurify"] },
                { label: "Tests", items: ["Vitest", "Playwright", "axe-core"] },
              ],
            },
            {
              kind: "paragraph",
              text: "Die feingranularen Signale von Solid aktualisieren nur die DOM-Knoten, die von geänderten Werten abhängen, und mit den Release Candidates von 2.0 können asynchrone Lesezugriffe bis zur nächsten Ladegrenze suspendieren. Der Router lädt jede Seite verzögert, und es gibt keinen JavaScript-Server in Produktion: Das Rust-Binary liefert statische Dateien aus.",
            },
          ],
        },
        {
          id: "budget",
          heading: "Ladebudget",
          blocks: [
            {
              kind: "list",
              items: [
                "Ein Vite-Plugin verfolgt statische Importe ab dem Entry-Chunk und lässt den Build fehlschlagen, wenn die initialen Skripte und Styles **130 KiB** gzip überschreiten.",
                "Nur die englischen Oberflächentexte sind im initialen Chunk enthalten. Andere Sprachen und die „Über“-Seiten jeder Sprache sind separate Chunks, die bei Auswahl geladen werden.",
                "Gedrosselte mobile Leistungstests zielen auf einen LCP von höchstens 2,5 s, Interaktionen von höchstens 200 ms und einen CLS von höchstens 0,1.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Lokalisierung",
          blocks: [
            {
              kind: "paragraph",
              text: "Die Oberfläche unterstützt Englisch, Koreanisch, Französisch, Spanisch, vereinfachtes und traditionelles Chinesisch, Japanisch und Deutsch. Ein Katalog mit rund 650 Schlüsseln wird gegen das Register des Backends validiert, und Platzhalter müssen die Übersetzung überstehen. Diese „Über“-Seiten sind typisierte Datenmodule, sodass eine Übersetzung keinen Abschnitt stillschweigend auslassen kann, den es auf Englisch gibt.",
            },
          ],
        },
        {
          id: "design",
          heading: "Designsystem",
          blocks: [
            {
              kind: "paragraph",
              text: "Die Gestaltung ist warm und technisch: cremefarbenes Papier oder reines Schwarz, haarfeine Rahmen, Monospace für alles Technische und jeweils nur eine Akzentfarbe. Farben sind semantische CSS-Variablen, sodass Komponenten nie eigene Dark-Mode-Varianten brauchen. Akzentvorgaben sind in OKLCH definiert, das die wahrgenommene Helligkeit über Farbtöne hinweg konstant hält, und heller oder dunkler Modus sowie Akzent werden vor dem ersten Rendern angewendet.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Animationen",
          blocks: [
            {
              kind: "list",
              items: [
                "Das Einblenden von Abschnitten und der Lesefortschrittsbalken nutzen scrollgesteuerte CSS-Animationen, die, wo unterstützt, außerhalb des Hauptthreads laufen.",
                "Statistiken zählen über eine registrierte `@property`-Ganzzahl hoch, wobei der Endwert für Hilfstechnologien im Markup steht.",
                "Der Themenwechsel breitet sich über die View Transitions API vom Umschalter aus.",
                "Das Höhenlinienfeld der Startseite ist ein kleiner WebGL-Shader, der geladen wird, sobald die Seite im Leerlauf ist, nur in leistungsfähigen Desktop-Browsern läuft, seine Bildrate begrenzt und außerhalb des sichtbaren Bereichs pausiert.",
                "Alles Dekorative entfällt, wenn das System reduzierte Bewegung anfordert.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Technologie-Stack",
      title: "Subsysteme",
      lede: "Die Teile der Website, die interessanter sind, als ein Blog es unbedingt bräuchte.",
      sections: [
        {
          id: "live-chat",
          heading: "Livechat und Anrufe",
          blocks: [
            {
              kind: "paragraph",
              text: "Der Chat läuft über einen WebSocket mit kompaktem Binärcodec, Moderation und begrenztem Verlauf. Sprach- und Videoanrufe laufen über eine Selective Forwarding Unit (SFU) im Axum-Prozess auf Basis von webrtc-rs: Jeder Browser hält eine einzige Peer-Verbindung, und die SFU leitet codierte Medien ohne Transkodierung weiter, über begrenzte Kanäle pro Veröffentlichung und eine Teilnehmerobergrenze. [Livechat öffnen](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Minecraft-Geländeexplorer",
          blocks: [
            {
              kind: "paragraph",
              text: "Ein Paper-Plugin stellt einen UNIX-Socket für denselben Benutzer bereit, über den begrenzte Weltabfragen laufen, mit jeweils einer Operation und strikten Limits für die Frame-Größe. Der [Explorer](/minecraft) zeichnet squaremap-Kacheln mit einem eigenen Leaflet-Client, statt das JavaScript der generierten Karte auszuführen.",
            },
            {
              kind: "paragraph",
              text: "Biom-Overlays nutzen **CYBM**, ein kompaktes binäres Kachelformat, das gzip-komprimiert ausgeliefert wird, oder ein PNG, dessen privater `cyBM`-Chunk die exakten Biom-Identitäten enthält, sodass beim Überfahren mit der Maus nie anhand von Pixelfarben geraten wird. Seedbasierte Vorhersagen für unerkundetes Gelände stammen von einem separaten Weltgenerierungs-Worker auf Basis von Pumpkin.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Geo-IP-Abfragen",
          blocks: [
            {
              kind: "paragraph",
              text: "Die [Besucherkarte](/visitor-board) und die [Geo-IP-Datenbank](/geo-ip-db) lokalisieren Adressen, ohne eine externe API aufzurufen. Ein Vorverarbeitungswerkzeug wandelt die CSV-Dateien von IP2Location LITE in `BTreeMap`s um, deren Schlüssel der Bereichsanfang ist, kodiert sie mit bitcode und komprimiert sie mit zstd auf Stufe 22.",
            },
            {
              kind: "table",
              columns: ["Datensatz", "CSV", "Bundle"],
              numericColumns: [1, 2],
              rows: [
                ["IPv4", "312,5 MB", "24,9 MB"],
                ["IPv6", "794,2 MB", "42,4 MB"],
              ],
            },
            {
              kind: "paragraph",
              text: "Beim Start dekodiert der Server jeweils ein Bundle, um den Spitzenspeicher zu begrenzen, und interniert die wiederholten Ortsnamen. Eine Abfrage findet den letzten Bereich, der bei oder unterhalb der Adresse beginnt, und prüft dessen Endgrenze.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "Hochgeladene [Demo-Bundles](/projects) werden auf der Festplatte zwischengespeichert, anhand ihrer Magic Bytes erkannt, auf 50 MiB entpackt begrenzt und aus einem begrenzten Cache in Sandbox-Frames ausgeliefert. Der Workspace baut außerdem Rust-Demos, darunter ein Breakout-Spiel und einen Raytracer.",
            },
            {
              kind: "paragraph",
              text: "Die [EU5 Location DB](/eu5-locations-db) ist eine nach WebAssembly kompilierte Slint-Anwendung. Die Seite übermittelt ihren hellen oder dunklen Modus über ein kleines Nachrichtenprotokoll, sodass sich die eingebettete Palette mit der Website ändert.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Foto-Pipeline",
          blocks: [
            {
              kind: "paragraph",
              text: "Uploads werden innerhalb fester Pixelgrenzen dekodiert und mit der langsamsten Voreinstellung nach AVIF kodiert, was Verarbeitungszeit gegen kleinere Dateien eintauscht. Orte aus EXIF-Daten lassen sich vor der Veröffentlichung auf einer Karte bestätigen, und es laufen nie mehr als zwei Verarbeitungsjobs gleichzeitig.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Technologie-Stack",
      title: "Leistung",
      lede: "Profilgesteuerte Optimierung und BOLT im Backend, gemessen mit einem Lastgenerator, der nicht selbst der Engpass ist.",
      sections: [
        {
          id: "approach",
          heading: "Vorgehen",
          blocks: [
            {
              kind: "paragraph",
              text: "Optimierte Builds werden mit einer repräsentativen Last trainiert. Ein instrumentiertes Binary zeichnet Profile gegen eine Wegwerf-Testumgebung mit synthetischen PostgreSQL-, SMTP-, S3-, OpenID-Connect- und Minecraft-Endpunkten auf. LLVM baut den Server mit diesen Profilen neu (PGO), und BOLT ordnet anschließend das Code-Layout des finalen Binarys anhand gesampelter Ausführung neu an. Alle Varianten teilen LTO, Frame-Pointer und eine neu gebaute Standardbibliothek.",
            },
          ],
        },
        {
          id: "correction",
          heading: "Der Benchmark, der sich selbst maß",
          blocks: [
            {
              kind: "paragraph",
              text: "Die erste Messreihe erzeugte die Last aus Playwright und schien Regressionen zu zeigen. Sie waren nicht echt: Playwright-API-Aufrufe ohne Scope durchsuchen frühere Testschritte nach ihrem Elternschritt, sodass der Client im Lauf der Messung langsamer wurde. Gegen einen trivialen HTTP-Server fiel der Client allein über neun Stichproben von 2.684 auf 1.314 Anfragen/s.",
            },
            {
              kind: "paragraph",
              text: "Die Lasterzeugung wechselte zu oha. Playwright prüft weiterhin die Antwort jeder Route, aber außerhalb des gemessenen Zeitfensters.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Aufbau",
          blocks: [
            {
              kind: "list",
              items: [
                "Server, Anbieter der Testumgebung und PostgreSQL auf vier Kerne eines AMD Ryzen 7 9800X3D gepinnt; der Lastclient auf die anderen vier.",
                "HTTPS über gepooltes HTTP/1.1 mit 32 Verbindungen, vollständigem Lesen der Antworten und Fristen von zwei Sekunden.",
                "Drei rotierte Runden mit fünf Stichproben pro Variante, jede nach 10.000 Aufwärmanfragen, ausgehend von einem frischen Datenbank-Snapshot.",
                "Insgesamt 27.000.000 gemessene Anfragen; jede lieferte HTTP 200.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Ergebnisse",
          blocks: [
            {
              kind: "bars",
              caption: "Median der Anfragen pro Sekunde über 15 Stichproben pro Variante.",
              groups: [
                {
                  label: "Öffentliche Lesezugriffe über zehn Routen",
                  items: [
                    { label: "Referenz", value: 44874.12, display: "44.874" },
                    { label: "PGO", value: 52925.46, display: "52.925" },
                    { label: "PGO + BOLT", value: 53888.61, display: "53.889", emphasis: true },
                  ],
                },
                {
                  label: "Kleines Health-Check-JSON",
                  items: [
                    { label: "Referenz", value: 272837.41, display: "272.837" },
                    { label: "PGO", value: 308685.26, display: "308.685", emphasis: true },
                    { label: "PGO + BOLT", value: 306276.64, display: "306.277" },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "Tail-Latenz für den Mix aus zehn Routen.",
              columns: ["Variante", "p95 (ms)", "p99 (ms)"],
              numericColumns: [1, 2],
              rows: [
                ["Referenz", "2,380", "2,967"],
                ["PGO", "2,020", "2,524"],
                ["PGO + BOLT", "1,987", "2,483"],
              ],
            },
            {
              kind: "paragraph",
              text: "PGO steigerte den gemischten Durchsatz um **17,9 %** und PGO mit BOLT um **20,1 %**, und beide optimierten Builds gewannen jede rotierte Runde. BOLT brachte bei den gemischten Lesezugriffen 1,8 % mehr als PGO und war auf der trivialen Health-Check-Route 0,8 % langsamer. Das sind Ergebnisse mit vier Kernen; die Kapazität in Produktion hängt auch vom Netzwerk, dem Working Set der Datenbank und dem realen Datenverkehr ab.",
            },
          ],
        },
        {
          id: "transport",
          heading: "Auch kleine Korrekturen zählen",
          blocks: [
            {
              kind: "paragraph",
              text: "Der Standard-Rustls-Acceptor in axum-server setzte `TCP_NODELAY` nicht, sodass in Header- und Daten-Frames aufgeteilte HTTP/2-Antworten auf eine verzögerte Bestätigung warten konnten. Die Aktivierung auf jedem angenommenen Socket veränderte eine 59.571 Byte große Kachelantwort über eine wiederverwendete Verbindung wie folgt.",
            },
            {
              kind: "table",
              columns: ["TCP_NODELAY", "Protokoll", "Erstes Byte (Median)", "Vollständig (Median)"],
              numericColumns: [2, 3],
              rows: [
                ["Aus", "HTTP/2", "0,290 ms", "40,981 ms"],
                ["Ein", "HTTP/2", "0,113 ms", "0,152 ms"],
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
