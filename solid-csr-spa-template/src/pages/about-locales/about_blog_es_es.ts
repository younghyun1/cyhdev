import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "noviembre de 2026",
  chrome: {
    overview: "Resumen",
    updated: "Última actualización",
    onThisPage: "En esta página",
    sectionNavigation: "Páginas de tecnologías",
    previous: "Anterior",
    next: "Siguiente",
    readingTime: "{minutes} min de lectura",
  },
  pages: {
    overview: {
      eyebrow: "Sobre este sitio",
      title: "Tecnologías del blog",
      lede: "Un binario Rust, una base de datos PostgreSQL y una máquina pequeña. Estas páginas describen cómo se construye, aloja, protege y mide el sitio, y por qué tiene esta forma.",
      sections: [
        {
          id: "at-a-glance",
          heading: "De un vistazo",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53.889 req/s",
                  label: "Rendimiento mediano en lecturas públicas mixtas con PGO y BOLT en cuatro núcleos del servidor",
                },
                {
                  value: "~150 µs",
                  label: "Ida y vuelta observada con la base de datos mediante un socket UNIX, frente a ~600 µs por TCP en localhost",
                },
                {
                  value: "0",
                  label: "Procesos de Node.js en producción; el frontend va integrado en el binario del servidor",
                },
                {
                  value: "24",
                  count: 24,
                  label: "Idiomas de interfaz, cada uno descargado solo al seleccionarlo",
                },
                {
                  value: "4.096",
                  label: "Conexiones abiertas admitidas antes de TLS, con un máximo de 64 por dirección o /64 de IPv6",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Proceso que sirve TLS, la API, WebSockets, los medios WebRTC y el frontend integrado",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Recorrido de una solicitud",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Navegador",
                  detail: "HTTP/2 o HTTP/1.1 sobre TLS. El frontend se almacena en caché con URL inmutables que incluyen una huella.",
                },
                {
                  label: "Admisión",
                  detail: "Las conexiones TCP se admiten antes de TLS: 4.096 en total y 64 por dirección o por /64 de IPv6.",
                },
                {
                  label: "rustls",
                  detail: "TLS termina dentro del proceso Rust, con `TCP_NODELAY` en cada socket.",
                },
                {
                  label: "Axum",
                  detail: "Encabezados de seguridad, sesiones, límites de frecuencia, compresión y límites de tamaño del cuerpo; después, el controlador de la funcionalidad.",
                },
                {
                  label: "Caché · Diesel",
                  detail: "Cachés en memoria acotadas responden a las lecturas frecuentes; todo lo demás pasa por Diesel asíncrono.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "En el mismo servidor mediante un socket UNIX, con E/S asíncrona io_uring.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Capítulos",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Infraestructura",
                  href: "/about-blog/infrastructure",
                  meta: "Servidor · Gentoo · red",
                  text: "La máquina, el sistema operativo y el sistema de archivos, la red y por qué no hay proxy inverso.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "El stack del servicio, el modelo de datos, los límites en tiempo de ejecución, los encabezados de seguridad y los contratos de API.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "La estructura de la aplicación, la carga, la localización, el sistema de diseño y las animaciones.",
                },
                {
                  title: "Subsistemas",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "El chat en directo con una SFU dentro del proceso, el explorador de Minecraft, las búsquedas Geo-IP y WebAssembly.",
                },
                {
                  title: "Rendimiento",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Las compilaciones optimizadas, cómo se midieron y los resultados.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Cómo se verifican los cambios",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy en los destinos nativos y WebAssembly como control de implementación.",
                "Comprobaciones locales con cargo xtask del formato, las pruebas unitarias nativas, la deriva del contrato OpenAPI y, en el frontend, la comprobación de tipos, el lint, las pruebas unitarias y la compilación.",
                "Playwright en Chromium y WebKit con comprobaciones de accesibilidad de axe y de reajuste en móvil hasta 320 píxeles CSS.",
                "Pruebas de integración y de reversión de migraciones contra una instancia desechable de PostgreSQL, además de un análisis local de secretos con gitleaks.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Tecnologías",
      title: "Infraestructura",
      lede: "Una máquina pequeña en casa, una distribución Linux basada en código fuente y ninguna capa que el tráfico no necesite.",
      sections: [
        {
          id: "host",
          heading: "El servidor",
          blocks: [
            {
              kind: "paragraph",
              text: "El sitio se ejecuta en un [miniservidor](https://store.minisforum.com/products/minisforum-um690l-slim) en mi domicilio: un procesador móvil Ryzen 9 de ocho núcleos que alcanza los 4,9 GHz, 32 GB de RAM y un SSD NVMe de 1 TB. Por unos 400 dólares, cuesta menos que un año de hardware en la nube notablemente menos potente, y refleja mi entusiasmo por el autoalojamiento.",
            },
            {
              kind: "paragraph",
              text: "Ejecuta el servidor combinado de backend y frontend, PostgreSQL y un servidor de Minecraft. Si quieres jugar, escríbeme.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo y btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) no es la opción obvia para un servidor; en el trabajo elegiría Debian estable sobre ext4. Aquí disfruto compilando yo mismo los paquetes con optimizaciones para el procesador concreto.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) aporta instantáneas, compresión y copia en escritura, pero la copia en escritura fragmenta mucho los archivos de bases de datos. Por eso, los directorios de datos de PostgreSQL y Minecraft quedan excluidos de ella. Una [prueba de rendimiento de sistemas de archivos](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) muestra a btrfs con la configuración predeterminada por detrás de ext4 y XFS; sospecho que desactivar la copia en escritura cierra la mayor parte de esa diferencia, lo que sería una medición interesante.",
            },
          ],
        },
        {
          id: "network",
          heading: "Red",
          blocks: [
            {
              kind: "paragraph",
              text: "El tráfico llega por una conexión Xfinity por cable de 1 Gbps; el módem del proveedor no tiene Ethernet de 2,5 Gb, algo que este sitio no notará. Route 53 aloja el DNS.",
            },
            {
              kind: "paragraph",
              text: "Dentro de la máquina no hay proxy inverso, entorno de ejecución de contenedores ni malla de servicios. PostgreSQL se ejecuta en el sistema operativo y un único binario Rust sirve el frontend y la API. El mismo proceso redirige a HTTPS las solicitudes HTTP sin cifrar.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "Sockets UNIX",
          blocks: [
            {
              kind: "paragraph",
              text: "El servidor se conecta a PostgreSQL mediante un socket UNIX en lugar de TCP en localhost, lo que evita por completo la pila de red. Las [pruebas de rendimiento](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) muestran tasas de transacciones casi duplicadas; aquí he observado idas y vueltas de apenas **150 µs**, frente a unos **600 µs** por localhost. Es anticuado e irrelevante para la mayoría de los despliegues en la nube, y es más rápido.",
            },
            {
              kind: "paragraph",
              text: "PostgreSQL 18, publicado en septiembre de 2025, añadió [E/S asíncrona](https://pganalyze.com/blog/postgres-18-async-io), e io_uring está habilitado aquí, lo que acelera notablemente las lecturas.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Compilación y despliegue",
          blocks: [
            {
              kind: "list",
              items: [
                "La compilación del frontend se integra en el binario del backend junto con copias precomprimidas en zstd y gzip, de modo que en producción no se ejecuta ningún proceso Node.js.",
                "Las compilaciones optimizadas pueden entrenarse con optimización guiada por perfiles y optimizarse tras el enlazado con BOLT; véase [Rendimiento](/about-blog/performance).",
                "Las credenciales, los certificados y los datos de ejecución quedan fuera de Git, y un análisis con gitleaks revisa el árbol antes de que los cambios salgan de la máquina.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Tecnologías",
      title: "Backend",
      lede: "Rust sobre Axum y Tokio, PostgreSQL mediante Diesel asíncrono y un límite explícito para todo lo que un cliente puede mantener abierto.",
      sections: [
        {
          id: "stack",
          heading: "Stack tecnológico",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "Ejecución", items: ["Axum", "Tokio", "hyper", "axum-server", "rustls", "mimalloc"] },
                { label: "Datos", items: ["PostgreSQL 18", "Diesel", "diesel-async", "tantivy", "bitcode", "zstd"] },
                { label: "Integraciones", items: ["openidconnect", "lettre", "Amazon S3", "webrtc-rs", "image + AVIF"] },
                { label: "Contratos", items: ["utoipa", "OpenAPI", "Swagger UI", "Generated TypeScript"] },
              ],
            },
            {
              kind: "paragraph",
              text: "El servicio es un único proceso con estado compartido: sesiones en memoria, sincronización de caché al arrancar y tareas de mantenimiento programadas. mimalloc es el asignador global; si una máquina tiene núcleos reales, vale la pena tomarse en serio el comportamiento del asignador.",
            },
          ],
        },
        {
          id: "data",
          heading: "Modelo de datos",
          blocks: [
            {
              kind: "paragraph",
              text: "En las startups y empresas coreanas que he conocido, MySQL y MariaDB se consideraban las únicas bases de datos relacionales serias. PostgreSQL las superó hace tiempo en los aspectos que me importan, desde los UUID nativos hasta el JSON binario.",
            },
            {
              kind: "list",
              items: [
                "Las **publicaciones** incluyen slugs, resúmenes, metadatos JSONB, estado de publicación, contadores desnormalizados y relaciones con etiquetas. Los comentarios se anidan mediante un padre que admite nulos, y los votos de publicaciones y comentarios tienen tablas separadas.",
                "Las **cuentas** guardan el país y el idioma, tablas de roles y permisos, y fotos de perfil versionadas en su propia tabla en lugar de en la fila de usuario más consultada.",
                "Las **claves** son valores UUIDv7 ordenados por tiempo: no se pueden enumerar como los ID secuenciales y son mucho más favorables a la localidad de los árboles B que los UUIDv4 aleatorios.",
                "La **búsqueda** usa un índice tantivy persistido en disco sobre los títulos y las etiquetas de las publicaciones, que se mantiene coherente con PostgreSQL, que sigue siendo la fuente autorizada.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Límites en tiempo de ejecución",
          blocks: [
            {
              kind: "list",
              items: [
                "Las conexiones se admiten antes de TLS mediante un limitador compartido: 4.096 conexiones abiertas y 64 por dirección IPv4 o /64 de IPv6. Los sockets rechazados se cierran sin handshake.",
                "Los encabezados de las solicitudes HTTP/1 deben llegar en 20 segundos; las conexiones HTTP/2 reciben un ping cada 30 segundos y admiten 128 flujos simultáneos.",
                "Cada caché tiene un límite fijo y una fuente autorizada a la que puede recurrir en lectura; ninguna crece con claves que controle un cliente. La caché de bundles WebAssembly, por ejemplo, ocupa como máximo 256 MiB.",
                "El trabajo bloqueante, como la codificación de imágenes, se ejecuta en grupos de workers acotados en lugar de en los workers de Tokio.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Encabezados de seguridad",
          blocks: [
            {
              kind: "paragraph",
              text: "La Content-Security-Policy se construye una sola vez al arrancar. Los scripts en línea solo se permiten mediante hashes SHA-256 calculados a partir del `index.html` compilado en el binario, de modo que el script de tema previo al primer pintado no puede desviarse de la política. El origen principal nunca instancia WebAssembly: las demostraciones y la aplicación EU5 se ejecutan en marcos aislados con sus propias políticas.",
            },
            {
              kind: "paragraph",
              text: "Las cookies de sesión son `Secure` y `HttpOnly`, las operaciones de superusuario comprueban los roles en el servidor, las cuentas pueden vincular identidades OpenID Connect y Swagger UI requiere autenticación en producción.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Contratos y recursos",
          blocks: [
            {
              kind: "list",
              items: [
                "Los controladores llevan anotaciones OpenAPI, los clientes TypeScript del frontend se generan a partir de ellas y cargo xtask openapi falla cuando los archivos generados se desvían.",
                "El manifiesto de Vite se integra junto con los recursos. Solo los archivos con huella incluidos en el manifiesto reciben una política de caché `immutable` de un año; el HTML siempre se revalida.",
                "Las rutas de recursos inexistentes devuelven 404 con `no-store` en lugar de la estructura de la aplicación, de modo que una página obsoleta no puede almacenar en caché HTML bajo la URL de un script.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Tecnologías",
      title: "Frontend",
      lede: "Una aplicación SolidJS 2 renderizada en el cliente, compilada en recursos estáticos e integrada en el binario del servidor.",
      sections: [
        {
          id: "stack",
          heading: "Stack tecnológico",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "Aplicación", items: ["SolidJS 2", "Solid Router 2", "TypeScript", "Vite 8"] },
                { label: "Estilos", items: ["Tailwind CSS 4", "IBM Plex Sans", "IBM Plex Mono", "OKLab"] },
                { label: "Componentes", items: ["Toast UI Editor", "Leaflet", "Chart.js", "highlight.js", "DOMPurify"] },
                { label: "Pruebas", items: ["Vitest", "Playwright", "axe-core"] },
              ],
            },
            {
              kind: "paragraph",
              text: "Las señales de grano fino de Solid actualizan solo los nodos del DOM que dependen de los valores modificados, y sus versiones candidatas 2.0 permiten que las lecturas asíncronas se suspendan hasta el límite de carga más cercano. El enrutador carga cada página de forma diferida y no hay servidor JavaScript en producción: el binario Rust entrega archivos estáticos.",
            },
          ],
        },
        {
          id: "loading",
          heading: "Carga",
          blocks: [
            {
              kind: "list",
              items: [
                "Las páginas, el panel de chat de la página de inicio y su shader de curvas de nivel son chunks separados, y la compilación informa del tamaño gzip de lo que se carga primero.",
                "Solo el texto de la interfaz en inglés va en el chunk inicial. Los demás idiomas, y las páginas «Acerca de» de cada idioma, son chunks separados que se cargan al seleccionarlos.",
                "Las pruebas de rendimiento móvil con limitación apuntan a un LCP de 2,5 s o menos, interacciones de 200 ms o menos y un CLS de 0,1 o menos.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Localización",
          blocks: [
            {
              kind: "paragraph",
              text: "La interfaz admite 24 idiomas, con diseño de derecha a izquierda para árabe, persa y urdu. Un catálogo de unas 740 claves se valida con el registro del backend, y los marcadores de posición deben sobrevivir a la traducción. Estas páginas «Acerca de» son módulos de datos tipados, así que una traducción no puede omitir en silencio una sección que exista en inglés.",
            },
          ],
        },
        {
          id: "design",
          heading: "Sistema de diseño",
          blocks: [
            {
              kind: "paragraph",
              text: "El aspecto es cálido y técnico: papel crema o negro puro, bordes finos, tipografía monoespaciada para todo lo técnico y un solo color de acento a la vez. Los colores son variables CSS semánticas, así que los componentes nunca necesitan variantes para el modo oscuro. El contraste de los ajustes de acento con los fondos de página se comprueba con pruebas, un color RGB introducido por el visitante se ajusta en cada modo para seguir siendo legible, y el modo claro u oscuro y el acento se aplican antes del primer pintado.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Animaciones",
          blocks: [
            {
              kind: "list",
              items: [
                "La aparición de las secciones y la barra de progreso de lectura usan animaciones CSS controladas por el desplazamiento, que se ejecutan fuera del hilo principal donde hay compatibilidad.",
                "Las estadísticas cuentan hacia arriba mediante un entero registrado con `@property`, con el valor final presente en el marcado para las tecnologías de apoyo.",
                "El cambio de tema se expande desde el interruptor mediante la API View Transitions.",
                "El campo de curvas de nivel de la página de inicio es un pequeño shader WebGL que se carga cuando la página queda inactiva, solo se ejecuta en navegadores de escritorio capaces, limita su tasa de fotogramas y se pausa fuera de la pantalla.",
                "Un ajuste del sitio lo desactiva todo. Las animaciones están activadas por defecto y no siguen el ajuste de movimiento reducido del sistema operativo.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Tecnologías",
      title: "Subsistemas",
      lede: "Las partes del sitio más interesantes de lo que un blog necesita estrictamente.",
      sections: [
        {
          id: "live-chat",
          heading: "Chat en directo y llamadas",
          blocks: [
            {
              kind: "paragraph",
              text: "El chat funciona sobre un WebSocket con un códec binario compacto, moderación e historial acotado. Las llamadas de voz y vídeo pasan por una unidad de reenvío selectivo (SFU) que se ejecuta dentro del proceso Axum sobre webrtc-rs: cada navegador mantiene una sola conexión entre pares y la SFU reenvía los medios codificados sin transcodificar, mediante canales acotados por publicación y un límite de participantes. [Abrir el chat en directo](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Explorador de terreno de Minecraft",
          blocks: [
            {
              kind: "paragraph",
              text: "Un plugin de Paper expone un socket UNIX restringido al mismo usuario para consultas acotadas sobre el mundo, con una operación a la vez y límites estrictos de tamaño de trama. El [explorador](/minecraft) dibuja las teselas de squaremap con su propio cliente Leaflet en lugar de ejecutar el JavaScript del mapa generado.",
            },
            {
              kind: "paragraph",
              text: "Las capas de biomas usan **CYBM**, un formato binario compacto de teselas servido con compresión gzip, o un PNG cuyo chunk privado `cyBM` contiene las identidades exactas de los biomas, de modo que al pasar el cursor nunca se adivina a partir de los colores de los píxeles. Las predicciones basadas en la semilla para el terreno inexplorado proceden de un worker de generación de mundos independiente basado en Pumpkin.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Búsquedas Geo-IP",
          blocks: [
            {
              kind: "paragraph",
              text: "El [mapa de visitantes](/visitor-board) y la [base de datos Geo-IP](/geo-ip-db) localizan direcciones sin llamar a ninguna API externa. Una herramienta de preprocesamiento convierte los archivos CSV de IP2Location LITE en `BTreeMap` indexados por el inicio del rango, los codifica con bitcode y los comprime con zstd en el nivel 22.",
            },
            {
              kind: "table",
              columns: ["Conjunto de datos", "CSV", "Bundle"],
              numericColumns: [1, 2],
              rows: [
                ["IPv4", "312,5 MB", "24,9 MB"],
                ["IPv6", "794,2 MB", "42,4 MB"],
              ],
            },
            {
              kind: "paragraph",
              text: "Al arrancar, el servidor decodifica un bundle cada vez para limitar el pico de memoria y aplica interning a los nombres de lugar repetidos. Una búsqueda encuentra el último rango que empieza en la dirección o antes y comprueba su límite final.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "Los [bundles de demostración](/projects) subidos se guardan temporalmente en disco, se identifican por sus bytes mágicos, se limitan a 50 MiB descomprimidos y se sirven desde una caché acotada dentro de marcos aislados. El espacio de trabajo también compila demostraciones en Rust, entre ellas un rompeladrillos y un trazador de rayos.",
            },
            {
              kind: "paragraph",
              text: "[EU5 Location DB](/eu5-locations-db) es una aplicación Slint compilada a WebAssembly. La página envía su modo claro u oscuro mediante un pequeño protocolo de mensajes, de modo que la paleta integrada cambia con el sitio.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Flujo de procesamiento de fotografías",
          blocks: [
            {
              kind: "paragraph",
              text: "Las subidas se decodifican dentro de límites fijos de píxeles y se codifican en AVIF con el ajuste más lento, a cambio de más tiempo de procesamiento y archivos más pequeños. Las ubicaciones de los datos EXIF pueden confirmarse en un mapa antes de publicar, y no se ejecutan más de dos trabajos de procesamiento a la vez.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Tecnologías",
      title: "Rendimiento",
      lede: "Optimización guiada por perfiles y BOLT en el backend, medidas con un generador de carga que no es el cuello de botella.",
      sections: [
        {
          id: "approach",
          heading: "Enfoque",
          blocks: [
            {
              kind: "paragraph",
              text: "Las compilaciones optimizadas se entrenan con una carga de trabajo representativa. Un binario instrumentado registra perfiles frente a un entorno de pruebas desechable con endpoints sintéticos de PostgreSQL, SMTP, S3, OpenID Connect y Minecraft. LLVM recompila el servidor con esos perfiles (PGO) y, después, BOLT reordena la disposición del código del binario final a partir de una ejecución muestreada. Todas las variantes comparten LTO, punteros de marco y una biblioteca estándar recompilada.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Configuración",
          blocks: [
            {
              kind: "list",
              items: [
                "Servidor, proveedores del entorno de pruebas y PostgreSQL fijados a cuatro núcleos de un AMD Ryzen 7 9800X3D; el cliente de carga, fijado a los otros cuatro.",
                "HTTPS sobre HTTP/1.1 con un pool de 32 conexiones, lectura completa de las respuestas y plazos de dos segundos.",
                "Tres rondas rotadas con cinco muestras por variante, cada una tras 10.000 solicitudes de calentamiento, a partir de una instantánea nueva de la base de datos.",
                "27.000.000 de solicitudes cronometradas en total; todas devolvieron HTTP 200.",
                "La carga la genera oha; Playwright comprueba la respuesta de cada ruta fuera de la ventana cronometrada.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Resultados",
          blocks: [
            {
              kind: "bars",
              caption: "Mediana de solicitudes por segundo en 15 muestras por variante.",
              groups: [
                {
                  label: "Lecturas públicas de diez rutas",
                  items: [
                    { label: "Referencia", value: 44874.12, display: "44.874" },
                    { label: "PGO", value: 52925.46, display: "52.925" },
                    { label: "PGO + BOLT", value: 53888.61, display: "53.889", emphasis: true },
                  ],
                },
                {
                  label: "JSON pequeño de estado",
                  items: [
                    { label: "Referencia", value: 272837.41, display: "272.837" },
                    { label: "PGO", value: 308685.26, display: "308.685", emphasis: true },
                    { label: "PGO + BOLT", value: 306276.64, display: "306.277" },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "Latencia de cola para la mezcla de diez rutas.",
              columns: ["Variante", "p95 (ms)", "p99 (ms)"],
              numericColumns: [1, 2],
              rows: [
                ["Referencia", "2,380", "2,967"],
                ["PGO", "2,020", "2,524"],
                ["PGO + BOLT", "1,987", "2,483"],
              ],
            },
            {
              kind: "paragraph",
              text: "PGO aumentó el rendimiento mixto un **17,9 %** y PGO con BOLT un **20,1 %**, y ambas compilaciones optimizadas ganaron todas las rondas rotadas. BOLT añadió un 1,8 % sobre PGO en las lecturas mixtas y fue un 0,8 % más lento en la ruta de estado trivial. Son resultados con cuatro núcleos; la capacidad en producción también depende de la red, del conjunto de trabajo de la base de datos y del tráfico real.",
            },
          ],
        },
        {
          id: "corrections",
          heading: "Correcciones",
          blocks: [
            {
              kind: "list",
              items: [
                "Al principio `TCP_NODELAY` no estaba activado: el aceptador Rustls predeterminado de axum-server lo deja desactivado, por lo que las respuestas HTTP/2 podían esperar unos 40 ms un acuse de recibo diferido. Ahora se activa en cada socket aceptado.",
                "La primera prueba generaba la carga desde Playwright, cuyo cliente se ralentizaba a medida que avanzaba la ejecución, y mostró regresiones que no existían. Esas cifras se descartaron.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
