import { Component } from "solid-js";

const CodeBlock: Component<{
  title: string;
  code: string;
  href: string;
  language: "sql" | "rust" | "typescript";
}> = (props) => {
  let html = props.code.replace(/</g, "&lt;").replace(/>/g, "&gt;");

  if (props.language === "sql") {
    html = html
      .replace(
        /\b(CREATE|TABLE|PRIMARY|KEY|DEFAULT|NOT|NULL|CONSTRAINT|FOREIGN|REFERENCES|ON|DELETE|CASCADE|INDEX|UNIQUE|IF|EXISTS)\b/g,
        '<span class="text-pink-400 font-bold">$1</span>',
      )
      .replace(
        /\b(UUID|VARCHAR|TEXT|TIMESTAMPTZ|BOOLEAN)\b/g,
        '<span class="text-blue-400">$1</span>',
      )
      .replace(
        /\b(uuidv7|now|true|false)\b/g,
        '<span class="text-purple-400">$1</span>',
      )
      .replace(/--.*$/gm, '<span class="text-gray-500 italic">$&</span>');
  } else if (props.language === "rust") {
    // Rust highlighting
    html = html
      .replace(
        /\b(pub|use|struct|impl|async|fn|let|mut|if|else|match|return|crate|mod|static|const|type|enum|trait|for|while|loop|await|where|derive)\b/g,
        '<span class="text-pink-400 font-bold">$1</span>',
      )
      .replace(
        /\b(String|u64|usize|Vec|Option|Result|bool|str|i32|f64|u8|i8|i64|u32|i16|u16|Uuid|Arc|ServerState|Session|PostInfo|HashSet|HashMap|RwLock|AtomicU64|Pool|AsyncPgConnection|Uri|IntoResponse|StatusCode|HeaderMap|Embed|State|CookieJar|Request|Body|Next|HandlerResponse|DefaultBodyLimit|GovernorLayer)\b/g,
        '<span class="text-blue-400">$1</span>',
      )
      .replace(
        /\b(true|false|Some|None|Ok|Err|scc|tokio)\b/g,
        '<span class="text-purple-400">$1</span>',
      )
      .replace(/\/\/.*$/gm, '<span class="text-gray-500 italic">$&</span>')
      .replace(/#!?\[.*\]/g, '<span class="text-yellow-400">$&</span>'); // Attributes
  } else if (props.language === "typescript") {
    html = html
      .replace(
        /\b(const|let|var|export|import|from|function|async|await|return|if|else|try|catch|throw|new|typeof|instanceof|void|interface|type|declare|namespace|module|as)\b/g,
        '<span class="text-pink-400 font-bold">$1</span>',
      )
      .replace(
        /\b(Promise|string|number|boolean|any|unknown|null|undefined|void|object|Array|Record|RequestInit|Headers|Response|Element|HTMLElement|RouteDefinition|MeResponse)\b/g,
        '<span class="text-blue-400">$1</span>',
      )
      .replace(
        /\b(true|false|this|window|document|console|sessionStorage|localStorage|JSON|fetch|Error|import|meta|env)\b/g,
        '<span class="text-purple-400">$1</span>',
      )
      .replace(/\/\/.*$/gm, '<span class="text-gray-500 italic">$&</span>');
  }

  return (
    <div class="my-6 rounded-lg overflow-hidden border border-gray-200 dark:border-gray-800 bg-[#1e1e1e] shadow-md dark:shadow-none ring-1 ring-gray-900/5">
      <div class="flex items-center justify-between px-4 py-2 bg-[#252526] border-b border-gray-700">
        <span class="text-xs font-mono text-gray-400 truncate flex-1">
          {props.title}
        </span>
        <a
          href={props.href}
          target="_blank"
          rel="noopener noreferrer"
          class="text-xs text-sky-400 hover:text-sky-300 hover:underline whitespace-nowrap ml-4"
        >
          GitHub ↗
        </a>
      </div>
      <div class="p-4 overflow-x-auto bg-[#1e1e1e]">
        <pre class="text-xs sm:text-sm font-mono leading-relaxed text-[#d4d4d4]">
          <code innerHTML={html}></code>
        </pre>
      </div>
    </div>
  );
};

export default function AboutBlog() {
  return (
    <div class="bg-sky-50 dark:bg-black min-h-screen transition-colors duration-90">
      <section class="max-w-3xl mx-auto text-gray-700 dark:text-gray-100 p-8">
        <div class="border-l-4 border-sky-500 dark:border-sky-400 pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">
            Blog Tech Stack
          </h1>
          <p class="text-xs text-gray-500 dark:text-gray-400">
            Last updated: 2026-01-14<br></br>
          </p>
        </div>

        <section class="space-y-8">
          {/* 0) On-Prem */}
          <section class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-950/40 p-5">
            <h2 class="text-lg font-bold mb-2 text-gray-900 dark:text-gray-100">
              0) Host Machine, OS, filesystem, and network configuration
            </h2>
            <p class="text-sm leading-6 text-gray-600 dark:text-gray-400">
              The site is hosted on a miniserver at my residence behind a 1Gbps
              wired Xfinity connection. The{" "}
              <a
                href="https://store.minisforum.com/products/minisforum-um690l-slim"
                class="text-sky-700 dark:text-sky-300 hover:underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                host
              </a>{" "}
              has an eight-core Ryzen 9 mobile processor running at up to
              4.9GHz, 32GB of 6400MT/s RAM, and a 1TB NVMe SSD for storage. At
              ~$400, it's a steal compared to having to pay AWS the equivalent
              amount over a year or so for significantly more anemic hardware,
              and reflects my enthusiasm for self-hosting. It runs the
              integrated backend-frontend server, the postgreSQL database, and a
              Minecraft server. If you'd like to play, shoot me an email.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-gray-900 dark:text-gray-100">
              0-1) ArchLinux
            </h3>
            <p class="mt-2 text-sm leading-6 text-gray-600 dark:text-gray-400">
              Not exactly the OS of choice for an enterprise server; if I were
              working somewhere, I would have just done Debian Stable with an
              ext4 filesystem with any old database engine. However, I
              personally I really enjoy tinkering and trying out the latest
              builds of everything, as well as doing CPU architecture optimized
              builds and installs for packages, as provided by the wonderful{" "}
              <a
                href="https://somegit.dev/ALHP/ALHP.GO"
                class="text-sky-700 dark:text-sky-300 hover:underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                ALHP
              </a>{" "}
              project. I would like to try hosting something on Gentoo
              sometimes, but I suspect the builds <em>will</em> take forever.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-gray-900 dark:text-gray-100">
              0-2) Using btrfs on a database-BE/FE-Minecraft host
            </h3>
            <p class="mt-2 text-sm leading-6 text-gray-600 dark:text-gray-400">
              <a
                href="https://en.wikipedia.org/wiki/Btrfs"
                class="text-sky-700 dark:text-sky-300 hover:underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                btrfs
              </a>{" "}
              is a modern filesystem with snapshotting, CoW, compression, all
              the fancy tricks. However, it's not the best choice for a database
              engine host due to extensive fragmentation caused by CoW; I've
              mitigated this by excluding the postgreSQL and Minecraft data
              directories from CoW.{" "}
              <a
                href="https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison"
                class="text-sky-700 dark:text-sky-300 hover:underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                A postgreSQL filesystem benchmark
              </a>{" "}
              does reveal that btrfs in its default state isn't a great
              performer; however, I do suspect that the CoW disabling might make
              it an equal to ext4 and xfs. It would make for an interesting
              benchmark.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-gray-900 dark:text-gray-100">
              0-3) Internal and external network configuration
            </h3>
            <p class="mt-2 text-sm leading-6 text-gray-600 dark:text-gray-400">
              When signing up for Xfinity's 2Gbps television cable Internet
              service, I hadn't realized that they would be cheap enough to
              provide a modem-router that in fact does <em>not</em> support
              2.5Gbps Ethernet. So 1Gbps it shall have to be, but I can't
              imagine a scenario in which that becomes a problem for my little
              website. Route 53 provides DNS services for my domain.
              <br></br>
              <br></br>
              Internally, there really is no reason to use reverse proxy,
              containerization or distributed service tools; it's just a plain
              old postgreSQL engine running on the OS, a Rust binary acting as
              both the front-end and API server, connected to the DB via a UNIX
              socket, which actually is way, way faster than{" "}
              <a
                href="https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/"
                class="text-sky-700 dark:text-sky-300 hover:underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                just using localhost, as detailed here.
              </a>{" "}
              TPS can near-double, and not going through the redundant network
              stack also cuts latency in half; I've observed latencies as low as
              150 microseconds between the DB and the server. Using localhost
              typically got me around 600 microseconds. Not a scenario relevant
              to cloud enterprise service configurations, but it's neat. It's
              old-school. It's faster! PostgreSQL 18, which came out in November
              2025, also introduced async I/O in the form of{" "}
              <a
                href="https://pganalyze.com/blog/postgres-18-async-io"
                class="text-sky-700 dark:text-sky-300 hover:underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                io_uring support
              </a>{" "}
              and it has been enabled. Accelerates reads quite a bit.
            </p>
          </section>

          {/* 1) PostgreSQL */}
          <section class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-950/40 p-5">
            <h2 class="text-lg font-bold mb-2 text-gray-900 dark:text-gray-100">
              1) Data
            </h2>
            <h3 class="mt-5 text-sm font-semibold text-gray-900 dark:text-gray-100">
              1-1) PostgreSQL 18
            </h3>
            <p class="mt-2 text-sm leading-6 text-gray-600 dark:text-gray-400">
              Throughout my startup and corporate career in Korea, I've noticed
              a culture that considers MySQL or MariaDB the only worthwhile
              RDBMS and I've yet to figure out why. Some of my older bosses have
              told me that PostgreSQL used to be not considered a serious option
              at all, which is bizarre given that in the last decade or so, it's
              been making heavy strides in performance, extensibility,
              datatypes, and tooling and has arguably surpassed MySQL in so many
              ways, especially in terms of UUID datatype support and binary
              encoding of JSON data.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-gray-900 dark:text-gray-100">
              1-2) Schema highlights (blog + auth)
            </h3>
            <p class="mt-2 text-sm leading-6 text-gray-600 dark:text-gray-400">
              TODO
            </p>

            <h3 class="mt-5 text-sm font-semibold text-gray-900 dark:text-gray-100">
              1-3) UUIDv7
            </h3>
            <p class="mt-2 text-sm leading-6 text-gray-600 dark:text-gray-400">
              TODO
            </p>

            <h3 class="mt-5 text-sm font-semibold text-gray-900 dark:text-gray-100">
              1-4) Diagram (request + data path)
            </h3>
            <p class="mt-2 text-sm leading-6 text-gray-600 dark:text-gray-400">
              TODO
            </p>
          </section>

          {/* 2) Backend */}
          <section class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-950/40 p-5">
            <h2 class="text-lg font-bold mb-2 text-gray-900 dark:text-gray-100">
              2) Backend (Rust)
            </h2>
            <p class="text-sm leading-6 text-gray-600 dark:text-gray-400">
              TODO
            </p>
          </section>

          {/* 3) Frontend */}
          <section class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-950/40 p-5">
            <h2 class="text-lg font-bold mb-2 text-gray-900 dark:text-gray-100">
              3) Frontend
            </h2>
            <p class="text-sm leading-6 text-gray-600 dark:text-gray-400">
              TODO
            </p>
          </section>

          {/* 4) Network & HTTPS */}
          <section class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-950/40 p-5">
            <h2 class="text-lg font-bold mb-2 text-gray-900 dark:text-gray-100">
              4) HTTPS, routing, and safety rails
            </h2>
            <p class="text-sm leading-6 text-gray-600 dark:text-gray-400">
              TODO
            </p>
          </section>
        </section>
      </section>
    </div>
  );
}
