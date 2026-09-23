import { RepositorySourceMap } from "../../components/RepositorySourceMap";
import { pageStyles } from "../../styles/pageStyles";

export default function AboutBlog() {
  return (
    <main lang="zh-Hans" class={pageStyles.page}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">
            博客技术栈
          </h1>
          <p class="text-xs text-ink-muted">
            最后更新：2026-08-30
            <br />
          </p>
        </div>

        <section class="space-y-8">
          {/* 0) On-Prem */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              0）主机、操作系统、文件系统与网络配置
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              本站托管在我住所的一台迷你服务器上，通过 1Gbps 有线 Xfinity 网络接入。
              {" "}
              <a
                href="https://store.minisforum.com/products/minisforum-um690l-slim"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                主机
              </a>{" "}
              配备最高运行频率为 4.9GHz 的八核 Ryzen 9 移动处理器、32GB 6400MT/s 内存和 1TB NVMe SSD。售价约 400 美元，相比一年左右支付同等金额给 AWS 却只能获得性能弱得多的硬件，实在划算，也体现了我对自托管的热情。它运行集成的前后端服务器、postgreSQL 数据库和 Minecraft 服务器。想来玩的话，给我发邮件。

            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-1）Gentoo
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              它并非企业服务器的常见选择；如果在公司工作，我大概会采用 Debian Stable、ext4 文件系统和普通数据库引擎。不过我个人很喜欢折腾并亲自编译软件，也喜欢为 CPU 架构优化软件包的构建和安装，这些都得益于优秀的{" "}
              <a
                href="https://www.gentoo.org/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                Gentoo
              </a>{" "}
              项目。
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-2）在数据库、前后端和 Minecraft 共用的主机上使用 btrfs
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
              是一种现代文件系统，支持快照、写时复制（CoW）、压缩等功能。不过，CoW 造成的大量碎片使它并非数据库服务器的理想选择；我通过将 PostgreSQL 和 Minecraft 数据目录排除在 CoW 之外来缓解这一问题。{" "}
              <a
                href="https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                一项 PostgreSQL 文件系统基准测试
              </a>{" "}
              显示 btrfs 默认状态下的性能并不理想；不过我猜关闭 CoW 后，它可能达到 ext4 和 XFS 的水平。这会是个有意思的基准测试。
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-3）内外部网络配置
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              申请 Xfinity 的 2Gbps 有线电视互联网服务时，我没想到他们会省到提供一台实际上<em>不</em>支持 2.5Gbps 以太网的光猫路由器。因此只能使用 1Gbps，不过我想象不出这会给我的小网站带来什么问题。我的域名由 Route 53 提供 DNS 服务。
              <br />
              <br />
              在内部网络中，确实没有理由使用反向代理、容器化或分布式服务工具；这里只是在操作系统上运行普通的 PostgreSQL 引擎，由一个 Rust 二进制程序同时担任前端和 API 服务器，并通过 UNIX socket 连接数据库，速度实际上远远快于{" "}
              <a
                href="https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                仅使用 localhost，详情见此文。
              </a>{" "}
              TPS 几乎可以翻倍，而且省去冗余网络协议栈也能让延迟减半；我观测到数据库与服务器之间的延迟低至 150 微秒。使用 localhost 通常约为 600 微秒。这对云端企业服务配置并不适用，但很有意思。老派，却更快！PostgreSQL 18 于 2025 年 11 月发布，还以{" "}
              <a
                href="https://pganalyze.com/blog/postgres-18-async-io"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                io_uring 支持
              </a>{" "}
              的形式引入了异步 I/O，目前已启用，显著加快了读取速度。
            </p>
          </section>

          {/* 1) PostgreSQL */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              1）数据
            </h2>
            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-1）PostgreSQL 18
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              在韩国创业公司和企业工作的这些年，我注意到一种观念：MySQL 或 MariaDB 才是值得使用的关系型数据库，而我一直不明白原因。一些资深上司告诉我，过去 PostgreSQL 根本不被视为可靠的选择。这很奇怪，因为过去十多年它在性能、可扩展性、数据类型和工具方面进步显著，许多方面甚至可以说已超过 MySQL，尤其是 UUID 数据类型支持和 JSON 数据的二进制编码。
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-2）架构要点（博客与身份验证）
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              数据库架构刻意保持朴素，但这是优点：包含用户、内存会话、邮箱验证令牌、密码重置令牌、文章、评论、投票表、标签、头像以及少量地理和国际化支持表。博客当然不只是一个 <code>posts</code> 表再祈祷一切顺利。文章还保存 slug、摘要、JSONB 元数据、发布状态、反规范化计数器和标签映射。评论通过可空的父评论 ID 形成线程。文章和评论投票分别存放于专用表中，让查询路径更简单，也避免后来出现荒唐的条件逻辑。
              <br />
              <br />
              身份验证设计同样注重实用：用户记录保存国家和语言，让网站不至于只收下邮箱就把你忘掉。设置角色和权限表，是因为我不想让授权设计陷入死角。头像版本单独存放在自己的表中，而不是塞进用户行；这样更容易处理替换，也避免给频繁访问的用户记录添加无关负担。
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-3）UUIDv7
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              几乎所有面向用户的对象都以 UUID 为标识，这不是偶然。我不想暴露连续 ID，让任何人都能推测行数或像 2009 年那样枚举资源。按时间排序的 UUID 对索引也比完全随机的 UUIDv4 更友好，在写入量确实存在时尤其有用。换句话说，它全局唯一、难以猜测，也更有利于数据局部性。插入模式更平稳，B-tree 抖动更少，数据库也不必花太多时间做可避免的整理工作。
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-4）示意图（请求与数据路径）
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              这是一张非常“高大上”的企业架构图。因为我还没费心画方框，所以先用文字呈现：
              <br />
              <br />
              浏览器请求 -&gt; Axum 路由器 -&gt; 中间件链（日志、身份验证/会话查询、速率限制、请求体大小检查）-&gt; 处理器 -&gt; Diesel 异步查询或内存缓存查询 -&gt; 通过 UNIX socket 访问 PostgreSQL -&gt; DTO 响应 -&gt; 压缩后的 HTTPS 响应返回浏览器。
              <br />
              <br />
              静态资源的路径更短。构建后的 SolidJS 应用直接嵌入 Rust 二进制文件；如果浏览器支持，则通过内容协商提供 zstd/gzip 压缩。生产环境无需常驻 Node 进程，也无需单独的静态文件主机；从“收到请求”到“发出字节”之间没有额外跳转。
            </p>
          </section>

          {/* 2) Backend */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              2）后端（Rust）
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              后端是基于 Axum、Tokio、Diesel 和 PostgreSQL 构建的 Rust 服务，并使用 mimalloc 作为全局分配器。既然要在拥有真正多核的机器上自托管，分配器行为也值得认真对待。服务通过 rustls 直接终止 TLS，提供嵌入式 SPA，公开身份验证、博客、摄影和国际化的 JSON API，并通过 WebSockets 向服务器仪表板推送主机状态。路由器包含请求压缩、目前较宽松的 CORS、相当宽裕的速率限制，以及在生产环境中受身份验证保护的 Swagger UI。
              <br />
              <br />
              应用侧采用单进程设计，共享服务器状态对象、在内存中管理会话、启动时同步缓存，并安排定时维护任务。身份验证使用安全的 HTTP-only Cookie、邮箱验证和密码重置令牌；超级用户操作会明确检查角色。更有意思的是延迟表现：数据库流量走 UNIX socket，连接池依据物理核心数调整大小，博客列表读取大多来自缓存，响应组装则批量完成数据补充，避免退化为大量细碎查询。这样带来的速度正是我看重的：跳转更少、复制更少、往返更少，也不用等抽象层自我庆祝完毕。
            </p>
          </section>

          {/* 3）前端 */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              3）前端
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              前端是使用 Vite 和 TypeScript 构建的客户端 SolidJS 单页应用。路由按需加载，状态管理保持简单；部署时整个应用会编译为静态资源并打包进 Rust 二进制文件。因此生产环境不需要维护 JavaScript 服务器进程，首次访问由 Rust 服务器尽可能高效地提供静态文件。
              <br />
              <br />
              博客界面兼顾实用，也体现了我不愿使用玩具编辑器的坚持。Markdown 编辑由 Toast UI Editor 处理；粘贴或上传的图片会直接通过摄影上传 API，因此撰写文章不会变得痛苦。搜索支持标题和标签，页面可通过查询参数导航；身份验证状态在客户端跟踪，但由服务器执行检查；应用还会读取响应头以显示服务器构建信息。样式基于 Tailwind，并采用共享页面样式系统。Solid 运行时精简、很少反复改动 DOM，正合我意：界面层最重要的工作就是别碍事。
            </p>
          </section>

          {/* 4) Network & HTTPS */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              4）HTTPS、路由与安全防护
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              HTTPS 由 Rust 服务器通过 rustls 直接处理。普通
              HTTP 请求会重定向到 HTTPS，Cookie 标记为 secure 和 HTTP-only，生产环境的 Cookie 域名限定为本站域名，不会随意扩散。浏览器支持时，静态资源使用 zstd 或 gzip 传输；SPA 回退路由让深层链接无需额外的反向代理层即可正常工作。
              <br />
              <br />
              安全措施并不花哨，但都落实到位：上传请求体大小限制、中间件日志、受保护路由的身份验证、敏感端点的超级用户检查、用于减少滥用的速率限制，以及客户端请求使用的 API key。更重要的是，部署形态本身简单易懂：一个二进制文件、一个数据库、一台主机，应用直接处理 TLS，也几乎没有神秘延迟或配置漂移的借口。这种架构也许不时髦，却快速、可观测，而且开销很低。
            </p>
          </section>

          <RepositorySourceMap />
        </section>
      </section>
    </main>
  );
}
