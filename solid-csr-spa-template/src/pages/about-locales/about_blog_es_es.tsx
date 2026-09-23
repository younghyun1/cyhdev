import { RepositorySourceMap } from "../../components/RepositorySourceMap";
import { pageStyles } from "../../styles/pageStyles";

export default function AboutBlog() {
  return (
    <main lang="es-ES" class={pageStyles.page}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">
            Tecnologías del blog
          </h1>
          <p class="text-xs text-ink-muted">
            Última actualización: 2026-08-30
            <br />
          </p>
        </div>

        <section class="space-y-8">
          {/* 0) Local */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              0) Máquina anfitriona, sistema operativo, sistema de archivos y configuración de red
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              El sitio está alojado en un miniserver en mi domicilio, detrás de una conexión Xfinity por cable de 1 Gbps. El{" "}
              <a
                href="https://store.minisforum.com/products/minisforum-um690l-slim"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                servidor anfitrión
              </a>{" "}
              cuenta con un procesador móvil Ryzen 9 de ocho núcleos que alcanza los 4,9 GHz, 32 GB de RAM a 6400 MT/s y un SSD NVMe de 1 TB. Por unos 400 dólares, es una ganga frente a pagar a AWS una cantidad equivalente durante un año por hardware mucho menos potente; además, refleja mi entusiasmo por el autoalojamiento. Ejecuta el servidor integrado de backend y frontend, la base de datos PostgreSQL y un servidor de Minecraft. Escríbeme si te gustaría jugar.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-1) Gentoo
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              No es precisamente el sistema operativo habitual para un servidor empresarial; si administrara uno en una empresa, seguramente elegiría Debian Stable, ext4 y cualquier motor de base de datos convencional. Sin embargo, disfruto trastear y compilar programas por mi cuenta, además de crear e instalar paquetes optimizados para la arquitectura de la CPU, algo que permite el magnífico{" "}
              <a
                href="https://www.gentoo.org/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                Gentoo
              </a>{" "}
              proyecto.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-2) Uso de btrfs en un servidor de base de datos, backend/frontend y Minecraft
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
              es un sistema de archivos moderno con instantáneas, CoW, compresión y muchas otras funciones. Sin embargo, no es la mejor opción para un servidor de bases de datos, por la fragmentación considerable que provoca CoW; lo he mitigado excluyendo de CoW los directorios de datos de PostgreSQL y Minecraft.{" "}
              <a
                href="https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                Una prueba de rendimiento de sistemas de archivos para PostgreSQL
              </a>{" "}
              muestra que btrfs no ofrece un gran rendimiento con la configuración predeterminada. Aun así, sospecho que desactivar CoW podría ponerlo al nivel de ext4 y xfs. Sería interesante comprobarlo con una prueba comparativa.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-3) Configuración de red interna y externa
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Al contratar el servicio de Internet por cable de Xfinity de 2 Gbps, no me di cuenta de que el módem-router incluido sería tan básico que <em>no</em> admitiría Ethernet de 2,5 Gbps. Así que tendré que conformarme con 1 Gbps, aunque no imagino que eso vaya a ser un problema para mi pequeño sitio web. Route 53 proporciona el DNS de mi dominio.
              <br />
              <br />
              Dentro de la red no hay motivo real para usar un proxy inverso, contenedores ni herramientas de servicios distribuidos: solo hay un motor PostgreSQL ejecutándose en el sistema operativo y un binario Rust que sirve tanto el frontend como la API, conectado a la base de datos mediante un socket UNIX, que resulta mucho más rápido que{" "}
              <a
                href="https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                usar localhost, como se explica aquí.
              </a>{" "}
              El rendimiento en transacciones por segundo puede casi duplicarse y evitar la pila de red redundante también reduce la latencia a la mitad; he observado tan solo 150 microsegundos entre la base de datos y el servidor. Con localhost solía obtener unos 600 microsegundos. No es un caso relevante para las configuraciones empresariales en la nube, pero resulta curioso. Es un enfoque clásico y más rápido. PostgreSQL 18, publicado en noviembre de 2025, también incorporó E/S asíncrona mediante{" "}
              <a
                href="https://pganalyze.com/blog/postgres-18-async-io"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                compatibilidad con io_uring
              </a>{" "}
              que ya está habilitada. Acelera bastante las lecturas.
            </p>
          </section>

          {/* 1) PostgreSQL */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              1) Datos
            </h2>
            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-1) PostgreSQL 18
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Durante mi trayectoria en startups y empresas de Corea, he observado una cultura que considera MySQL o MariaDB los únicos sistemas gestores de bases de datos relacionales que valen la pena; todavía no entiendo por qué. Algunos antiguos jefes me dijeron que PostgreSQL ni siquiera se consideraba una opción seria, algo extraño dado que en la última década ha avanzado mucho en rendimiento, extensibilidad, tipos de datos y herramientas, y posiblemente ha superado a MySQL en muchos aspectos, especialmente en la compatibilidad con UUID y la codificación binaria de datos JSON.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-2) Aspectos destacados del esquema (blog y autenticación)
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              El esquema es deliberadamente sencillo, en el buen sentido: usuarios, sesiones en memoria, tokens de verificación de correo y restablecimiento de contraseña, publicaciones, comentarios, tablas de votos, etiquetas, fotos de perfil y algunas tablas auxiliares de geografía e internacionalización. El blog no consiste simplemente en una tabla <code>posts</code> y cruzar los dedos. Las publicaciones tienen slugs, resúmenes, metadatos JSONB, estado de publicación, contadores desnormalizados y relaciones con etiquetas. Los comentarios se anidan mediante un identificador nullable del comentario padre. Los votos se guardan en tablas separadas para publicaciones y comentarios, lo que simplifica las consultas y evita después condiciones absurdas.
              <br />
              <br />
              La autenticación es igual de práctica: el registro del usuario guarda el país y el idioma para que el sitio haga algo más que pedirte el correo y olvidarse de ti. Hay tablas de roles y permisos porque no quiero limitar mis opciones de autorización. Las fotos de perfil tienen versiones en una tabla propia, en vez de mezclarse con la fila del usuario; así es más sencillo reemplazarlas y no se carga el registro de usuario más consultado con datos ajenos.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-3) UUIDv7
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Casi todos los recursos visibles para el usuario se identifican con UUID, y no es casualidad. No me gusta exponer identificadores secuenciales que permitan deducir cuántas filas hay o enumerar recursos como en 2009. Los UUID ordenados por tiempo también son más amables con los índices que los UUIDv4 completamente aleatorios, algo útil cuando las escrituras son reales. En otras palabras: son únicos globalmente, difíciles de adivinar y más favorables a la localidad. Las inserciones son más regulares, hay menos reorganización en los árboles B y la base de datos dedica menos tiempo a tareas de mantenimiento evitables.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-4) Diagrama (recorrido de la solicitud y los datos)
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Un diagrama de arquitectura empresarial muy glamuroso, presentado como texto porque aún no me he molestado en dibujar cajitas:
              <br />
              <br />
              Solicitud del navegador -&gt; enrutador Axum -&gt; cadena de middleware (registro, búsqueda de autenticación/sesión, límite de solicitudes y comprobación del tamaño del cuerpo) -&gt; controlador -&gt; consulta asíncrona de Diesel o búsqueda en caché en memoria -&gt; PostgreSQL mediante socket UNIX -&gt; respuesta DTO -&gt; respuesta HTTPS comprimida al navegador.
              <br />
              <br />
              Los recursos estáticos siguen un recorrido aún más corto. La aplicación SolidJS compilada se integra directamente en el binario Rust y se sirve con negociación de contenido zstd/gzip si el navegador lo admite. No hay un proceso Node activo en producción, ni un servidor independiente de archivos estáticos, ni un salto adicional entre «llegó la solicitud» y «salieron los bytes».
            </p>
          </section>

          {/* 2) Backend */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              2) Backend (Rust)
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              El backend es un servicio Rust construido con Axum, Tokio, Diesel y PostgreSQL, con mimalloc como asignador global. Si se alojan servicios en una máquina con núcleos reales, conviene tomarse en serio el comportamiento del asignador. El servicio termina TLS directamente con rustls, sirve la SPA integrada, expone API JSON para autenticación, blog, fotografía e internacionalización, y envía estadísticas del servidor por WebSockets al panel de control. El enrutador incluye compresión de solicitudes, CORS permisivo por ahora, límites de solicitudes bastante generosos y una interfaz Swagger protegida por autenticación en producción.
              <br />
              <br />
              En la aplicación, el diseño favorece un solo proceso con un objeto compartido de estado del servidor, gestión de sesiones en memoria, sincronización de caché al iniciar y tareas de mantenimiento programadas. La autenticación usa cookies seguras HTTP-only, verificación de correo, tokens para restablecer contraseñas y comprobaciones explícitas de rol para operaciones de superusuario. Lo más interesante es la latencia: el tráfico de la base de datos usa un socket UNIX, el tamaño del grupo de conexiones se ajusta al número de núcleos físicos, las listas del blog se leen en gran medida desde la caché y las respuestas se enriquecen por lotes, en vez de acumular consultas diminutas. Así la pila es rápida en lo que me importa: menos saltos, copias y viajes de ida y vuelta, y menos espera a que una abstracción se felicite a sí misma.
            </p>
          </section>

          {/* 3) Frontend */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              3) Frontend
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              El frontend es una aplicación de página única (SPA) de SolidJS que se ejecuta en el cliente y está construida con Vite y TypeScript. Las rutas se cargan de forma diferida, el estado se mantiene sencillo y todo se compila en recursos estáticos que se incluyen en el binario Rust durante el despliegue. Así no hay que mantener un proceso de servidor JavaScript en producción; el servidor Rust entrega directamente los archivos estáticos con la mayor eficiencia posible.
              <br />
              <br />
              La interfaz del blog combina practicidad con mi negativa a usar editores de juguete. La edición Markdown se realiza con Toast UI Editor, y las imágenes pegadas o cargadas pasan directamente por la API de carga de fotografías, para que redactar una publicación no sea una experiencia penosa. La búsqueda admite títulos y etiquetas, la navegación usa parámetros de consulta, el estado de autenticación se sigue en el cliente pero se verifica en el servidor, y la aplicación lee los encabezados de respuesta para mostrar la información de compilación del servidor. Los estilos usan Tailwind y un sistema compartido. Solid mantiene una ejecución ligera y reduce los cambios del DOM, justo lo que espero de una capa de interfaz cuyo trabajo principal es no estorbar.
            </p>
          </section>

          {/* 4) Red y HTTPS */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              4) HTTPS, enrutamiento y medidas de seguridad
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              HTTPS lo gestiona directamente el servidor Rust con rustls. Las solicitudes HTTP se redirigen a HTTPS, las cookies se marcan como seguras y HTTP-only, y en producción el dominio de las cookies se limita al dominio del sitio. Los recursos estáticos se sirven con zstd o gzip cuando es compatible, y el enrutamiento de reserva de la SPA permite que funcionen los enlaces profundos sin depender de otra capa de proxy inverso.
              <br />
              <br />
              Las medidas de protección no son especialmente sofisticadas, pero existen: límites al tamaño del cuerpo de las solicitudes para las cargas, registros mediante middleware, controles de autenticación en rutas protegidas, comprobaciones de superusuario en puntos de acceso sensibles, límites de solicitudes para dificultar los abusos y compatibilidad con claves de API para solicitudes de clientes. Además, el despliegue es fácil de entender: un binario, una base de datos, un anfitrión y TLS en la aplicación, con pocas oportunidades para latencias misteriosas o desviaciones de configuración. No es una arquitectura de moda, pero es rápida, observable y consume pocos recursos.
            </p>
          </section>

          <RepositorySourceMap />
        </section>
      </section>
    </main>
  );
}
