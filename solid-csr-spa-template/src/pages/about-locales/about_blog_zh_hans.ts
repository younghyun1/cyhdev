import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "2026年11月",
  chrome: {
    overview: "概览",
    updated: "最后更新",
    onThisPage: "本页内容",
    sectionNavigation: "技术栈页面",
    previous: "上一页",
    next: "下一页",
    readingTime: "阅读约需 {minutes} 分钟",
  },
  pages: {
    overview: {
      eyebrow: "关于本站",
      title: "博客技术栈",
      lede: "一个 Rust 二进制程序、一个 PostgreSQL 数据库、一台小型机器。这些页面介绍本站如何构建、托管、保障安全和进行测量，以及为何采用这样的形态。",
      sections: [
        {
          id: "at-a-glance",
          heading: "要点速览",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53,889 req/s",
                  label: "在四个服务器核心上启用 PGO 和 BOLT 后，混合公开读取吞吐量的中位数",
                },
                {
                  value: "~150 µs",
                  label: "通过 UNIX socket 观测到的数据库往返时间；通过 localhost TCP 则为 ~600 µs",
                },
                {
                  value: "130 KiB",
                  count: 130,
                  suffix: " KiB",
                  label: "初始前端资源的 gzip 预算，在构建打包时强制执行",
                },
                {
                  value: "8",
                  count: 8,
                  label: "界面语言，每种语言仅在选择时下载",
                },
                {
                  value: "0.15 ms",
                  label: "启用 `TCP_NODELAY` 后 HTTP/2 瓦片请求完成时间的中位数，此前为 41 ms",
                },
                {
                  value: "1",
                  count: 1,
                  label: "同时提供 TLS、API、WebSockets、WebRTC 媒体和内嵌前端的进程",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "请求路径",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "浏览器",
                  detail: "基于 TLS 的 HTTP/2 或 HTTP/1.1。前端资源通过带指纹的不可变 URL 缓存。",
                },
                {
                  label: "连接准入",
                  detail: "TCP 连接在 TLS 之前进行准入：总计最多 4,096 个，每个地址或 IPv6 /64 最多 64 个。",
                },
                {
                  label: "rustls",
                  detail: "TLS 在 Rust 进程内终止，每个 socket 都启用 `TCP_NODELAY`。",
                },
                {
                  label: "Axum",
                  detail: "依次经过安全响应头、会话、速率限制、压缩和请求体大小限制，然后交给功能处理器。",
                },
                {
                  label: "缓存 · Diesel",
                  detail: "有界内存缓存应对热点读取，其余请求通过异步 Diesel 处理。",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "位于同一主机，通过 UNIX socket 连接，并使用 io_uring 异步 I/O。",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "章节",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "基础设施",
                  href: "/about-blog/infrastructure",
                  meta: "主机 · Gentoo · 网络",
                  text: "机器、操作系统与文件系统、网络，以及为什么没有反向代理。",
                },
                {
                  title: "后端",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "服务技术栈、数据模型、运行时限制、安全响应头和 API 契约。",
                },
                {
                  title: "前端",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "应用外壳、加载预算、本地化、设计系统和动效。",
                },
                {
                  title: "子系统",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "使用进程内 SFU 的实时聊天、Minecraft 探索器、Geo-IP 查询和 WebAssembly。",
                },
                {
                  title: "性能",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "优化构建、测错了对象的基准测试，以及修正后的数据。",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "如何检查变更",
          blocks: [
            {
              kind: "list",
              items: [
                "以针对原生和 WebAssembly 目标的 Clippy 检查作为实现阶段的关卡。",
                "验证工作流涵盖格式检查、原生单元测试、OpenAPI 契约漂移，以及前端的类型检查、lint、单元测试和构建。",
                "在 Chromium 和 WebKit 上运行 Playwright，包括 axe 无障碍检查，以及最窄至 320 CSS 像素的移动端重排检查。",
                "针对一次性 PostgreSQL 实例运行集成测试和迁移回滚测试，并在本地运行 gitleaks 密钥扫描。",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "技术栈",
      title: "基础设施",
      lede: "家中的一台小型机器、一个基于源码的 Linux 发行版，并且没有任何流量用不到的层。",
      sections: [
        {
          id: "host",
          heading: "主机",
          blocks: [
            {
              kind: "paragraph",
              text: "本站运行在我住所的一台[迷你服务器](https://store.minisforum.com/products/minisforum-um690l-slim)上：配备最高可加速至 4.9 GHz 的八核 Ryzen 9 移动处理器、32 GB 内存和 1 TB NVMe SSD。约 400 美元的价格，比租用一年性能明显更弱的云硬件还便宜，这也体现了我对自托管的热情。",
            },
            {
              kind: "paragraph",
              text: "它运行前后端合一的服务器、PostgreSQL 和一台 Minecraft 服务器。想来玩的话，请给我发邮件。",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo 与 btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) 并不是服务器的常规选择；在工作中，我会选择 ext4 上的 Debian stable。在这里，我喜欢针对具体处理器进行优化，亲自编译软件包。",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) 提供快照、压缩和写时复制，但写时复制会导致数据库文件严重碎片化，因此 PostgreSQL 和 Minecraft 的数据目录不启用写时复制。一项[文件系统基准测试](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison)显示，默认配置的 btrfs 落后于 ext4 和 XFS；我猜关闭写时复制能弥补大部分差距，这会是一项有意思的测量。",
            },
          ],
        },
        {
          id: "network",
          heading: "网络",
          blocks: [
            {
              kind: "paragraph",
              text: "流量通过 1 Gbps 有线 Xfinity 连接进入；运营商的调制解调器不支持 2.5 Gb 以太网，不过本站并不会受到影响。DNS 由 Route 53 托管。",
            },
            {
              kind: "paragraph",
              text: "机器内部没有反向代理、容器运行时或服务网格。PostgreSQL 直接运行在操作系统上，一个 Rust 二进制程序同时提供前端和 API。普通 HTTP 请求也由同一进程重定向到 HTTPS。",
            },
          ],
        },
        {
          id: "sockets",
          heading: "UNIX socket",
          blocks: [
            {
              kind: "paragraph",
              text: "服务器通过 UNIX socket 而非 localhost TCP 连接 PostgreSQL，完全绕过网络协议栈。[基准测试](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/)显示事务处理速率几乎翻倍；在这里，我观测到的往返时间低至 **150 µs**，而通过 localhost 约为 **600 µs**。这种做法很老派，与大多数云端部署无关，但它更快。",
            },
            {
              kind: "paragraph",
              text: "2025 年 9 月发布的 PostgreSQL 18 新增了[异步 I/O](https://pganalyze.com/blog/postgres-18-async-io)，本站已启用 io_uring，读取速度明显提升。",
            },
          ],
        },
        {
          id: "deploy",
          heading: "构建与部署",
          blocks: [
            {
              kind: "list",
              items: [
                "前端构建产物连同预压缩的 zstd 和 gzip 副本一起嵌入后端二进制文件，因此生产环境中不运行任何 Node.js 进程。",
                "优化构建可以通过配置文件引导优化进行训练，并用 BOLT 进行链接后优化；详见[性能](/about-blog/performance)。",
                "凭据、证书和运行时数据都不进入 Git，变更离开本机之前会先经过 gitleaks 扫描。",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "技术栈",
      title: "后端",
      lede: "基于 Axum 和 Tokio 的 Rust，通过异步 Diesel 访问 PostgreSQL，并对客户端可占用的一切资源设置明确上限。",
      sections: [
        {
          id: "stack",
          heading: "技术栈",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "运行时", items: ["Axum", "Tokio", "hyper", "axum-server", "rustls", "mimalloc"] },
                { label: "数据", items: ["PostgreSQL 18", "Diesel", "diesel-async", "tantivy", "bitcode", "zstd"] },
                { label: "集成", items: ["openidconnect", "lettre", "Amazon S3", "webrtc-rs", "image + AVIF"] },
                { label: "API 契约", items: ["utoipa", "OpenAPI", "Swagger UI", "Generated TypeScript"] },
              ],
            },
            {
              kind: "paragraph",
              text: "服务是一个共享状态的单一进程，包括内存会话、启动时缓存同步和定时维护任务。全局分配器为 mimalloc；既然机器拥有真正的多核，分配器行为就值得认真对待。",
            },
          ],
        },
        {
          id: "data",
          heading: "数据模型",
          blocks: [
            {
              kind: "paragraph",
              text: "在我所见的韩国初创公司和企业工作中，MySQL 和 MariaDB 被视为唯一正经的关系型数据库。但在我关心的方面，从原生 UUID 到二进制 JSON，PostgreSQL 早已超越它们。",
            },
            {
              kind: "list",
              items: [
                "**文章**包含 slug、摘要、JSONB 元数据、发布状态、反规范化计数器和标签映射。评论通过可为空的父引用形成线程，文章投票和评论投票分别存放在独立的表中。",
                "**账号**保存国家和语言，配有角色和权限表，头像则按版本存放在独立的表中，而不是放在频繁访问的用户行上。",
                "**主键**采用按时间排序的 UUIDv7：不像自增 ID 那样可被枚举，对 B-tree 局部性也比随机的 UUIDv4 友好得多。",
                "**搜索**使用持久化到磁盘的 tantivy 索引，覆盖文章标题和标签，并与仍作为权威数据源的 PostgreSQL 保持一致。",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "运行时限制",
          blocks: [
            {
              kind: "list",
              items: [
                "连接在 TLS 之前通过共享限制器准入：最多 4,096 个打开的连接，每个 IPv4 地址或 IPv6 /64 最多 64 个。被拒绝的 socket 不经握手直接关闭。",
                "HTTP/1 请求头必须在 20 秒内到达；HTTP/2 连接每 30 秒 ping 一次，允许 128 个并发流。",
                "每个缓存都有固定上限，并有可回源读取的权威数据源；没有任何缓存会随客户端控制的键增长。例如，WebAssembly 包缓存最多占用 256 MiB。",
                "图像编码等阻塞工作在有界工作线程池中运行，而不是占用 Tokio 工作线程。",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "安全响应头",
          blocks: [
            {
              kind: "paragraph",
              text: "Content-Security-Policy 在启动时构建一次。内联脚本只能通过根据编译进二进制文件的 `index.html` 计算出的 SHA-256 哈希放行，因此首次绘制前运行的主题脚本不会与策略脱节。主源从不实例化 WebAssembly：演示和 EU5 应用运行在各自策略下的沙箱框架中。",
            },
            {
              kind: "paragraph",
              text: "会话 Cookie 设置了 `Secure` 和 `HttpOnly`，超级用户操作在服务器端检查角色，账号可以关联 OpenID Connect 身份，生产环境中的 Swagger UI 需要身份验证。",
            },
          ],
        },
        {
          id: "contracts",
          heading: "API 契约与静态资源",
          blocks: [
            {
              kind: "list",
              items: [
                "处理器带有 OpenAPI 注解，前端的 TypeScript 客户端由此生成；生成文件一旦漂移，CI 就会失败。",
                "Vite 的清单与静态资源一同嵌入。只有清单中列出且带指纹的文件才会获得一年期的 `immutable` 缓存策略；HTML 始终重新验证。",
                "不存在的资源路径返回带 `no-store` 的 404，而不是应用外壳，因此过期页面无法以脚本 URL 缓存 HTML。",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "技术栈",
      title: "前端",
      lede: "客户端渲染的 SolidJS 2 应用，编译为静态资源并嵌入服务器二进制文件。",
      sections: [
        {
          id: "stack",
          heading: "技术栈",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "应用", items: ["SolidJS 2", "Solid Router 2", "TypeScript", "Vite 8"] },
                { label: "样式", items: ["Tailwind CSS 4", "IBM Plex Sans", "IBM Plex Mono", "OKLCH"] },
                { label: "组件", items: ["Toast UI Editor", "Leaflet", "Chart.js", "highlight.js", "DOMPurify"] },
                { label: "测试", items: ["Vitest", "Playwright", "axe-core"] },
              ],
            },
            {
              kind: "paragraph",
              text: "Solid 的细粒度信号只更新依赖于已变更值的 DOM 节点，其 2.0 候选版本允许异步读取挂起到最近的加载边界。路由器对每个页面都进行懒加载，生产环境中没有 JavaScript 服务器：静态文件由 Rust 二进制程序直接提供。",
            },
          ],
        },
        {
          id: "budget",
          heading: "加载预算",
          blocks: [
            {
              kind: "list",
              items: [
                "一个 Vite 插件从入口代码块开始追踪静态导入，如果初始脚本和样式的 gzip 体积超过 **130 KiB**，构建即失败。",
                "初始代码块只包含英文界面文本。其他语言以及各语言的关于页面都是单独的代码块，在选择时才加载。",
                "限速条件下的移动端性能测试目标为：LCP 不超过 2.5 s，交互不超过 200 ms，CLS 不超过 0.1。",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "本地化",
          blocks: [
            {
              kind: "paragraph",
              text: "界面支持英语、韩语、法语、西班牙语、简体中文、繁体中文、日语和德语。约 650 个键的目录会与后端注册表进行校验，占位符在翻译后必须保留。这些关于页面是带类型的数据模块，因此译文不可能悄无声息地遗漏英文版中存在的章节。",
            },
          ],
        },
        {
          id: "design",
          heading: "设计系统",
          blocks: [
            {
              kind: "paragraph",
              text: "整体风格温暖而富有技术感：米色纸张或纯黑底色、细线边框、技术内容使用等宽字体，并且一次只使用一种强调色。颜色均为语义化 CSS 变量，因此组件无需单独的深色模式变体。强调色预设以 OKLCH 定义，可在不同色相间保持一致的感知亮度；浅色或深色模式和强调色都会在首次绘制前应用。",
            },
          ],
        },
        {
          id: "motion",
          heading: "动效",
          blocks: [
            {
              kind: "list",
              items: [
                "章节显现效果和阅读进度条使用 CSS 滚动驱动动画，在受支持的浏览器中脱离主线程运行。",
                "统计数字通过注册的 `@property` 整数递增显示，最终数值保留在标记中，供辅助技术读取。",
                "主题切换通过 View Transitions API 从切换按钮处向外展开。",
                "首页的等高线背景是一个小型 WebGL 着色器，在页面空闲后加载，仅在性能足够的桌面浏览器上运行，限制帧率，并在离开屏幕时暂停。",
                "当系统要求减少动态效果时，所有装饰性效果都会跳过。",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "技术栈",
      title: "子系统",
      lede: "本站中超出博客严格所需、更有意思的部分。",
      sections: [
        {
          id: "live-chat",
          heading: "实时聊天与通话",
          blocks: [
            {
              kind: "paragraph",
              text: "聊天基于 WebSocket，使用紧凑的二进制编解码、内容审核和有界历史记录。语音和视频通话经过一个基于 webrtc-rs、运行在 Axum 进程内的选择性转发单元（SFU）：每个浏览器只维持一个对等连接，SFU 通过按发布划分的有界通道转发已编码媒体而不进行转码，并限制参与人数。[打开实时聊天](/live-chat)。",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Minecraft 地形探索器",
          blocks: [
            {
              kind: "paragraph",
              text: "一个 Paper 插件开放仅限同一用户访问的 UNIX socket，用于有界的世界查询，每次只处理一个操作，并严格限制帧大小。[探索器](/minecraft)使用自己的 Leaflet 客户端绘制 squaremap 瓦片，而不运行所生成地图的 JavaScript。",
            },
            {
              kind: "paragraph",
              text: "生物群系叠加层使用 **CYBM**（一种以 gzip 压缩传输的紧凑二进制瓦片格式），或在私有 `cyBM` 数据块中携带精确生物群系标识的 PNG，因此悬停时从不需要根据像素颜色猜测。未探索地形的种子预测来自一个基于 Pumpkin 构建的独立世界生成工作进程。",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Geo-IP 查询",
          blocks: [
            {
              kind: "paragraph",
              text: "[访客地图](/visitor-board)和 [IP 查询](/geo-ip-db)无需调用外部 API 即可定位地址。一个预处理工具将 IP2Location LITE CSV 文件转换为以范围起始地址为键的 `BTreeMap`，用 bitcode 编码，再以 zstd 22 级压缩。",
            },
            {
              kind: "table",
              columns: ["数据集", "CSV", "打包文件"],
              numericColumns: [1, 2],
              rows: [
                ["IPv4", "312.5 MB", "24.9 MB"],
                ["IPv6", "794.2 MB", "42.4 MB"],
              ],
            },
            {
              kind: "paragraph",
              text: "启动时，服务器每次只解码一个打包文件以控制峰值内存，并对重复出现的地名做字符串驻留。查询时先找到起始地址不大于目标地址的最后一个范围，再检查其结束边界。",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "上传的[演示包](/projects)会先暂存到磁盘，通过魔数识别格式，解压后限制在 50 MiB 以内，并从有界缓存提供给沙箱框架。工作区还会构建 Rust 演示，包括打砖块游戏和光线追踪器。",
            },
            {
              kind: "paragraph",
              text: "[EU5 Location DB](/eu5-locations-db) 是一个编译为 WebAssembly 的 Slint 应用。页面通过一个小型消息协议传递浅色或深色模式，因此内嵌应用的配色会随网站切换。",
            },
          ],
        },
        {
          id: "photography",
          heading: "摄影处理流程",
          blocks: [
            {
              kind: "paragraph",
              text: "上传的图片在固定的像素上限内解码，并以最慢的预设编码为 AVIF，用处理时间换取更小的文件。EXIF 数据中的位置可在发布前于地图上确认，同时运行的处理任务不超过两个。",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "技术栈",
      title: "性能",
      lede: "对后端进行配置文件引导优化和 BOLT 优化，并用不会成为瓶颈的负载生成器进行测量。",
      sections: [
        {
          id: "approach",
          heading: "方法",
          blocks: [
            {
              kind: "paragraph",
              text: "优化构建基于具有代表性的工作负载进行训练。插桩后的二进制文件针对一个一次性测试夹具记录性能剖析数据，该夹具提供模拟的 PostgreSQL、SMTP、S3、OpenID Connect 和 Minecraft 端点。LLVM 使用这些剖析数据重新构建服务器（PGO），随后 BOLT 根据采样的执行情况重排最终二进制文件的代码布局。所有变体都使用 LTO、帧指针和重新构建的标准库。",
            },
          ],
        },
        {
          id: "correction",
          heading: "测量了自己的基准测试",
          blocks: [
            {
              kind: "paragraph",
              text: "第一轮测试由 Playwright 施加负载，结果似乎显示性能退化。但这些退化并不真实：未限定作用域的 Playwright API 调用会搜索之前的测试步骤来寻找父级，因此客户端随着运行推进而越来越慢。即使面对一个极简的 HTTP 服务器，仅客户端本身在九个样本中就从 2,684 降到 1,314 requests/s。",
            },
            {
              kind: "paragraph",
              text: "负载生成改用 oha。Playwright 仍会检查每条路由的响应，但在计时窗口之外进行。",
            },
          ],
        },
        {
          id: "setup",
          heading: "测试环境",
          blocks: [
            {
              kind: "list",
              items: [
                "服务器、测试夹具提供程序和 PostgreSQL 绑定到 AMD Ryzen 7 9800X3D 的四个核心；负载客户端绑定到另外四个核心。",
                "通过池化的 HTTP/1.1 使用 HTTPS，共 32 个连接，完整读取响应，超时期限为两秒。",
                "轮换顺序进行三轮，每个变体五个样本，每个样本前先发送 10,000 个预热请求，并从全新的数据库快照开始。",
                "计时请求共 27,000,000 个，全部返回 HTTP 200。",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "结果",
          blocks: [
            {
              kind: "bars",
              caption: "每个变体 15 个样本的每秒请求数中位数。",
              groups: [
                {
                  label: "十条路由的公开读取",
                  items: [
                    { label: "基线", value: 44874.12, display: "44,874" },
                    { label: "PGO", value: 52925.46, display: "52,925" },
                    { label: "PGO + BOLT", value: 53888.61, display: "53,889", emphasis: true },
                  ],
                },
                {
                  label: "小型健康检查 JSON",
                  items: [
                    { label: "基线", value: 272837.41, display: "272,837" },
                    { label: "PGO", value: 308685.26, display: "308,685", emphasis: true },
                    { label: "PGO + BOLT", value: 306276.64, display: "306,277" },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "十条路由混合负载的尾部延迟。",
              columns: ["变体", "p95 (ms)", "p99 (ms)"],
              numericColumns: [1, 2],
              rows: [
                ["基线", "2.380", "2.967"],
                ["PGO", "2.020", "2.524"],
                ["PGO + BOLT", "1.987", "2.483"],
              ],
            },
            {
              kind: "paragraph",
              text: "PGO 使混合吞吐量提升 **17.9%**，PGO 加 BOLT 提升 **20.1%**，两种优化构建在每一轮轮换中都胜出。在混合读取上，BOLT 比 PGO 多提升 1.8%，而在极简的健康检查路由上慢 0.8%。这些是四核结果；生产环境的容量还取决于网络、数据库工作集和真实流量。",
            },
          ],
        },
        {
          id: "transport",
          heading: "小修复也很重要",
          blocks: [
            {
              kind: "paragraph",
              text: "axum-server 默认的 Rustls 接收器没有设置 `TCP_NODELAY`，因此拆分为头部帧和数据帧的 HTTP/2 响应可能需要等待延迟确认。在每个已接受的 socket 上启用它后，通过复用连接传输的 59,571 字节瓦片响应变化如下。",
            },
            {
              kind: "table",
              columns: ["TCP_NODELAY", "协议", "首字节时间中位数", "完成时间中位数"],
              numericColumns: [2, 3],
              rows: [
                ["关闭", "HTTP/2", "0.290 ms", "40.981 ms"],
                ["开启", "HTTP/2", "0.113 ms", "0.152 ms"],
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
