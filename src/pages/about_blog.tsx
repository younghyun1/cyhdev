import { pageStyles } from "../styles/pageStyles";

export default function AboutBlog() {
  return (
    <main class={pageStyles.page}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-slate-700 dark:text-slate-100`}
      >
        <div class="border-l-4 border-slate-300 dark:border-slate-700 pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">
            Blog Tech Stack
          </h1>
          <p class="text-xs text-slate-500 dark:text-slate-400">
            Last updated: 2026-03-06
            <br />
          </p>
        </div>

        <section class="space-y-8">
          {/* 0) On-Prem */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-slate-900 dark:text-slate-100">
              0) Host Machine, OS, filesystem, and network configuration
            </h2>
            <p class="text-sm leading-6 text-slate-600 dark:text-slate-400">
              The site is hosted on a miniserver at my residence behind a 1Gbps
              wired Xfinity connection. The{" "}
              <a
                href="https://store.minisforum.com/products/minisforum-um690l-slim"
                class={pageStyles.link}
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

            <h3 class="mt-5 text-sm font-semibold text-slate-900 dark:text-slate-100">
              0-1) Gentoo
            </h3>
            <p class="mt-2 text-sm leading-6 text-slate-600 dark:text-slate-400">
              Not exactly the OS of choice for an enterprise server; if I were
              working somewhere, I would have just done Debian Stable with an
              ext4 filesystem with any old database engine. However, I
              personally I really enjoy tinkering and compiling stuff myself, as
              well as doing CPU architecture optimized builds and installs for
              packages, as enabled by the wonderful{" "}
              <a
                href="https://www.gentoo.org/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                Gentoo
              </a>{" "}
              project.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-slate-900 dark:text-slate-100">
              0-2) Using btrfs on a database-BE/FE-Minecraft host
            </h3>
            <p class="mt-2 text-sm leading-6 text-slate-600 dark:text-slate-400">
              <a
                href="https://en.wikipedia.org/wiki/Btrfs"
                class={pageStyles.link}
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
                class={pageStyles.link}
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

            <h3 class="mt-5 text-sm font-semibold text-slate-900 dark:text-slate-100">
              0-3) Internal and external network configuration
            </h3>
            <p class="mt-2 text-sm leading-6 text-slate-600 dark:text-slate-400">
              When signing up for Xfinity's 2Gbps television cable Internet
              service, I hadn't realized that they would be cheap enough to
              provide a modem-router that in fact does <em>not</em> support
              2.5Gbps Ethernet. So 1Gbps it shall have to be, but I can't
              imagine a scenario in which that becomes a problem for my little
              website. Route 53 provides DNS services for my domain.
              <br />
              <br />
              Internally, there really is no reason to use reverse proxy,
              containerization or distributed service tools; it's just a plain
              old postgreSQL engine running on the OS, a Rust binary acting as
              both the front-end and API server, connected to the DB via a UNIX
              socket, which actually is way, way faster than{" "}
              <a
                href="https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/"
                class={pageStyles.link}
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
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                io_uring support
              </a>{" "}
              and it has been enabled. Accelerates reads quite a bit.
            </p>
          </section>

          {/* 1) PostgreSQL */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-slate-900 dark:text-slate-100">
              1) Data
            </h2>
            <h3 class="mt-5 text-sm font-semibold text-slate-900 dark:text-slate-100">
              1-1) PostgreSQL 18
            </h3>
            <p class="mt-2 text-sm leading-6 text-slate-600 dark:text-slate-400">
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

            <h3 class="mt-5 text-sm font-semibold text-slate-900 dark:text-slate-100">
              1-2) Schema highlights (blog + auth)
            </h3>
            <p class="mt-2 text-sm leading-6 text-slate-600 dark:text-slate-400">
              The schema is intentionally boring in the good way: users,
              sessions-in-memory, email verification tokens, password reset
              tokens, posts, comments, vote tables, tags, profile pictures, and
              a few geography/i18n support tables. The blog itself is not just a
              <code>posts</code> table and a prayer. Posts carry slugs,
              summaries, metadata as JSONB, publish state, denormalized
              counters, and tag mappings. Comments are threaded via a nullable
              parent comment ID. Votes are split into dedicated post and comment
              vote tables, which keeps query paths simpler and avoids some very
              silly conditional logic later on.
              <br />
              <br />
              Auth is similarly practical: the user record stores country and
              language so the site can do more than just ask for an email and
              forget you exist. Role and permission tables are present because I
              dislike painting myself into authorization corners. Profile
              pictures are versioned in their own table rather than smeared onto
              the user row, which makes replacement logic cleaner and avoids
              overloading the hot user record with unrelated concerns.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-slate-900 dark:text-slate-100">
              1-3) UUIDv7
            </h3>
            <p class="mt-2 text-sm leading-6 text-slate-600 dark:text-slate-400">
              Pretty much everything user-facing is keyed by UUIDs, and that was
              not an accident. I do not enjoy exposing sequential IDs that let
              anybody infer row counts or enumerate resources like it's 2009.
              Time-ordered UUIDs also behave much more politely for indexes than
              fully random UUIDv4 values, which is handy when writes are not
              purely theoretical. In other words: globally unique, hard to
              guess, and less hostile to locality. Insert patterns stay saner,
              B-tree churn is lower, and the database spends less time doing
              avoidable housekeeping.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-slate-900 dark:text-slate-100">
              1-4) Diagram (request + data path)
            </h3>
            <p class="mt-2 text-sm leading-6 text-slate-600 dark:text-slate-400">
              Very glamorous enterprise architecture diagram, reproduced here in
              text form because I have not yet bothered to draw boxes:
              <br />
              <br />
              Browser request -&gt; Axum router -&gt; middleware chain (logging,
              auth/session lookup, rate limit, body size checks) -&gt; handler
              -&gt; Diesel async query or in-memory cache lookup -&gt;
              PostgreSQL over a UNIX socket -&gt; DTO response -&gt; compressed
              HTTPS response back to browser.
              <br />
              <br />
              Static assets take an even shorter route. The built SolidJS app is
              embedded directly into the Rust binary and served with content
              negotiation for zstd/gzip if the browser supports it. No Node
              process lingering in production, no separate static file host, and
              no extra hop between "request arrived" and "bytes went out."
            </p>
          </section>

          {/* 2) Backend */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-slate-900 dark:text-slate-100">
              2) Backend (Rust)
            </h2>
            <p class="text-sm leading-6 text-slate-600 dark:text-slate-400">
              The backend is a Rust service built on Axum, Tokio, Diesel, and
              PostgreSQL, with mimalloc as the global allocator because if one
              is going to self-host on a machine with actual cores, one may as
              well be at least a little bit serious about allocator behavior. It
              terminates TLS directly with rustls, serves the embedded SPA,
              exposes JSON APIs for auth/blog/photography/i18n, and also pushes
              host stats over WebSockets for the server dashboard. The router
              includes request compression, permissive CORS for now, fairly
              generous rate limiting, and a Swagger UI that gets gated behind
              auth in production.
              <br />
              <br />
              On the application side, the design leans into a single process
              with a shared server state object, in-memory session management,
              startup cache synchronization, and scheduled maintenance jobs.
              Authentication uses secure HTTP-only cookies, email verification,
              password reset tokens, and explicit role checks for superuser
              operations. The more interesting part is the latency profile:
              database traffic stays on a UNIX socket, the connection pool sizes
              itself against physical core count, blog list reads come largely
              from cache, and the response assembly does the enrichment work in
              batches rather than degenerating into a pile of tiny lookups. That
              keeps the stack fast in the way I actually care about: fewer hops,
              fewer copies, fewer round trips, and less waiting around for
              abstraction to finish congratulating itself.
            </p>
          </section>

          {/* 3) Frontend */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-slate-900 dark:text-slate-100">
              3) Frontend
            </h2>
            <p class="text-sm leading-6 text-slate-600 dark:text-slate-400">
              The frontend is a client-side SolidJS SPA built with Vite and
              TypeScript. Routes are lazy-loaded, state is kept fairly
              straightforward, and the whole thing is compiled into static
              assets that get bundled into the Rust binary at deploy time. That
              means there is no production JavaScript server process to babysit,
              and initial delivery is just the Rust server handing out static
              files as efficiently as it can.
              <br />
              <br />
              The blog UI itself is a mix of practicality and me refusing to use
              toy editors. Markdown authoring is handled with Toast UI Editor,
              and pasted or uploaded images go straight through the photography
              upload API so post composition is not a miserable experience.
              Search supports title queries and tags, pages are navigable via
              query params, auth state is tracked client-side but enforced
              server-side, and the app keeps an eye on response headers so it
              can display server build information. Styling is Tailwind-based
              with a shared page style system. Solid helps here by being lean at
              runtime and very sparing with DOM churn, which is exactly what I
              want from a UI layer whose main job is to stay out of the way.
            </p>
          </section>

          {/* 4) Network & HTTPS */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-slate-900 dark:text-slate-100">
              4) HTTPS, routing, and safety rails
            </h2>
            <p class="text-sm leading-6 text-slate-600 dark:text-slate-400">
              HTTPS is handled directly by the Rust server with rustls. Plain
              HTTP requests get bounced to HTTPS, cookies are marked secure and
              HTTP-only, and the production cookie domain is locked to the site
              domain rather than sprayed around indiscriminately. Static assets
              are served with zstd or gzip when supported, and SPA fallback
              routing means deep links work without involving some separate
              reverse proxy layer.
              <br />
              <br />
              The safety rails are not especially exotic, but they are there:
              request body limits for uploads, middleware logging, auth gates on
              protected routes, superuser checks on sensitive endpoints, rate
              limiting to make abuse less amusing, and API key support for
              client requests. More importantly, the deployment shape itself is
              simple enough to reason about. One binary, one database, one host,
              TLS on the app, and very few excuses for mystery latency or
              configuration drift. It is not fashionable architecture, but it is
              fast, observable, and stubbornly low-overhead.
            </p>
          </section>
        </section>
      </section>
    </main>
  );
}
