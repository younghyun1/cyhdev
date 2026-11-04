import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "noviembre de 2026",
  chrome: {
    overview: "Resumen",
    updated: "Última actualización",
    onThisPage: "En esta página",
    sectionNavigation: "Páginas Sobre mí",
    previous: "Anterior",
    next: "Siguiente",
    readingTime: "{minutes} min de lectura",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "Sobre mí",
      lede: "Ingeniero de backend e infraestructura; trabajo con Rust, PostgreSQL y plataformas en la nube. Desarrollo servicios que aprovechan bien el hardware sobre el que se ejecutan, junto con el despliegue, la monitorización y la seguridad que los mantienen en funcionamiento.",
      sections: [
        {
          id: "introduction",
          heading: "Introducción",
          blocks: [
            {
              kind: "paragraph",
              text: "Soy un ingeniero de software coreano y vivo en Colorado. En **Soundpatrol** trabajo como ingeniero Rust en ciberseguridad, DevOps, servicios web y herramientas para desarrolladores, principalmente con Rust, Python, PostgreSQL, Kubernetes y Google Cloud.",
            },
            {
              kind: "paragraph",
              text: "Antes dirigí el backend y la infraestructura de un gemelo digital de un edificio de Samsung C&T, contribuí a los backends de las aplicaciones de Hyundai, Kia y Genesis, y desarrollé la recopilación de datos de una plataforma de análisis de YouTube. Desde 2023 mi trabajo se centra en servicios backend, canalizaciones de datos y la infraestructura que los sostiene.",
            },
            {
              kind: "paragraph",
              text: "Creo en crear servicios que aprovechen bien el hardware moderno y los lenguajes de programación. En la práctica, eso significa un diseño cuidadoso de los esquemas, colas y cachés acotadas, una gestión explícita de errores y despliegues que se puedan inspeccionar y recuperar. Disfruto ayudando a mis compañeros a aprender Rust y quiero trabajar con personas que se enorgullecen de su oficio.",
            },
          ],
        },
        {
          id: "results",
          heading: "Resultados destacados",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97 %",
                  count: 97,
                  suffix: " %",
                  label: "Menos gasto mensual en la nube en el gemelo digital de Samsung C&T, de unos 5.000 a 150 dólares",
                },
                {
                  value: "80",
                  count: 80,
                  label: "Endpoints REST implementados para el gemelo digital, con lógica de dominio de ingeniería de la construcción",
                },
                {
                  value: "9 meses",
                  count: 9,
                  suffix: " meses",
                  label: "Operación en producción del gemelo digital sin paradas en tiempo de ejecución",
                },
                {
                  value: "+20 %",
                  count: 20,
                  prefix: "+",
                  suffix: " %",
                  label: "Más rendimiento en lecturas mixtas en el backend de este sitio gracias a [PGO y BOLT](/about-blog/performance)",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Contenedores de servicios enlazados estáticamente, generados con compilaciones de varias etapas",
                },
                {
                  value: "30.000",
                  label: "Líneas escritas para el backend del gemelo digital, cerca del 75 % de su código",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Herramientas",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Lenguajes",
                  items: ["Rust", "Python", "TypeScript", "SQL", "Java", "C", "C++"],
                },
                {
                  label: "Backend",
                  items: ["Axum", "Tokio", "Diesel", "tokio-postgres", "Spring Boot", "FastAPI", "WebSockets", "gRPC", "Protocol Buffers"],
                },
                {
                  label: "Datos",
                  items: ["PostgreSQL", "Valkey / Redis", "SQLite / Turso", "MySQL", "FAISS", "tantivy"],
                },
                {
                  label: "Infraestructura",
                  items: ["Kubernetes", "Helm", "Docker", "OpenTofu / Terraform", "GitHub Actions", "Google Cloud", "AWS", "Linux"],
                },
                {
                  label: "Fiabilidad y seguridad",
                  items: ["OpenTelemetry", "IAM", "TLS", "Network policy", "Credential lifecycle", "Incident response"],
                },
                {
                  label: "Interfaces y dispositivos",
                  items: ["SolidJS", "Leptos", "Slint", "WebAssembly", "WebRTC", "BACnet", "Modbus"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "Más sobre mí",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Experiencia",
                  href: "/about/experience",
                  meta: "2016 - actualidad",
                  text: "Mis puestos, desde Soundpatrol hasta mis primeras prácticas, con los proyectos y resultados de cada uno.",
                },
                {
                  title: "Proyectos",
                  href: "/about/projects",
                  meta: "Rust · código abierto",
                  text: "Sillok, EU5 Location DB, este sitio web y otros trabajos independientes y de código abierto.",
                },
                {
                  title: "Trayectoria",
                  href: "/about/background",
                  meta: "Formación · investigación",
                  text: "Formación, investigación publicada, premios, idiomas y actividades fuera del software.",
                },
                {
                  title: "Cómo funciona este sitio",
                  href: "/about-blog",
                  meta: "Tecnologías",
                  text: "El servidor, el backend, el frontend, los subsistemas y las mediciones de rendimiento de cyhdev.com.",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "Contacto",
          blocks: [
            {
              kind: "list",
              items: [
                "**Correo electrónico**: [younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**: [github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**: [linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "El correo electrónico es la forma más fiable de contactarme. Si quieres unirte al servidor de Minecraft, indícalo también ahí.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "Sobre mí",
      title: "Experiencia",
      lede: "Trabajo de backend, infraestructura y seguridad en una startup estadounidense, para grandes empresas coreanas y en unas primeras prácticas, seguido de la trayectoria anterior al software.",
      sections: [
        {
          id: "software",
          heading: "Ingeniería de software",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Ingeniero Rust",
                  period: "mayo de 2026 - actualidad",
                  location: "Estados Unidos · Remoto",
                  summary: "Ciberseguridad, DevOps, desarrollo de servicios web y herramientas para desarrolladores en una empresa de detección de música y audio.",
                  highlights: [
                    "Mantengo la infraestructura de Kubernetes en Google Cloud, la infraestructura como código y flujos de despliegue revisados que usan credenciales de corta duración limitadas a cada entorno.",
                    "Planifico y ejecuto migraciones de almacenes de datos en producción, verificando el estado migrado y ensayando la recuperación antes de cada conmutación.",
                    "Consolido métricas, registros y trazas en OpenTelemetry, y desarrollo herramientas en Rust para el estado de los servicios, el diagnóstico y la visibilidad de costes.",
                    "Trabajo en la preparación en materia de seguridad: administración de accesos, rotación de credenciales y seguimiento de su caducidad, políticas, respuesta a incidentes y evidencias de auditoría.",
                    "Desarrollo servicios y herramientas internos en Rust y Python para los flujos de operaciones y soporte.",
                  ],
                  tags: ["Rust", "Python", "PostgreSQL", "Kubernetes", "Google Cloud", "OpenTofu", "OpenTelemetry"],
                },
                {
                  organization: "GenesisNest",
                  title: "Ingeniero de software",
                  period: "ene. 2025 - jul. 2025",
                  location: "Seongnam, Corea del Sur",
                  summary: "Servicios backend y herramientas de infraestructura para plataformas internas y la aplicación oficial para fans de un grupo de K-pop.",
                  highlights: [
                    "Aporté API, correcciones de errores e integraciones a los backends de las aplicaciones oficiales de Hyundai, Kia y Genesis, trabajando en una extensa base de código Java y coordinándome con proveedores de servicios utilizados por decenas de millones de personas en Asia y Europa.",
                    "Desarrollé canalizaciones de internacionalización en Rust y Python que transforman y cargan contenido traducido en bases de datos corporativas, coordinándome con los administradores de bases de datos responsables de conjuntos de datos vinculados a los sistemas de fábrica.",
                    "Desarrollé herramientas de gestión de infraestructura para servicios internos.",
                  ],
                  tags: ["Java", "Spring Boot", "Rust", "Python"],
                },
                {
                  organization: "pampam Inc.",
                  title: "Ingeniero de software por contrato",
                  period: "nov. 2024 - dic. 2024",
                  location: "Seúl, Corea del Sur",
                  highlights: [
                    "Diseñé el esquema PostgreSQL y el backend Rust de una plataforma de análisis de YouTube asistida por IA y dirigí su despliegue en AWS y Google Cloud.",
                    "Desarrollé flujos de recopilación con la API de YouTube que abarcaban decenas de millones de comentarios y usuarios, combinando datos de canales, transcripciones y contenido de páginas.",
                    "Integré modelos de lenguaje alojados y modelos de código abierto ejecutados en el servidor para generar resúmenes y análisis de marketing.",
                  ],
                  tags: ["Rust", "PostgreSQL", "AWS", "Google Cloud"],
                },
                {
                  organization: "EAN Technology Co. Ltd.",
                  title: "Ingeniero de software backend (líder)",
                  period: "ago. 2023 - ago. 2024",
                  location: "Seúl, Corea del Sur",
                  summary: "Dirigí el desarrollo de backend, bases de datos e infraestructura en la nube de dos plataformas de edificios y energía.",
                  highlights: [],
                  projects: [
                    {
                      name: "Gemelo digital del Global Engineering Center de Samsung C&T",
                      highlights: [
                        "Sustituí un backend Express y un conjunto de microservicios en la nube por un monolito Rust/Axum optimizado, lo que redujo el gasto mensual en infraestructura de unos **5.000 a 150 dólares**.",
                        "Escribí unas **30.000 líneas**, cerca del 75 % del backend, incluidos **80 endpoints REST** con lógica de dominio de ingeniería de la construcción, integración de datos y operaciones en tiempo real.",
                        "Reduje muchas rutas de solicitud con un P99 de varios segundos a milisegundos de una o dos cifras en un despliegue de dos núcleos y 4 GB, mediante Rust asíncrono, multihilo, caché y ajuste del esquema y las consultas de PostgreSQL.",
                        "Integré varios miles de sensores, contadores y controladores de climatización y de sistemas de agua mediante BACnet y Modbus, in situ en la sede de Samsung C&T.",
                        "El sistema funcionó nueve meses en producción sin ninguna parada en tiempo de ejecución, con una cobertura de gestión de errores de alrededor del 90 %.",
                        "Monté CI/CD con runners de GitHub, AWS CodeDeploy y Docker; las compilaciones estáticas ajustadas al procesador producían contenedores portables de unos 20 MB.",
                        "Conecté la inferencia de modelos predictivos mediante ECS, EC2, SQS, SNS, S3 y SageMaker, implementé los requisitos de seguridad de Samsung SDS e integré el backend con una interfaz en Unreal Engine 4 desarrollada por un estudio asociado.",
                      ],
                    },
                    {
                      name: "Servicio de gestión energética con IA",
                      highlights: [
                        "Diseñé desde cero el esquema PostgreSQL y la API Rust/Axum y escribí unas **25.000 líneas**, cerca del 70 % del backend.",
                        "Elegí SQL directo con tokio-postgres en lugar de un ORM para controlar las consultas y los tiempos de compilación, aplicando lo aprendido en el gemelo digital.",
                        "Ayudé a dos ingenieros procedentes de Node.js y Java a ser productivos en Rust en aproximadamente un mes, en un equipo de nueve personas formado por ingenieros de backend, frontend y ciencia de datos.",
                      ],
                    },
                  ],
                  tags: ["Rust", "Axum", "PostgreSQL", "AWS", "Docker", "BACnet", "Modbus"],
                },
                {
                  organization: "Artifyc Inc.",
                  title: "Becario de ingeniería de software",
                  period: "ago. 2022 - mar. 2023",
                  location: "Austin, Texas · Remoto",
                  highlights: [
                    "Desarrollé un bot de Discord en Python que usaba la API de Reddit para encontrar solicitudes de encargos artísticos y publicarlas en un canal de la comunidad, programado en AWS Lambda cada cinco minutos.",
                    "Implementé lógica en Lambda para la conversión de divisas, los cobros y el cálculo de recargos en distintas jurisdicciones.",
                    "Contribuí con revisiones del frontend React en el diseño, los recursos y los textos.",
                  ],
                  tags: ["Python", "AWS Lambda", "React"],
                },
              ],
            },
          ],
        },
        {
          id: "other",
          heading: "Antes del software",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Korea Institute of Maritime Science and Technology",
                  title: "Traductor e intérprete",
                  period: "jun. - jul. 2023, ago. 2024",
                  location: "Seúl, Corea del Sur · Contrato",
                  highlights: [
                    "Interpreté en inglés, coreano y noruego en una conferencia de investigación marítima a la que asistieron el embajador de Noruega en Corea, un secretario de Estado noruego, la dirección del instituto y el presidente de Dongwon Group.",
                    "Traduje discursos y preparé al personal del instituto y a su director para una conferencia internacional y para un evento del Departamento de Energía de Estados Unidos sobre un proyecto conjunto.",
                  ],
                },
                {
                  organization: "Ejército de la República de Corea · 2.ª División de Infantería del Ejército de EE. UU.",
                  title: "Sargento, enlace e instructor KATUSA",
                  period: "mar. 2016 - dic. 2017",
                  location: "Dongducheon, Corea del Sur",
                  highlights: [
                    "Serví en el Batallón de Cuartel General de la división a través del programa [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army), en un entorno conjunto coreano-estadounidense.",
                    "Formé y gestioné la incorporación de unos 6.000 militares estadounidenses y coreanos en cursos de preparación física, cultura, historia y prevención.",
                    "Dirigí un equipo de 13 personas y mantuve la preparación operativa durante la [crisis de 2017](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) en la península.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Operario de carga logística",
                  period: "2020 - 2022",
                  location: "Corea del Sur · Contratos puntuales",
                  highlights: [
                    "Trabajo nocturno en almacén compaginado con los estudios universitarios.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "Sobre mí",
      title: "Proyectos",
      lede: "Trabajo independiente y de código abierto, sobre todo en Rust. Las demostraciones interactivas están en la [página de proyectos](/projects).",
      sections: [
        {
          id: "featured",
          heading: "Destacados",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "Una crónica de trabajo de línea de comandos. Los objetivos, las tareas completadas, las correcciones y las retractaciones se registran como eventos de solo anexado con proyecciones actuales indexadas, y un archivo respaldado por Git combina los historiales entre equipos.",
                  tags: ["Event sourcing", "Sincronización"],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Una aplicación de escritorio y navegador para los datos del mapa de Europa Universalis V, con conjuntos de datos comprimidos integrados, índices de búsqueda y ordenación precalculados, filtros por categoría y tablas redimensionables. [Pruébala aquí](/eu5-locations-db).",
                  tags: ["Nativo + web", "Índices"],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Este sitio: publicación, fotografía, un foro, chat en directo con llamadas, un explorador de terreno de Minecraft y demostraciones en WebAssembly, servidos por un único binario Rust. [Cómo está construido](/about-blog).",
                  tags: ["Monorepo", "Autoalojado"],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "Un generador y formateador de UUIDv4 con salida a terminal y a archivo, comprobación opcional de duplicados y formato compatible con Python y JSON; unas tres veces más rápido que uuidgen en una primera prueba local.",
                  tags: ["Rendimiento"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "Más trabajos",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: una [corrección upstream](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) para el servidor de Minecraft en Rust que conserva la configuración del Overworld de Paper al importar y guardar; pendiente de revisión.",
                "**Herramientas de mapas de Minecraft**: un plugin de Paper y servicios Rust detrás del [explorador de terreno](/minecraft) de este sitio, incluido un formato binario compacto de teselas de biomas y predicciones de biomas basadas en la semilla.",
                "**Sincronización de sistemas de archivos**: una utilidad en Rust para sincronizar carpetas en ambos sentidos y limpiar artefactos de compilación entre sistemas de archivos heterogéneos, con ejecuciones de prueba, registros de operaciones, sumas de comprobación opcionales y salvaguardas contra borrados masivos no deseados.",
                "**impulsr**: una herramienta de transcripción de YouTube y recopilación de comentarios que genera resúmenes con modelos de lenguaje para trabajos de marketing.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Cómo construyo",
          blocks: [
            {
              kind: "paragraph",
              text: "Prefiero sistemas pequeños e inspeccionables: un solo binario cuando basta con uno, un límite explícito en cada cola y cada caché, y formatos de datos elegidos según el patrón de acceso y no por costumbre.",
            },
            {
              kind: "paragraph",
              text: "Las mediciones van antes que las afirmaciones; las [notas de rendimiento](/about-blog/performance) de este sitio muestran cómo se realizan.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "Sobre mí",
      title: "Trayectoria",
      lede: "Formación, investigación, cualificaciones e intereses fuera del software.",
      sections: [
        {
          id: "education",
          heading: "Formación académica",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Sungkyunkwan University",
                  title: "Grado en Ingeniería de Software (B.Eng.)",
                  period: "mar. 2015 - ago. 2023",
                  location: "Suwon, Corea del Sur",
                  summary: "Departamento de Ciencias de la Computación e Ingeniería. El periodo de la carrera incluye el servicio militar obligatorio.",
                  highlights: [
                    "Asignaturas de software embebido, circuitos lógicos, sistemas operativos, redes, programación de bajo nivel y electrónica, junto con prácticas de ingeniería de software.",
                    "La Universidad Sungkyunkwan remonta sus orígenes a 1398 como academia real de la dinastía Joseon y es una de las principales universidades de investigación de Corea.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Investigación",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Análisis emocional de comunidades en redes sociales mediante un modelo de PLN para el idioma coreano.** Apliqué KoBERT al análisis de sentimiento y la categorización de redes sociales en coreano, examinando el cambio emocional durante la primera etapa de la COVID-19.",
            },
            {
              kind: "paragraph",
              text: "El proyecto recibió el Premio de Bronce en los premios de trabajos y proyectos de fin de grado de la universidad, entre unas 60 candidaturas.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Premios y cualificaciones",
          blocks: [
            {
              kind: "table",
              columns: ["Año", "Reconocimiento", "Detalle"],
              rows: [
                ["2023", "**Premio de Bronce**", "Premios de trabajos y proyectos de fin de grado de la Universidad Sungkyunkwan"],
                ["2023", "**IELTS 8,5 / 9,0**", "MCER C2"],
                ["2021", "**7.º puesto**", "Hackathon de Diseño e Ideas Capstone, Korean Society for Internet Information"],
                ["2020", "**3.er puesto**", "Hackathon Fintech, Seoul National University of Science and Technology"],
                ["2018", "**TOEFL 117 / 120**", "MCER C2"],
              ],
            },
          ],
        },
        {
          id: "languages",
          heading: "Idiomas",
          blocks: [
            {
              kind: "list",
              items: [
                "**Coreano**: lengua materna.",
                "**Inglés**: dominio profesional (C2).",
                "**Francés, alemán, español y mandarín**: nociones básicas.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "Fuera del software",
          blocks: [
            {
              kind: "list",
              items: [
                "**Fotografía**: fotografía de paisaje y retrato desde 2010, con unas 30.000 imágenes archivadas. Hay una selección en la [galería](/photographs).",
                "**Periodismo**: reportero estudiantil del Sungkyun Times, la revista en inglés de la universidad, donde cubrí tecnología, seguridad y asuntos internacionales.",
                "**Docencia y voluntariado**: enseñé inglés en un centro comunitario y colaboré como voluntario en una tienda de segunda mano gestionada por personas con discapacidad.",
                "**Traducción**: interpretación y traducción para instituciones públicas; véase [Experiencia](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
