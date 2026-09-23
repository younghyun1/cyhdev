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
    <main lang="es-ES" class={`${pageStyles.page} about-page`}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">Sobre mí</h1>
          <p class="text-xs text-ink-muted">
            Última actualización: septiembre de 2026
          </p>
        </div>

        <section class="mb-12">
          <h2 class="text-xl font-bold mb-4 border-b border-line pb-2">
            1. Introducción
          </h2>

          <div class="mb-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div>
              <h3 class="text-lg font-semibold">
                Young Hyun Chi / 지영현 / 池營賢 / 池营贤
              </h3>

              <p class="text-ink-muted">
                Ingeniero de backend e infraestructura · Rust / Fiabilidad de plataformas / Seguridad
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
                  alt="Retrato"
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
            <h3 class="font-bold mb-2">Contacto</h3>
            <ul class="list-none space-y-2 text-sm">
              <li>
                <span class="w-20 inline-block font-medium text-ink-muted">
                  Correo:
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
            Soy ingeniero de backend y vivo en Colorado. Me especializo en servicios Rust, bases de datos PostgreSQL e infraestructura en la nube. En Soundpatrol trabajo en ciberseguridad, DevOps, servicios web y herramientas para desarrolladores con Rust, Python, PostgreSQL, Kubernetes y GCP.
          </p>
          <p class="mb-4 leading-relaxed">
            Entre mis proyectos anteriores están un gemelo digital para Samsung C&amp;T que integra miles de sensores ambientales y se ejecuta en AWS; una plataforma de análisis de canales de YouTube basada en transcripciones, comentarios y modelos de lenguaje; y sistemas backend y canalizaciones de datos para las aplicaciones de Hyundai, Kia y Genesis, además de la aplicación para fans de un grupo de K-pop.
          </p>
          <p class="mb-4 leading-relaxed">
            Creo en crear servicios que aprovechen bien el hardware moderno y los lenguajes de programación. Disfruto resolver los detalles de la concurrencia, las consultas a bases de datos, el uso de memoria y los despliegues, en vez de tratar los recursos de la nube como un sustituto ilimitado de la ingeniería. También disfruto alojar mis propios servicios, crear herramientas en Rust y ayudar a mis compañeros a aprender Rust.
          </p>
          <p class="mb-4 leading-relaxed">
            Me enorgullece haber trabajado en distintos campos y con compañeros de culturas y orígenes diversos. Entre 2016 y 2017, presté servicio durante 21 meses cerca de la frontera con Corea del Norte en la 2.ª División de Infantería del Ejército de Estados Unidos, junto a personal militar estadounidense y coreano. Durante la universidad trabajé en periodismo, fotografía y traducción, además de turnos nocturnos en almacenes y logística. Desde 2023, mi carrera se centra en la ingeniería backend,
            canalizaciones de datos y la infraestructura que las mantiene en funcionamiento.
          </p>
        </section>

        {/* 2) Trayectoria profesional */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            2. Trayectoria como desarrollador
          </h2>

          <div class="space-y-8">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Soundpatrol</h3>
                <span class="text-sm font-mono text-ink-muted">Estados Unidos · Remoto | mayo de 2026 - actualidad</span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">Ingeniero Rust</p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>Desarrollo servicios web y herramientas internas en Rust y Python.</li>
                <li>Mantengo la infraestructura en la nube, los despliegues de Kubernetes y las canalizaciones CI/CD.</li>
                <li>Trabajo en monitorización, controles de acceso, gestión de credenciales y otras medidas de ciberseguridad.</li>
              </ul>
            </div>

            {/* GenesisNest */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">GenesisNest</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Seongnam, Corea del Sur | ene. 2025 - jul. 2025
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Ingeniero de software
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Desarrollé herramientas de gestión de infraestructura para servicios internos y la aplicación oficial de un grupo de K-pop. Me familiaricé con el ecosistema Spring Boot.
                </li>
                <li>
                  Trabajé por contrato en los sistemas backend de las aplicaciones oficiales de Hyundai, Kia y Genesis de Hyundai Motor Company, integrándome en una extensa base de código Java.
                </li>
                <li>
                  Implementé nuevas API y correcciones de errores, coordinándome con proveedores de servicios utilizados por decenas de millones de personas en Asia y Europa.
                </li>
                <li>
                  Establecí canalizaciones de datos para su trabajo de internacionalización y escribí varios programas en Rust y Python para automatizar la integración de contenido traducido en la base de datos de Hyundai. Me coordiné con administradores de bases de datos corporativos que gestionaban conjuntos de datos muy grandes vinculados a los sistemas de fábrica de la empresa.
                </li>
              </ul>
            </div>

            {/* pampam */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">pampam Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Seúl, Corea del Sur | nov. 2024 - dic. 2024
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Ingeniero de software por contrato
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Diseñé el esquema PostgreSQL y el backend Rust de una plataforma de análisis de YouTube basada en IA, y dirigí los despliegues en AWS y GCP.
                </li>
                <li>
                  Dirigí la recopilación de datos a gran escala mediante la API de YouTube (decenas de millones de comentarios y usuarios).
                </li>
                <li>
                  Apliqué modelos de lenguaje y modelos de código abierto ejecutados en el servidor para generar análisis y resúmenes.
                </li>
              </ul>
            </div>

            {/* EAN Technology */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">EAN Technology Co. Ltd</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Seúl, Corea del Sur | ago. 2023 - ago. 2024
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Ingeniero de software backend (líder)
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>Proyecto de gemelo digital de Samsung C&T:</b> Retiré una arquitectura de microservicios disfuncional y la sustituí por un monolito Axum (Rust) muy optimizado. Reduje la factura mensual de la nube de unos 5.000 a unos 150 dólares.
                </li>
                <li>
                  Aporté unas 30.000 líneas de código (cerca del 75 % del total) e implementé 80 endpoints con lógica de dominio compleja.
                </li>
                <li>
                  Logré cero interrupciones del servicio y una cobertura de gestión de errores de aproximadamente el 90 % durante nueve meses en producción.
                </li>
                <li>
                  Integré miles de sensores físicos en tiempo real (BACnet, Modbus) y servidores locales de la sede de Samsung C&amp;T.
                </li>
                <li>
                  Optimicé las consultas y el esquema de PostgreSQL: en hardware limitado (2 núcleos y 4 GB de RAM), reduje de varios segundos a decenas de milisegundos la latencia P99 de algunas consultas.
                </li>
                <li>
                  Implementé CI/CD con GitHub Runners, AWS CodeDeploy y Docker. Usé MUSL, enlazado estático y compilaciones de varias etapas con imágenes scratch para generar imágenes Docker de unos 20 MB, con mayor seguridad y arranques en frío más eficientes.
                </li>
                <li>
                  Ayudé a mis compañeros a aprender Rust y a migrar desde Node.js y Java en dos meses.
                </li>
              </ul>
            </div>

            {/* Artifyc */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Artifyc Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Austin, Texas (remoto) | ago. 2022 - mar. 2023
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Becario de ingeniería de software
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Implementé un bot de Discord en Python que usaba la API de Reddit para recopilar y publicar solicitudes de encargos artísticos.
                </li>
                <li>
                  Desarrollé lógica backend sin servidor en AWS Lambda para la conversión de divisas y el cálculo de recargos.
                </li>
                <li>
                  Colaboré en cambios del frontend React y en la integración de recursos.
                </li>
              </ul>
            </div>
          </div>
        </section>


        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">Proyectos destacados</h2>
          <ul class="list-disc list-inside text-sm space-y-3 text-ink">
            <li><a href="https://github.com/younghyun1/sillok" class={pageStyles.link}>Sillok</a>: una herramienta Rust de línea de comandos para registros de trabajo estructurados, con historial de solo anexado, proyecciones indexadas de Turso/SQLite, correcciones y sincronización entre equipos mediante Git.</li>
            <li><a href="https://github.com/younghyun1/eu5-location-filter" class={pageStyles.link}>EU5 Location DB</a>: una aplicación de escritorio y WebAssembly creada con Rust y Slint, con datos cartográficos comprimidos integrados, índices precalculados, filtros y tablas redimensionables.</li>
            <li><a href="https://github.com/younghyun1/cyhdev" class={pageStyles.link}>cyhdev.com</a>: este sitio creado con Rust/Axum, PostgreSQL y SolidJS, que incluye publicaciones, fotografía, debates, chat y demostraciones en el navegador.</li>
            <li><a href="https://github.com/younghyun1/oohid" class={pageStyles.link}>oohid</a>: un generador de UUID escrito en Rust con formato configurable, salida a archivo y comprobación opcional de duplicados.</li>
          </ul>
        </section>

        {/* 3) Trayectoria fuera de TI */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            3. Trayectoria fuera de TI
          </h2>

          <div class="space-y-6">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">
                  Instituto Coreano de Ciencia y Tecnología Marítimas
                </h3>
                <span class="text-sm font-mono text-ink-muted">
                  Seúl, Corea del Sur | jun.-jul. 2023, ago. 2024
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Traductor e intérprete
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Interpreté entre inglés, coreano y noruego en conferencias de alto nivel con la participación del embajador de Noruega y del presidente de Dongwon Group.
                </li>
                <li>
                  Me confiaron detalles de negociaciones estatales y comerciales muy sensibles, así como conversaciones sobre investigación y desarrollo marítimos.
                </li>
                <li>
                  Preparé al personal y al director de KIMST para eventos del Departamento de Energía de Estados Unidos.
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
                    Ejército de Estados Unidos
                  </a>{" "}
                  (
                  <a
                    href="https://en.wikipedia.org/wiki/2nd_Infantry_Division_(United_States)"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    2.ª División de Infantería
                  </a>
                  ) /{" "}
                  <a
                    href="https://en.wikipedia.org/wiki/Republic_of_Korea_Army"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    Ejército de la República de Corea
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
                  , Corea del Sur | mar. 2016 - dic. 2017
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
                | Instructor | Sargento
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Serví como soldado de enlace e instructor. Capacité y procesé administrativamente a unos 6.000 militares estadounidenses y coreanos.
                </li>
                <li>
                  Impartí cursos de preparación física, cultura, historia y prevención de riesgos.
                </li>
                <li>
                  Mantuve la capacidad operativa durante la{" "}
                  <a
                    href="https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    crisis de 2017
                  </a>
                  ; dirigí un equipo de trece personas.
                </li>
              </ul>
            </div>

            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Coupang Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Corea del Sur | 2020 - 2022
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Operario de almacén
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Realicé trabajo físico intenso en almacenes logísticos durante turnos nocturnos.
                </li>
              </ul>
            </div>
          </div>
        </section>

        {/* 4) Formación académica */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            4. Formación académica
          </h2>

          <div class="mb-6">
            <div class="flex flex-wrap justify-between items-baseline mb-2">
              <h3 class="text-lg font-bold">Sungkyunkwan University</h3>
              <span class="text-sm font-mono text-ink-muted">
                Suwon, Corea del Sur | mar. 2015 - ago. 2023
              </span>
            </div>
            <p class="text-sm font-medium mb-2 text-ink-muted">
              Grado en Ingeniería de Software
            </p>
            <div class="space-y-2 text-sm text-ink">
              <p>
                Completé un programa integral que combinaba informática, ingeniería de software web e integrado e ingeniería electrónica. Me centré en lógica, lenguajes de bajo nivel, sistemas operativos e ingeniería de redes, además de las prácticas de ingeniería de software.
              </p>
              <p>
                La Universidad Sungkyunkwan es la más antigua de Corea. Se fundó en 1398 como academia estatal de funcionarios del reino de Joseon, según la tradición confuciana. Desde la liberación, ha sido una de las principales instituciones académicas e históricas del país y, desde que estudié allí, se ha mantenido entre las 100 mejores universidades del mundo.
              </p>
            </div>
          </div>

          <div>
            <h4 class="font-bold mb-2 text-md">Publicaciones</h4>
            <ul class="list-disc list-inside text-sm space-y-2 text-ink">
              <li>
                <b>
                  Análisis emocional de comunidades en redes sociales mediante un modelo de PLN para el idioma coreano
                </b>{" "}
                (IMCOM 2023, IEEE Xplore).
                <br />
                <span class="ml-5 block text-xs text-ink-muted mt-1">
                  Utilicé KoBERT para analizar el sentimiento en datos de redes sociales durante la primera etapa de la pandemia de COVID-19. Recibió el Premio de Bronce en Proyectos de Graduación.
                </span>
              </li>
            </ul>
          </div>
        </section>

        {/* 5) Voluntariado e intereses */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            5. Voluntariado e intereses
          </h2>

          <div class="grid md:grid-cols-2 gap-8">
            <div>
              <h3 class="font-bold mb-3 text-lg">Proyectos personales</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>
                  <b>oohid:</b> Un generador de UUIDv4 para línea de comandos escrito en Rust, unas tres veces más rápido que libuuid. Incluye detección de duplicados y formato para Python/JSON.
                </li>
                <li>
                  <b>impulsr:</b> Herramienta para transcribir vídeos de YouTube y recopilar comentarios destinados a resúmenes de marketing.
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-3 text-lg">Voluntariado</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>Enseñé inglés en un centro comunitario.</li>
                <li>Colaboré como voluntario en una tienda de segunda mano gestionada por personas con discapacidad.</li>
              </ul>
            </div>
          </div>
        </section>

        {/* 6) Aficiones */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            6. Aficiones
          </h2>

          <div class="space-y-4">
            <div>
              <h3 class="font-bold inline mr-2">Fotografía</h3>
              <span class="text-sm text-ink-muted">
                Fotógrafo aficionado de paisajes y retratos desde 2010. Tengo archivadas unas 30.000 fotografías.
              </span>
              <div class="mt-2">
                <a
                  href="/photographs"
                  class={`inline-flex items-center ${pageStyles.link} font-medium`}
                >
                  Ver portafolio fotográfico →
                </a>
              </div>
            </div>

            <div>
              <h3 class="font-bold inline mr-2">Periodismo</h3>
              <p class="text-sm text-ink mt-1">
                Fui periodista estudiantil del Sungkyun Times.
              </p>
            </div>
          </div>
        </section>

        {/* 7) Cualificaciones y otras habilidades */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            7. Cualificaciones y otras habilidades
          </h2>

          <div class="grid md:grid-cols-2 gap-8 mb-8">
            <div>
              <h3 class="font-bold mb-2">Cualificaciones y premios</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>TOEFL:</b> 117/120 (CEFR C2)
                </li>
                <li>
                  <b>IELTS:</b> 8.5/9.0 (CEFR C2)
                </li>
                <li>
                  <b>7.º puesto:</b> Hackathon de Diseño e Ideas Capstone de 2021
                </li>
                <li>
                  <b>3.er puesto:</b> Hackathon Fintech 2020 (SNUST)
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-2">Idiomas</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>Coreano (nativo)</li>
                <li>Inglés (C2)</li>
                <li>Francés, alemán, español y chino mandarín básicos</li>
              </ul>
            </div>
          </div>
        </section>
      </section>
    </main>
  );
}
