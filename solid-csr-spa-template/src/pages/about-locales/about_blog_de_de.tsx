import { RepositorySourceMap } from "../../components/RepositorySourceMap";
import { pageStyles } from "../../styles/pageStyles";

export default function AboutBlog() {
  return (
    <main lang="de-DE" class={pageStyles.page}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">
            Technologie-Stack des Blogs
          </h1>
          <p class="text-xs text-ink-muted">
            Zuletzt aktualisiert: 30.08.2026
            <br />
          </p>
        </div>

        <section class="space-y-8">
          {/* 0) On-Prem */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              0) Hostsystem, Betriebssystem, Dateisystem und Netzwerkkonfiguration
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              Die Website läuft auf einem Miniserver bei mir zu Hause, hinter einem kabelgebundenen Xfinity-Anschluss mit 1 Gbit/s. Der{" "}
              <a
                href="https://store.minisforum.com/products/minisforum-um690l-slim"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                Host
              </a>{" "}
              verfügt über einen mobilen Ryzen-9-Prozessor mit acht Kernen und bis zu 4,9 GHz, 32 GB RAM mit 6400 MT/s sowie eine 1-TB-NVMe-SSD. Für rund 400 US-Dollar ist das ein Schnäppchen im Vergleich zu den Kosten für ähnlich viel AWS-Hardware über ein Jahr, die deutlich weniger leistungsfähig wäre. Das spiegelt meine Begeisterung für selbst gehostete Dienste wider. Darauf laufen der kombinierte Backend-Frontend-Server, die PostgreSQL-Datenbank und ein Minecraft-Server. Wenn du mitspielen möchtest, schreib mir eine E-Mail.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-1) Gentoo
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Nicht gerade das Betriebssystem der Wahl für einen Unternehmensserver. Für einen Arbeitsplatz hätte ich einfach Debian Stable mit ext4 und irgendeiner Datenbank-Engine verwendet. Privat tüftle ich jedoch gern und kompiliere Software selbst. Das wunderbare{" "}
              <a
                href="https://www.gentoo.org/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                Gentoo
              </a>{" "}
              -Projekt macht außerdem für Pakete optimierte Builds und Installationen passend zur CPU-Architektur möglich.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-2) btrfs auf einem Host für Datenbank, Backend/Frontend und Minecraft
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              <a
                href="https://en.wikipedia.org/wiki/Btrfs"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                btrfs
              </a>{" "}
              ist ein modernes Dateisystem mit Snapshots, CoW, Komprimierung und vielen weiteren Funktionen. Wegen der durch CoW verursachten starken Fragmentierung eignet es sich jedoch nicht ideal für einen Datenbankserver. Ich habe das abgemildert, indem ich die PostgreSQL- und Minecraft-Datenverzeichnisse von CoW ausgenommen habe.{" "}
              <a
                href="https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                Ein PostgreSQL-Dateisystem-Benchmark
              </a>{" "}
              zeigt tatsächlich, dass btrfs in der Standardeinstellung keine besonders gute Leistung bietet. Ich vermute aber, dass es mit deaktiviertem CoW ext4 und xfs ebenbürtig sein könnte. Das wäre ein interessanter Benchmark.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-3) Interne und externe Netzwerkkonfiguration
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Als ich den 2-Gbit/s-Kabelinternetdienst von Xfinity bestellte, war mir nicht klar, dass der Anbieter so sehr sparen würde, dass das Modem-Router-Gerät tatsächlich <em>kein</em> 2,5-Gbit/s-Ethernet unterstützt. Also müssen es 1 Gbit/s sein; ich kann mir jedoch kaum vorstellen, dass das für meine kleine Website zum Problem wird. Route 53 stellt den DNS-Dienst für meine Domain bereit.
              <br />
              <br />
              Intern brauche ich wirklich weder einen Reverse-Proxy noch Containerisierung oder Werkzeuge für verteilte Dienste. Es läuft einfach eine PostgreSQL-Engine direkt auf dem Betriebssystem und ein Rust-Binary, das zugleich Frontend- und API-Server ist und über einen UNIX-Socket mit der Datenbank verbunden ist. Das ist tatsächlich deutlich schneller als{" "}
              <a
                href="https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                localhost zu verwenden, wie hier beschrieben.
              </a>{" "}
              Der Durchsatz kann sich nahezu verdoppeln, und ohne den überflüssigen Netzwerk-Stack halbiert sich auch die Latenz. Zwischen Datenbank und Server habe ich nur 150 Mikrosekunden gemessen; mit localhost waren es typischerweise etwa 600 Mikrosekunden. Für Cloud-Unternehmenssysteme ist das nicht relevant, aber es ist interessant. Es ist altmodisch. Und schneller! PostgreSQL 18, erschienen im November 2025, führte außerdem asynchrone Ein-/Ausgabe in Form von{" "}
              <a
                href="https://pganalyze.com/blog/postgres-18-async-io"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                io_uring-Unterstützung
              </a>{" "}
              ein, die aktiviert ist. Sie beschleunigt Lesezugriffe deutlich.
            </p>
          </section>

          {/* 1) PostgreSQL */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              1) Daten
            </h2>
            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-1) PostgreSQL 18
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Während meiner Zeit in koreanischen Start-ups und Unternehmen ist mir eine Kultur aufgefallen, in der MySQL oder MariaDB als einzig sinnvolle relationale Datenbank gelten. Warum, habe ich bisher nicht herausgefunden. Einige frühere Vorgesetzte erzählten mir, PostgreSQL sei früher überhaupt nicht als ernsthafte Option angesehen worden. Das ist angesichts der großen Fortschritte bei Leistung, Erweiterbarkeit, Datentypen und Werkzeugen im letzten Jahrzehnt merkwürdig. In vielerlei Hinsicht hat PostgreSQL MySQL wohl überholt, besonders bei UUID-Datentypen und der binären Speicherung von JSON-Daten.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-2) Schema im Überblick (Blog + Authentifizierung)
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Das Schema ist bewusst auf die gute Art unspektakulär: Benutzer, Sitzungen im Arbeitsspeicher, E-Mail-Verifizierungstokens, Tokens zum Zurücksetzen von Passwörtern, Beiträge, Kommentare, Abstimmungstabellen, Tags, Profilbilder und einige Tabellen zur Unterstützung von Geodaten und Internationalisierung. Der Blog besteht nicht einfach aus einer
              <code>posts</code>-Tabelle und einem Stoßgebet. Beiträge enthalten Slugs, Zusammenfassungen, Metadaten als JSONB, Veröffentlichungsstatus, denormalisierte Zähler und Tag-Zuordnungen. Kommentare bilden über eine optionale übergeordnete Kommentar-ID verschachtelte Threads. Stimmen liegen in eigenen Tabellen für Beiträge und Kommentare. Das hält Abfragen einfacher und erspart später ziemlich absurde Fallunterscheidungen.
              <br />
              <br />
              Auch die Authentifizierung ist pragmatisch: Der Benutzerdatensatz speichert Land und Sprache, damit die Website mehr kann, als nur nach einer E-Mail-Adresse zu fragen und dich dann zu vergessen. Rollen- und Berechtigungstabellen verhindern, dass ich mich bei der Zugriffskontrolle in eine Sackgasse manövriere. Profilbilder werden mit Versionen in einer eigenen Tabelle verwaltet, statt sie an den Benutzerdatensatz anzuhängen. Das vereinfacht den Austausch und hält den häufig genutzten Datensatz frei von sachfremden Feldern.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-3) UUIDv7
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Fast alles, was Nutzende sehen, wird über UUIDs adressiert, und das ist Absicht. Ich möchte keine fortlaufenden IDs offenlegen, mit denen sich Zeilenzahlen ableiten oder Ressourcen wie im Jahr 2009 aufzählen lassen. Zeitlich geordnete UUIDs sind für Indizes außerdem deutlich verträglicher als völlig zufällige UUIDv4-Werte, was bei realen Schreiblasten hilfreich ist. Kurz gesagt: weltweit eindeutig, schwer zu erraten und besser für die Datenlokalität. Einfügungen bleiben geordneter, B-Bäume müssen weniger umsortieren und die Datenbank hat weniger vermeidbare Aufräumarbeit.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-4) Diagramm (Anfrage- und Datenpfad)
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Ein äußerst glamouröses Unternehmensarchitekturdiagramm, hier als Text, weil ich mich noch nicht dazu aufgerafft habe, Kästchen zu zeichnen:
              <br />
              <br />
              Browser-Anfrage -&gt; Axum-Router -&gt; Middleware-Kette (Protokollierung, Authentifizierungs-/Sitzungsprüfung, Ratenbegrenzung, Prüfung der Anfragegröße) -&gt; Handler -&gt; asynchrone Diesel-Abfrage oder Cache-Abfrage im Arbeitsspeicher -&gt; PostgreSQL über UNIX-Socket -&gt; DTO-Antwort -&gt; komprimierte HTTPS-Antwort an den Browser.
              <br />
              <br />
              Statische Assets nehmen einen noch kürzeren Weg. Die gebaute SolidJS-Anwendung ist direkt in das Rust-Binary eingebettet und wird per Inhaltsaushandlung mit zstd oder gzip ausgeliefert, sofern der Browser das unterstützt. Kein dauerhaft laufender Node-Prozess in Produktion, kein separater Host für statische Dateien und kein zusätzlicher Umweg zwischen „Anfrage angekommen“ und „Bytes ausgeliefert“.
            </p>
          </section>

          {/* 2) Backend */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              2) Backend (Rust)
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              Das Backend ist ein Rust-Dienst auf Basis von Axum, Tokio, Diesel und PostgreSQL. Als globaler Allokator kommt mimalloc zum Einsatz: Wer selbst auf einem Rechner mit echten Prozessorkernen hostet, kann sich auch ein wenig mit dem Verhalten des Allokators beschäftigen. Der Dienst beendet TLS direkt mit rustls, liefert die eingebettete SPA aus, stellt JSON-APIs für Authentifizierung, Blog, Fotografie und Internationalisierung bereit und sendet außerdem Host-Statistiken per WebSockets an das Server-Dashboard. Der Router umfasst Anfragekomprimierung, vorerst großzügig konfiguriertes CORS, recht großzügige Ratenbegrenzung und eine Swagger-UI, die in Produktion durch Authentifizierung geschützt ist.
              <br />
              <br />
              Auf Anwendungsebene setzt das Design auf einen einzelnen Prozess mit gemeinsamem Serverzustandsobjekt, Sitzungsverwaltung im Arbeitsspeicher, Synchronisierung des Caches beim Start und geplanten Wartungsaufgaben. Die Authentifizierung nutzt sichere HTTP-Only-Cookies, E-Mail-Verifizierung, Tokens zum Zurücksetzen des Passworts und explizite Rollenprüfungen für Superuser-Aktionen. Interessanter ist das Latenzprofil: Der Datenverkehr zur Datenbank bleibt auf einem UNIX-Socket, der Verbindungspool richtet seine Größe nach der Zahl physischer Kerne aus, Listenabfragen des Blogs kommen größtenteils aus dem Cache und Antworten werden in Stapeln angereichert, statt in unzählige winzige Abfragen zu zerfallen. So bleibt der Stack auf die Weise schnell, die mir wichtig ist: weniger Umwege, weniger Kopien, weniger Roundtrips und weniger Warten darauf, dass eine Abstraktion sich selbst auf die Schulter klopft.
            </p>
          </section>

          {/* 3) Frontend */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              3) Frontend
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              Das Frontend ist eine clientseitige SolidJS-SPA, erstellt mit Vite und TypeScript. Routen werden verzögert geladen, der Zustand bleibt überschaubar, und die Anwendung wird zu statischen Assets kompiliert, die bei der Bereitstellung in das Rust-Binary eingebunden werden. Dadurch muss in Produktion kein JavaScript-Serverprozess betreut werden; die erste Auslieferung besteht einfach darin, dass der Rust-Server statische Dateien möglichst effizient bereitstellt.
              <br />
              <br />
              Die Blog-Oberfläche verbindet Pragmatismus mit meiner Weigerung, Spielzeug-Editoren zu verwenden. Markdown wird mit dem Toast UI Editor geschrieben; eingefügte oder hochgeladene Bilder laufen direkt über die Upload-API für Fotos, damit das Verfassen von Beiträgen nicht zur Qual wird. Die Suche unterstützt Titelabfragen und Tags, Seiten lassen sich über Query-Parameter aufrufen, der Authentifizierungsstatus wird clientseitig erfasst, aber serverseitig durchgesetzt. Außerdem wertet die Anwendung Antwort-Header aus, um Build-Informationen des Servers anzuzeigen. Das Styling basiert auf Tailwind und einem gemeinsamen Seitensystem. Solid bleibt zur Laufzeit schlank und verändert das DOM sparsam. Genau das wünsche ich mir von einer UI-Schicht, deren Hauptaufgabe darin besteht, nicht im Weg zu stehen.
            </p>
          </section>

          {/* 4) Network & HTTPS */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              4) HTTPS, Routing und Schutzmaßnahmen
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              HTTPS wird direkt vom Rust-Server mit rustls abgewickelt. Reine HTTP-Anfragen werden auf HTTPS umgeleitet, Cookies als Secure und HTTP-Only markiert und die Cookie-Domain in Produktion auf die Website-Domain beschränkt. Statische Assets werden bei Unterstützung mit zstd oder gzip ausgeliefert. Das SPA-Fallback-Routing sorgt dafür, dass tiefe Links ohne separaten Reverse-Proxy funktionieren.
              <br />
              <br />
              Die Schutzmaßnahmen sind nicht besonders exotisch, aber vorhanden: Größenbeschränkungen für Upload-Anfragen, Middleware-Protokollierung, Authentifizierungsprüfungen für geschützte Routen, Superuser-Prüfungen an sensiblen Endpunkten, Ratenbegrenzung gegen Missbrauch und API-Schlüssel für Client-Anfragen. Noch wichtiger: Die Bereitstellung ist einfach genug, um sie zu verstehen. Ein Binary, eine Datenbank, ein Host, TLS in der Anwendung und kaum Raum für mysteriöse Latenz oder Konfigurationsabweichungen. Diese Architektur ist nicht modisch, aber schnell, beobachtbar und konsequent ressourcenschonend.
            </p>
          </section>

          <RepositorySourceMap />
        </section>
      </section>
    </main>
  );
}
