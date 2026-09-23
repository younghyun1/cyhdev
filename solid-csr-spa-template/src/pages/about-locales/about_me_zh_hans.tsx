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
    <main lang="zh-Hans" class={`${pageStyles.page} about-page`}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">关于我</h1>
          <p class="text-xs text-ink-muted">
            最后更新：2026年9月
          </p>
        </div>

        <section class="mb-12">
          <h2 class="text-xl font-bold mb-4 border-b border-line pb-2">
            1. 简介
          </h2>

          <div class="mb-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div>
              <h3 class="text-lg font-semibold">
                Young Hyun Chi / 지영현 / 池營賢 / 池营贤
              </h3>

              <p class="text-ink-muted">
                后端与基础设施工程师 · Rust / 平台可靠性 / 安全
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
                  alt="个人肖像"
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
            <h3 class="font-bold mb-2">联系方式</h3>
            <ul class="list-none space-y-2 text-sm">
              <li>
                <span class="w-20 inline-block font-medium text-ink-muted">
                  邮箱：
                </span>
                <a href="mailto:younghyun1@gmail.com" class={pageStyles.link}>
                  younghyun1@gmail.com
                </a>
              </li>
              <li>
                <span class="w-20 inline-block font-medium text-ink-muted">
                  GitHub：
                </span>
                <a href="https://github.com/younghyun1" class={pageStyles.link}>
                  github.com/younghyun1
                </a>
              </li>
              <li>
                <span class="w-20 inline-block font-medium text-ink-muted">
                  LinkedIn：
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
            我是一名常驻科罗拉多州的后端工程师，专注于 Rust 服务、
            PostgreSQL 数据库和云基础设施。在 Soundpatrol，我从事网络安全、DevOps、Web 服务及开发者工具相关工作，使用 Rust、

            Python、PostgreSQL、Kubernetes 和 GCP。
          </p>
          <p class="mb-4 leading-relaxed">
            早期项目包括整合数千个环境传感器、部署于 AWS 的 Samsung C&amp;T 数字孪生系统；
            采用字幕、评论和语言模型的 YouTube 频道分析平台；
            以及 Hyundai、Kia、Genesis 应用和某 K-pop 组合粉丝应用的后端及数据管道。

          </p>
          <p class="mb-4 leading-relaxed">
            我相信服务应充分利用现代硬件和编程语言。我喜欢深入处理并发、数据库查询、内存使用和部署等细节，而不是把云资源当作工程能力的无限替代品。我也喜欢自托管、构建 Rust 工具，并帮助同事学习 Rust。




          </p>
          <p class="mb-4 leading-relaxed">
            我很珍惜曾涉足不同领域、与多元文化背景的同事共事的经历。2016 至 2017 年，我在朝韩边境附近的美国陆军第 2 步兵师服役 21 个月，与美国和韩国军方人员协作。大学期间，我从事过新闻、摄影和翻译，也做过夜班仓储和物流工作。自 2023 年起，我专注于后端工程、数据管道及其运行所需的基础设施。






          </p>
        </section>

        {/* 2) Professional Career */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            2. 软件开发职业经历
          </h2>

          <div class="space-y-8">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Soundpatrol</h3>
                <span class="text-sm font-mono text-ink-muted">美国 · 远程 | 2026年5月 - 至今</span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">Rust 工程师</p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>使用 Rust 和 Python 开发 Web 服务及内部工具。</li>
                <li>维护云基础设施、Kubernetes 部署和 CI/CD 管道。</li>
                <li>负责监控、访问控制、凭据管理及其他网络安全措施。</li>
              </ul>
            </div>

            {/* GenesisNest */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">GenesisNest</h3>
                <span class="text-sm font-mono text-ink-muted">
                  韩国城南 | 2025年1月 - 2025年7月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                软件工程师
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  为内部服务和某 K‑Pop 组合官方应用开发基础设施管理工具，并熟悉了 Spring Boot 生态。
                </li>
                <li>
                  参与现代汽车集团 Hyundai/Kia/Genesis 官方应用后端的合同开发，融入大型 Java 代码库开展工作。
                </li>
                <li>
                  实现新 API 并修复缺陷，与供应商协作维护在亚洲和欧洲拥有数千万用户的服务。
                </li>
                <li>
                  为国际化工作搭建数据管道，并编写多个 Rust 和 Python 脚本，将译文自动集成到 Hyundai 数据库。与企业 DBA 协作处理关联工厂系统的大型数据集。
                </li>
              </ul>
            </div>

            {/* pampam */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">pampam Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  韩国首尔 | 2024年11月 - 2024年12月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                合同软件工程师
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  为 AI 驱动的 YouTube 分析平台设计 PostgreSQL 架构和 Rust 后端，并主导其在 AWS 和 GCP 上的部署。
                </li>
                <li>
                  通过 YouTube API 主导大规模数据采集，涉及数千万条评论和用户。
                </li>
                <li>
                  使用 LLM 和服务器端开源模型生成分析洞见和摘要。
                </li>
              </ul>
            </div>

            {/* EAN Technology */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">EAN Technology Co. Ltd</h3>
                <span class="text-sm font-mono text-ink-muted">
                  韩国首尔 | 2023年8月 - 2024年8月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                后端软件工程师（负责人）
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>Samsung C&T 数字孪生项目：</b>淘汰运行不佳的微服务架构，改用高度优化的 Axum（Rust）单体服务。将每月云账单从约 5,000 美元降至约 150 美元。
                </li>
                <li>
                  编写约 30,000 行代码（约占总量的 75%），实现 80 个包含复杂领域逻辑的端点。
                </li>
                <li>
                  在九个月的生产运行期间实现零运行时停机，错误处理覆盖率约为 90%。
                </li>
                <li>
                  在 Samsung C&amp;T 总部集成数千个实时物理传感器（BACnet、Modbus）及本地服务器。
                </li>
                <li>
                  优化 PostgreSQL 查询和架构，使资源受限的硬件（2 核、4GB RAM）上的 P99 延迟降至两位数毫秒；部分未优化查询原本需要数秒。
                </li>
                <li>
                  使用 GitHub Runners、AWS CodeDeploy 和 Docker 实现 CI/CD。采用 MUSL、静态链接和基于 scratch 镜像的多阶段构建，生成约 20MB 的 Docker 镜像，以提升安全性和冷启动效率。
                </li>
                <li>
                  指导同事学习 Rust，帮助他们在两个月内从 Node.js/Java 转型。
                </li>
              </ul>
            </div>

            {/* Artifyc */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Artifyc Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  美国德克萨斯州奥斯汀（远程）| 2022年8月 - 2023年3月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                软件工程实习生
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  使用 Python 和 Reddit API 编写 Discord 机器人，抓取并发布艺术委托需求。
                </li>
                <li>
                  在 AWS Lambda 上开发无服务器后端逻辑，用于全球货币换算和附加费计算。
                </li>
                <li>
                  协助修改 React 前端并集成资源。
                </li>
              </ul>
            </div>
          </div>
        </section>


        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">精选项目</h2>
          <ul class="list-disc list-inside text-sm space-y-3 text-ink">
            <li><a href="https://github.com/younghyun1/sillok" class={pageStyles.link}>Sillok</a>：用于结构化工作记录的 Rust 命令行工具，支持仅追加历史、索引化的 Turso/SQLite 投影、修正，以及通过 Git 在多台设备间同步。</li>
            <li><a href="https://github.com/younghyun1/eu5-location-filter" class={pageStyles.link}>EU5 Location DB</a>：基于 Rust 和 Slint 的桌面及 WebAssembly 应用，内置压缩地图数据、预计算索引、可搜索筛选器和可调整大小的表格。</li>
            <li><a href="https://github.com/younghyun1/cyhdev" class={pageStyles.link}>cyhdev.com</a>：这个使用 Rust/Axum、PostgreSQL 和 SolidJS 构建的网站，提供发布、摄影、讨论、聊天和浏览器演示功能。</li>
            <li><a href="https://github.com/younghyun1/oohid" class={pageStyles.link}>oohid</a>：Rust UUID 生成器，支持自定义格式、文件输出和可选的重复检查。</li>
          </ul>
        </section>

        {/* 3) Non-IT Career */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            3. 非 IT 工作经历
          </h2>

          <div class="space-y-6">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
              <h3 class="text-lg font-bold">韩国海洋科学技术院</h3>
                <span class="text-sm font-mono text-ink-muted">
                  韩国首尔 | 2023年6月至7月、2024年8月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                笔译／口译
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  为有挪威大使和东远集团董事长出席的高级别会议提供英语、韩语和挪威语口译。
                </li>
                <li>
                  受托处理高度敏感的政府与商业事务细节，以及海洋研发相关讨论。
                </li>
                <li>
                  为韩国海洋科学技术院人员及院长参加美国能源部活动提供辅导。
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
                    美国陆军
                  </a>{" "}
                  (
                  <a
                    href="https://en.wikipedia.org/wiki/2nd_Infantry_Division_(United_States)"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    第2步兵师
                  </a>
                  ) /{" "}
                  <a
                    href="https://en.wikipedia.org/wiki/Republic_of_Korea_Army"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    韩国陆军
                  </a>
                </h3>
                <span class="text-sm font-mono text-ink-muted">
                  <a
                    href="https://en.wikipedia.org/wiki/Dongducheon"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    东豆川
                  </a>
                  ，韩国 | 2016年3月 - 2017年12月
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
                | 教官 | 中士
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  担任联络兵和教官，为约 6,000 名美国及韩国人员授课并办理行政报到。
                </li>
                <li>
                  教授体能、文化、历史和安全预防课程。
                </li>
                <li>
                  在以下时期保持作战准备状态：{" "}
                  <a
                    href="https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    2017年危机
                  </a>
                  ，并带领一支 13 人团队。
                </li>
              </ul>
            </div>

            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Coupang Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  韩国 | 2020 - 2022
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                装卸工
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  夜班期间在物流仓库从事高强度体力劳动。
                </li>
              </ul>
            </div>
          </div>
        </section>

        {/* 4) Academics */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            4. 教育经历
          </h2>

          <div class="mb-6">
            <div class="flex flex-wrap justify-between items-baseline mb-2">
              <h3 class="text-lg font-bold">成均馆大学</h3>
              <span class="text-sm font-mono text-ink-muted">
                韩国水原 | 2015年3月 - 2023年8月
              </span>
            </div>
            <p class="text-sm font-medium mb-2 text-ink-muted">
              软件工程学学士
            </p>
            <div class="space-y-2 text-sm text-ink">
              <p>
                完成了涵盖计算机科学、Web 与嵌入式软件工程以及电子工程的综合课程，重点学习逻辑、底层语言、操作系统、网络工程和软件工程实践。
              </p>
              <p>
                成均馆大学是韩国历史最悠久的大学，创立于 1398 年，最初是朝鲜王朝依照儒家传统设立的国家官学。韩国解放后，它一直是重要的学术与历史机构；我就读期间，学校也持续位列全球百强大学。
              </p>
            </div>
          </div>

          <div>
            <h4 class="font-bold mb-2 text-md">发表成果</h4>
            <ul class="list-disc list-inside text-sm space-y-2 text-ink">
              <li>
                <b>
                  基于韩语自然语言处理模型的社交媒体社群情感分析
                </b>{" "}
                (IMCOM 2023, IEEE Xplore).
                <br />
                <span class="ml-5 block text-xs text-ink-muted mt-1">
                  使用 KoBERT 分析新冠疫情初期的社交媒体数据情绪，并获毕业设计铜奖。
                </span>
              </li>
            </ul>
          </div>
        </section>

        {/* 5) Volunteer work and interests */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            5. 志愿服务与兴趣
          </h2>

          <div class="grid md:grid-cols-2 gap-8">
            <div>
              <h3 class="font-bold mb-3 text-lg">个人项目</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>
                  <b>oohid：</b>Rust 命令行 UUIDv4 生成器，速度约为 libuuid 的 3 倍，支持重复检查以及 Python/JSON 格式化。
                </li>
                <li>
                  <b>impulsr：</b>YouTube 字幕转录和评论收集工具，用于生成营销摘要。
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-3 text-lg">志愿服务</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>在社区中心教授英语。</li>
                <li>在由残障人士经营的旧货店担任志愿者。</li>
              </ul>
            </div>
          </div>
        </section>

        {/* 6) Hobbies */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            6. 兴趣爱好
          </h2>

          <div class="space-y-4">
            <div>
              <h3 class="font-bold inline mr-2">摄影</h3>
              <span class="text-sm text-ink-muted">
                自 2010 年起业余拍摄风景和人像，已归档约 30,000 张照片。
              </span>
              <div class="mt-2">
                <a
                  href="/photographs"
                  class={`inline-flex items-center ${pageStyles.link} font-medium`}
                >
                  查看摄影作品集 →
                </a>
              </div>
            </div>

            <div>
              <h3 class="font-bold inline mr-2">新闻工作</h3>
              <p class="text-sm text-ink mt-1">
                曾任《成均时报》学生记者。
              </p>
            </div>
          </div>
        </section>

        {/* 7) Qualifications and Other Skills */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            7. 资质与其他技能
          </h2>

          <div class="grid md:grid-cols-2 gap-8 mb-8">
            <div>
              <h3 class="font-bold mb-2">资质与奖项</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>TOEFL:</b> 117/120 (CEFR C2)
                </li>
                <li>
                  <b>IELTS:</b> 8.5/9.0 (CEFR C2)
                </li>
                <li>
                  <b>第 7 名：</b> 2021 Capstone Design & Idea Hackathon
                </li>
                <li>
                  <b>第 3 名：</b> 2020 Fintech Hackathon (SNUST)
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-2">语言</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>韩语（母语）</li>
                <li>英语（C2）</li>
                <li>初级法语、德语、西班牙语、普通话</li>
              </ul>
            </div>
          </div>
        </section>
      </section>
    </main>
  );
}
