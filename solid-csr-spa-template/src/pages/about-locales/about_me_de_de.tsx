import { pageStyles } from "../../styles/pageStyles";
import badgeAWS232F3E from "../../assets/badges/AWS-232F3E.svg";
import badgeAmazonLinux232F3E from "../../assets/badges/Amazon_Linux-232F3E.svg";
import badgeArchLinux1793D1 from "../../assets/badges/Arch_Linux-1793D1.svg";
import badgeAxum000000 from "../../assets/badges/Axum-000000.svg";
import badgeBACnet004B87 from "../../assets/badges/BACnet-004B87.svg";
import badgeCA8B9CC from "../../assets/badges/C-A8B9CC.svg";
import badgeDjango092E20 from "../../assets/badges/Django-092E20.svg";
import badgeDocker2496ED from "../../assets/badges/Docker-2496ED.svg";
import badgeGoogleCloud4285F4 from "../../assets/badges/Google_Cloud-4285F4.svg";
import badgeJava007396 from "../../assets/badges/Java-007396.svg";
import badgeKubernetes326CE5 from "../../assets/badges/Kubernetes-326CE5.svg";
import badgeLinuxFCC624 from "../../assets/badges/Linux-FCC624.svg";
import badgeModbusFFCC00 from "../../assets/badges/Modbus-FFCC00.svg";
import badgeMySQL4479A1 from "../../assets/badges/MySQL-4479A1.svg";
import badgeNGINX009639 from "../../assets/badges/NGINX-009639.svg";
import badgePostgreSQL316192 from "../../assets/badges/PostgreSQL-316192.svg";
import badgeProtocolBuffers3367D6 from "../../assets/badges/Protocol_Buffers-3367D6.svg";
import badgePython3776AB from "../../assets/badges/Python-3776AB.svg";
import badgeRust000000 from "../../assets/badges/Rust-000000.svg";
import badgeSpringBoot6DB33F from "../../assets/badges/Spring_Boot-6DB33F.svg";
import badgeTypeScript3178C6 from "../../assets/badges/TypeScript-3178C6.svg";
import badgeUbuntuE95420 from "../../assets/badges/Ubuntu-E95420.svg";
import badgeWebSockets010101 from "../../assets/badges/WebSockets-010101.svg";
import badgeWindows0078D6 from "../../assets/badges/Windows-0078D6.svg";
import badgemacOS000000 from "../../assets/badges/macOS-000000.svg";
import { For } from "solid-js";
import { createMediaQuery } from "../../utils/mediaQuery";

const BADGES = [
  [badgeRust000000, "Rust"],
  [badgeCA8B9CC, "C"],
  [badgeTypeScript3178C6, "TypeScript"],
  [badgePython3776AB, "Python"],
  [badgeJava007396, "Java"],
  [badgeAxum000000, "Axum"],
  [badgeSpringBoot6DB33F, "Spring Boot"],
  [badgeDjango092E20, "Django"],
  [badgePostgreSQL316192, "PostgreSQL"],
  [badgeMySQL4479A1, "MySQL"],
  [badgeAWS232F3E, "AWS"],
  [badgeGoogleCloud4285F4, "Google Cloud"],
  [badgeDocker2496ED, "Docker"],
  [badgeKubernetes326CE5, "Kubernetes"],
  [badgeNGINX009639, "NGINX"],
  [badgeLinuxFCC624, "Linux"],
  [badgeArchLinux1793D1, "Arch Linux"],
  [badgeUbuntuE95420, "Ubuntu"],
  [badgeAmazonLinux232F3E, "Amazon Linux"],
  [badgeWindows0078D6, "Windows"],
  [badgemacOS000000, "macOS"],
  [badgeProtocolBuffers3367D6, "Protocol Buffers"],
  [badgeBACnet004B87, "BACnet"],
  [badgeModbusFFCC00, "Modbus"],
  [badgeWebSockets010101, "WebSockets"],
] as const;

export default function About() {
  const isMobile = createMediaQuery("(max-width: 767px)");
  const deferredLoading = (): "lazy" | "eager" =>
    isMobile() ? "lazy" : "eager";
  const deferredDecoding = (): "async" | "auto" =>
    isMobile() ? "async" : "auto";

  return (
    <main lang="de-DE" class={`${pageStyles.page} about-page`}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">Über mich</h1>
          <p class="text-xs text-ink-muted">
            Zuletzt aktualisiert: September 2026
          </p>
        </div>

        <section class="mb-12">
          <h2 class="text-xl font-bold mb-4 border-b border-line pb-2">
            1. Einleitung
          </h2>

          <div class="mb-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div>
              <h3 class="text-lg font-semibold">
                Young Hyun Chi / 지영현 / 池營賢 / 池营贤
              </h3>

              <p class="text-ink-muted">
                Backend- und Infrastrukturentwickler · Rust / Plattformzuverlässigkeit / Sicherheit
              </p>
            </div>

            <figure class="flex flex-col items-center sm:items-end text-sm text-ink-muted">
              <a
                href="https://cyhdev-img.s3.us-west-1.amazonaws.com/images/05da63f0-0a8e-4d96-807f-280ded45a6d5.avif"
                target="_blank"
                rel="noopener noreferrer"
              >
                <img
                  src="https://cyhdev-img.s3.us-west-1.amazonaws.com/thumbnails/05da63f0-0a8e-4d96-807f-280ded45a6d5.avif"
                  alt="Porträt"
                  width="112"
                  height="112"
                  class="h-28 w-28 rounded-full object-cover shadow-sm ring-2 ring-line"
                  loading={deferredLoading()}
                  decoding={deferredDecoding()}
                />
              </a>
              <figcaption class="mt-1 text-xs text-ink-muted text-center sm:text-right max-w-44" />
            </figure>
          </div>
          <div>
            <h3 class="font-bold mb-2">Kontakt</h3>
            <ul class="list-none space-y-2 text-sm">
              <li>
                <span class="w-20 inline-block font-medium text-ink-muted">
                  E-Mail:
                </span>
                <a href="mailto:younghyun1@gmail.com" class={pageStyles.link}>
                  younghyun1@gmail.com
                </a>
              </li>
              <li>
                <span class="w-20 inline-block font-medium text-ink-muted">
                  GitHub:
                </span>
                <a href="https://github.com/younghyun1" class={pageStyles.link}>
                  github.com/younghyun1
                </a>
              </li>
              <li>
                <span class="w-20 inline-block font-medium text-ink-muted">
                  LinkedIn:
                </span>
                <a
                  href="https://www.linkedin.com/in/young-hyun-chi-553431376/"
                  class={pageStyles.link}
                >
                  linkedin.com/in/young-hyun-chi-553431376
                </a>
              </li>
            </ul>
          </div>
          <br />
          <div class="about-badges flex flex-wrap gap-2 mb-6">
            <For each={BADGES}>
              {([source, label]) => (
                <img
                  src={source}
                  alt={label}
                  height="28"
                  class="h-7 w-auto"
                  loading={deferredLoading()}
                  decoding={deferredDecoding()}
                />
              )}
            </For>
          </div>
          <p class="mb-4 leading-relaxed">
            Ich bin Backend-Entwickler in Colorado und auf Rust-Dienste, PostgreSQL-Datenbanken und Cloud-Infrastruktur spezialisiert. Bei Soundpatrol arbeite ich an Cybersicherheit, DevOps, Webdiensten und Entwicklerwerkzeugen und nutze dabei Rust, Python, PostgreSQL, Kubernetes und GCP.
          </p>
          <p class="mb-4 leading-relaxed">
            Zu früheren Projekten zählen ein digitaler Zwilling mit Tausenden Umweltsensoren für Samsung C&amp;T, betrieben auf AWS; eine Analyseplattform für YouTube-Kanäle mit Transkripten, Kommentaren und Sprachmodellen; sowie Backends und Datenpipelines für Anwendungen von Hyundai, Kia und Genesis und die Fan-App einer K-Pop-Gruppe.
          </p>
          <p class="mb-4 leading-relaxed">
            Ich möchte Dienste entwickeln, die moderne Hardware und Programmiersprachen sinnvoll nutzen. Ich beschäftige mich gern mit den Details von Nebenläufigkeit, Datenbankabfragen, Speicherverbrauch und Bereitstellung, statt Cloud-Ressourcen als unbegrenzten Ersatz für gute Entwicklungsarbeit zu betrachten. Außerdem betreibe ich gern eigene Dienste, entwickle Werkzeuge in Rust und helfe Kolleginnen und Kollegen beim Einstieg in Rust.
          </p>
          <p class="mb-4 leading-relaxed">
            Ich bin stolz darauf, in verschiedenen Bereichen und mit Kolleginnen und Kollegen aus unterschiedlichen Kulturen und Lebenswelten gearbeitet zu haben. Von 2016 bis 2017 diente ich 21 Monate nahe der nordkoreanischen Grenze bei der 2. US-Infanteriedivision und arbeitete mit amerikanischem und koreanischem Militärpersonal zusammen. Während des Studiums arbeitete ich im Journalismus, in der Fotografie und Übersetzung sowie nachts in Lager und Logistik. Seit 2023 konzentriere ich mich beruflich auf Backend-Entwicklung, Datenpipelines und die Infrastruktur, die sie am Laufen hält.
          </p>
        </section>

        {/* 2) Professional Career */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            2. Entwicklerlaufbahn
          </h2>

          <div class="space-y-8">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Soundpatrol</h3>
                <span class="text-sm font-mono text-ink-muted">Vereinigte Staaten · Remote | Mai 2026 - heute</span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">Rust-Entwickler</p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>Entwicklung von Webdiensten und internen Werkzeugen mit Rust und Python.</li>
                <li>Wartung der Cloud-Infrastruktur, Kubernetes-Bereitstellungen und CI/CD-Pipelines.</li>
                <li>Arbeit an Monitoring, Zugriffskontrollen, Zugangsdatenverwaltung und weiteren Cybersicherheitsmaßnahmen.</li>
              </ul>
            </div>

            {/* GenesisNest */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">GenesisNest</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Seongnam, Südkorea | Jan. 2025 - Juli 2025
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Softwareentwickler
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Entwickelte Infrastrukturverwaltungswerkzeuge für interne Dienste und die offizielle App einer K-Pop-Gruppe. Dabei machte ich mich mit dem Spring-Boot-Ökosystem vertraut.
                </li>
                <li>
                  Arbeitete als Auftragnehmer an den Backends der offiziellen Hyundai/Kia/Genesis-Apps und integrierte Änderungen in eine große Java-Codebasis.
                </li>
                <li>
                  Implementierte neue APIs und Fehlerbehebungen und koordinierte sich mit Dienstleistern für Services, die zig Millionen Menschen in Asien und Europa nutzen.
                </li>
                <li>
                  Richtete Datenpipelines für die Internationalisierung ein und schrieb mehrere Rust- und Python-Skripte, um übersetzte Inhalte automatisiert in die Hyundai-Datenbank zu integrieren. Dabei koordinierte ich mich mit Datenbankadministratoren, die sehr große Datensätze aus den Fabriksystemen des Unternehmens betreuten.
                </li>
              </ul>
            </div>

            {/* pampam */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">pampam Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Seoul, Südkorea | Nov. 2024 - Dez. 2024
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Softwareentwickler auf Vertragsbasis
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Entwarf das PostgreSQL-Schema und Rust-Backend einer KI-gestützten YouTube-Analyseplattform und leitete die Bereitstellung auf AWS und GCP.
                </li>
                <li>
                  Leitete die umfangreiche Datenerfassung über die YouTube-API mit zig Millionen Kommentaren und Nutzern.
                </li>
                <li>
                  Nutzte große Sprachmodelle und lokal betriebene Open-Source-Modelle, um Erkenntnisse und Zusammenfassungen zu erstellen.
                </li>
              </ul>
            </div>

            {/* EAN Technology */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">EAN Technology Co. Ltd</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Seoul, Südkorea | Aug. 2023 - Aug. 2024
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Leitender Backend-Softwareentwickler
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>Digital-Twin-Projekt für Samsung C&T:</b> Ersetzte eine fehleranfällige Microservice-Architektur durch einen hochoptimierten Axum-Monolithen in Rust. Senkte die monatlichen Cloud-Kosten von etwa 5.000 auf 150 US-Dollar.
                </li>
                <li>
                  Trug etwa 30.000 Codezeilen (rund 75 % des Codes) bei und implementierte 80 Endpunkte mit komplexer Fachlogik.
                </li>
                <li>
                  Erreichte während neun Monaten im Produktivbetrieb keine Laufzeitabbrüche und eine Abdeckung der Fehlerbehandlung von rund 90 %.
                </li>
                <li>
                  Integrierte Tausende physische Echtzeitsensoren (BACnet, Modbus) und lokale Server am Hauptsitz von Samsung C&amp;T.
                </li>
                <li>
                  Optimierte PostgreSQL-Abfragen und -Schema. Auf begrenzter Hardware (2 Kerne, 4 GB RAM) sank die P99-Latenz bei zuvor nicht optimierten Abfragen von mehreren Sekunden auf zweistellige Millisekundenwerte.
                </li>
                <li>
                  Implementierte CI/CD mit GitHub Runners, AWS CodeDeploy und Docker. Mit MUSL, statischem Linken und mehrstufigen Builds mit „scratch“-Images entstanden etwa 20 MB große Docker-Images mit höherer Sicherheit und kürzeren Kaltstarts.
                </li>
                <li>
                  Begleitete Kolleginnen und Kollegen beim Erlernen von Rust und ermöglichte ihnen den Umstieg von Node.js/Java innerhalb von zwei Monaten.
                </li>
              </ul>
            </div>

            {/* Artifyc */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Artifyc Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Austin, TX (Remote) | Aug. 2022 - März 2023
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Softwareentwickler im Praktikum
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Implementierte einen Python-Discord-Bot, der über die Reddit-API Anfragen für Kunstaufträge sammelte und veröffentlichte.
                </li>
                <li>
                  Entwickelte serverlose Backend-Logik auf AWS Lambda für weltweite Währungsumrechnung und Aufschlagsberechnung.
                </li>
                <li>
                  Unterstützte Anpassungen am React-Frontend und die Integration von Assets.
                </li>
              </ul>
            </div>
          </div>
        </section>


        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">Ausgewählte Projekte</h2>
          <ul class="list-disc list-inside text-sm space-y-3 text-ink">
            <li><a href="https://github.com/younghyun1/sillok" class={pageStyles.link}>Sillok</a>: ein Rust-CLI-Werkzeug für strukturierte Arbeitsprotokolle mit unveränderlichem Verlauf, indizierten Turso-/SQLite-Projektionen, Korrekturen und Git-gestützter Synchronisierung zwischen Geräten.</li>
            <li><a href="https://github.com/younghyun1/eu5-location-filter" class={pageStyles.link}>EU5 Location DB</a>: eine Desktop- und WebAssembly-Anwendung mit Rust und Slint, eingebetteten komprimierten Kartendaten, vorberechneten Indizes, durchsuchbaren Filtern und anpassbaren Tabellen.</li>
            <li><a href="https://github.com/younghyun1/cyhdev" class={pageStyles.link}>cyhdev.com</a>: diese Website mit Rust/Axum, PostgreSQL und SolidJS, einschließlich Veröffentlichungen, Fotografie, Diskussionen, Chat und Browser-Demos.</li>
            <li><a href="https://github.com/younghyun1/oohid" class={pageStyles.link}>oohid</a>: ein UUID-Generator in Rust mit konfigurierbarem Format, Dateiausgabe und optionaler Prüfung auf Duplikate.</li>
          </ul>
        </section>

        {/* 3) Non-IT Career */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            3. Berufserfahrung außerhalb der IT
          </h2>

          <div class="space-y-6">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">
                  Korea Institute of Maritime Science and Technology
                </h3>
                <span class="text-sm font-mono text-ink-muted">
                  Seoul, Südkorea | Juni-Juli 2023, Aug. 2024
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Übersetzer und Dolmetscher
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Dolmetschte bei hochrangigen Konferenzen mit dem norwegischen Botschafter und dem Vorsitzenden der Dongwon Group aus dem Englischen, Koreanischen und Norwegischen.
                </li>
                <li>
                  War in vertrauliche staatliche und geschäftliche Vorgänge sowie Gespräche zu Forschung und Entwicklung im maritimen Bereich eingebunden.
                </li>
                <li>
                  Bereitete Mitarbeitende und den Direktor des KIMST auf Veranstaltungen des US-Energieministeriums vor.
                </li>
              </ul>
            </div>

            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">
                  <a
                    href="https://en.wikipedia.org/wiki/United_States_Army"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    US Army
                  </a>{" "}
                  (
                  <a
                    href="https://en.wikipedia.org/wiki/2nd_Infantry_Division_(United_States)"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    2nd Infantry Division
                  </a>
                  ) /{" "}
                  <a
                    href="https://en.wikipedia.org/wiki/Republic_of_Korea_Army"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    ROK Army
                  </a>
                </h3>
                <span class="text-sm font-mono text-ink-muted">
                  <a
                    href="https://en.wikipedia.org/wiki/Dongducheon"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    Dongducheon
                  </a>
                  , Südkorea | März 2016 - Dez. 2017
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                <a
                  href="https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army"
                  class={pageStyles.link}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  KATUSA
                </a>{" "}
                | Ausbilder | Sergeant
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Diente als Verbindungssoldat und Ausbilder. Schulung und administrative Aufnahme von rund 6.000 amerikanischen und koreanischen Angehörigen.
                </li>
                <li>
                  Unterrichtete Fitness, Kultur, Geschichte sowie Sicherheits- und Präventionskurse.
                </li>
                <li>
                  Hielt die Einsatzbereitschaft während der{" "}
                  <a
                    href="https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    Krise von 2017
                  </a>
                  ; führte ein Team von dreizehn Personen.
                </li>
              </ul>
            </div>

            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Coupang Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Südkorea | 2020 - 2022
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Lagerarbeiter
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Schwere körperliche Arbeit in Logistiklagern während der Nachtschicht.
                </li>
              </ul>
            </div>
          </div>
        </section>

        {/* 4) Academics */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            4. Studium
          </h2>

          <div class="mb-6">
            <div class="flex flex-wrap justify-between items-baseline mb-2">
              <h3 class="text-lg font-bold">Sungkyunkwan University</h3>
              <span class="text-sm font-mono text-ink-muted">
                Suwon, Südkorea | März 2015 - Aug. 2023
              </span>
            </div>
            <p class="text-sm font-medium mb-2 text-ink-muted">
              Bachelor of Engineering in Software Engineering
            </p>
            <div class="space-y-2 text-sm text-ink">
              <p>
                Absolvierte ein umfassendes Studium aus Informatik, Web- und Embedded-Softwareentwicklung sowie Elektrotechnik. Schwerpunkte waren Logik, hardwarenahe Programmiersprachen, Betriebssysteme und Netzwerktechnik sowie Methoden der Softwareentwicklung.
              </p>
              <p>
                Die Sungkyunkwan University ist die älteste Universität Koreas. Sie wurde 1398 als konfuzianische Verwaltungsakademie des Königreichs Joseon gegründet. Seit der Befreiung zählt sie zu den führenden akademischen und historischen Einrichtungen Koreas und wird seit meiner Studienzeit durchgehend unter den 100 besten Universitäten der Welt geführt.
              </p>
            </div>
          </div>

          <div>
            <h4 class="font-bold mb-2 text-md">Veröffentlichungen</h4>
            <ul class="list-disc list-inside text-sm space-y-2 text-ink">
              <li>
                <b>
                  Emotionsanalyse sozialer Mediengemeinschaften mit einem NLP-Modell für die koreanische Sprache
                </b>{" "}
                (IMCOM 2023, IEEE Xplore).
                <br />
                <span class="ml-5 block text-xs text-ink-muted mt-1">
                  Nutzte KoBERT für eine Sentimentanalyse sozialer Mediendaten zu Beginn der COVID-Pandemie. Mit dem Bronze-Preis für Abschlussprojekte ausgezeichnet.
                </span>
              </li>
            </ul>
          </div>
        </section>

        {/* 5) Volunteer work and interests */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            5. Ehrenamt &amp; Interessen
          </h2>

          <div class="grid md:grid-cols-2 gap-8">
            <div>
              <h3 class="font-bold mb-3 text-lg">Persönliche Projekte</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>
                  <b>oohid:</b> Ein CLI-Generator für UUIDv4 in Rust, etwa dreimal schneller als libuuid. Mit Duplikatprüfung und Formatierung für Python/JSON.
                </li>
                <li>
                  <b>impulsr:</b> Ein Werkzeug zur Transkription von YouTube-Videos und Sammlung von Kommentaren für Marketing-Zusammenfassungen.
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-3 text-lg">Ehrenamt</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>Gab Englischunterricht in einem Gemeindezentrum.</li>
                <li>Engagierte mich ehrenamtlich in einem Secondhandladen, der von Menschen mit Behinderungen betrieben wird.</li>
              </ul>
            </div>
          </div>
        </section>

        {/* 6) Hobbies */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            6. Hobbys
          </h2>

          <div class="space-y-4">
            <div>
              <h3 class="font-bold inline mr-2">Fotografie</h3>
              <span class="text-sm text-ink-muted">
                Amateurfotograf für Landschafts- und Porträtfotografie seit 2010. Rund 30.000 archivierte Fotos.
              </span>
              <div class="mt-2">
                <a
                  href="/photographs"
                  class={`inline-flex items-center ${pageStyles.link} font-medium`}
                >
                  Fotografie-Portfolio ansehen →
                </a>
              </div>
            </div>

            <div>
              <h3 class="font-bold inline mr-2">Journalismus</h3>
              <p class="text-sm text-ink mt-1">
                Ehemaliger studentischer Journalist bei der Sungkyun Times.
              </p>
            </div>
          </div>
        </section>

        {/* 7) Qualifications and Other Skills */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            7. Qualifikationen und weitere Fähigkeiten
          </h2>

          <div class="grid md:grid-cols-2 gap-8 mb-8">
            <div>
              <h3 class="font-bold mb-2">Qualifikationen & Auszeichnungen</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>TOEFL:</b> 117/120 (CEFR C2)
                </li>
                <li>
                  <b>IELTS:</b> 8.5/9.0 (CEFR C2)
                </li>
                <li>
                  <b>7. Platz:</b> Capstone-Design- und Ideen-Hackathon 2021
                </li>
                <li>
                  <b>3. Platz:</b> Fintech-Hackathon 2020 (SNUST)
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-2">Sprachen</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>Koreanisch (Muttersprache)</li>
                <li>Englisch (C2)</li>
                <li>Grundkenntnisse in Französisch, Deutsch, Spanisch und Mandarin</li>
              </ul>
            </div>
          </div>
        </section>
      </section>
    </main>
  );
}
