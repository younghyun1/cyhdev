import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "2026年11月",
  chrome: {
    overview: "概览",
    updated: "最后更新",
    onThisPage: "本页内容",
    sectionNavigation: "关于我页面",
    previous: "上一页",
    next: "下一页",
    readingTime: "阅读约需 {minutes} 分钟",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "关于我",
      lede: "我是一名后端与基础设施工程师，使用 Rust、PostgreSQL 和云平台。我构建能充分利用底层硬件的服务，也负责让这些服务持续运行的部署、监控和安全工作。",
      sections: [
        {
          id: "introduction",
          heading: "简介",
          blocks: [
            {
              kind: "paragraph",
              text: "我是一名常驻科罗拉多州的韩国软件工程师。在 **Soundpatrol**，我担任 Rust 工程师，从事网络安全、DevOps、Web 服务和开发者工具相关工作，主要使用 Rust、Python、PostgreSQL、Kubernetes 和 Google Cloud。",
            },
            {
              kind: "paragraph",
              text: "此前，我主导了 Samsung C&T 建筑数字孪生的后端与基础设施工作，参与了 Hyundai、Kia 和 Genesis 应用后端的开发，并为一个 YouTube 分析平台构建了数据采集系统。自 2023 年起，我的工作重心是后端服务、数据管道及其底层基础设施。",
            },
            {
              kind: "paragraph",
              text: "我相信服务应充分利用现代硬件和编程语言。具体而言，这意味着严谨的数据库模式设计、有界的队列和缓存、显式的错误处理，以及可检查、可恢复的部署。我乐于帮助同事上手 Rust，也希望与以自己的手艺为荣的人共事。",
            },
          ],
        },
        {
          id: "results",
          heading: "主要成果",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "Samsung C&T 数字孪生每月云支出的降幅，从约 5,000 美元降至 150 美元",
                },
                {
                  value: "80",
                  count: 80,
                  label: "为数字孪生实现的 REST 端点，包含建筑工程领域逻辑",
                },
                {
                  value: "9 个月",
                  count: 9,
                  suffix: " 个月",
                  label: "数字孪生在生产环境中运行且未发生运行时停机的时长",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "本站后端通过 [PGO 和 BOLT](/about-blog/performance) 获得的混合读取吞吐量提升",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "通过多阶段构建生成的静态链接服务容器",
                },
                {
                  value: "30,000",
                  label: "为数字孪生后端编写的代码行数，约占其代码的 75%",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "技术工具箱",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "编程语言",
                  items: ["Rust", "Python", "TypeScript", "SQL", "Java", "C", "C++"],
                },
                {
                  label: "后端",
                  items: ["Axum", "Tokio", "Diesel", "tokio-postgres", "Spring Boot", "FastAPI", "WebSockets", "gRPC", "Protocol Buffers"],
                },
                {
                  label: "数据",
                  items: ["PostgreSQL", "Valkey / Redis", "SQLite / Turso", "MySQL", "FAISS", "tantivy"],
                },
                {
                  label: "基础设施",
                  items: ["Kubernetes", "Helm", "Docker", "OpenTofu / Terraform", "GitHub Actions", "Google Cloud", "AWS", "Linux"],
                },
                {
                  label: "可靠性与安全",
                  items: ["OpenTelemetry", "IAM", "TLS", "Network policy", "Credential lifecycle", "Incident response"],
                },
                {
                  label: "界面与设备",
                  items: ["SolidJS", "Leptos", "Slint", "WebAssembly", "WebRTC", "BACnet", "Modbus"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "了解更多",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "工作经历",
                  href: "/about/experience",
                  meta: "2016年 - 至今",
                  text: "从 Soundpatrol 到第一份实习的各段经历，以及每段经历背后的项目和成果。",
                },
                {
                  title: "项目",
                  href: "/about/projects",
                  meta: "Rust · 开源",
                  text: "Sillok、EU5 Location DB、本网站，以及其他独立和开源作品。",
                },
                {
                  title: "个人背景",
                  href: "/about/background",
                  meta: "教育 · 研究",
                  text: "教育经历、已发表的研究、奖项、语言能力，以及软件之外的经历。",
                },
                {
                  title: "本站如何运作",
                  href: "/about-blog",
                  meta: "技术栈",
                  text: "cyhdev.com 背后的主机、后端、前端、子系统和性能测量。",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "联系方式",
          blocks: [
            {
              kind: "list",
              items: [
                "**邮箱**：[younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**：[github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**：[linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "邮件是联系我最可靠的方式。如果想加入 Minecraft 服务器，也请在邮件中说明。",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "关于我",
      title: "工作经历",
      lede: "在美国初创公司、为韩国企业客户以及早期实习中从事的后端、基础设施和安全工作，以及进入软件行业之前的经历。",
      sections: [
        {
          id: "software",
          heading: "软件工程",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Rust 工程师",
                  period: "2026年5月 - 至今",
                  location: "美国 · 远程",
                  summary: "为一家音乐与音频检测公司负责网络安全、DevOps、Web 服务开发和开发者工具。",
                  highlights: [
                    "维护 Google Cloud 上的 Kubernetes 基础设施、基础设施即代码，以及经过审查、使用短期且按环境限定范围的凭据的部署工作流。",
                    "规划并执行生产数据存储迁移，在每次切换前验证迁移后的状态并演练恢复。",
                    "将指标、日志和链路追踪统一到 OpenTelemetry，并构建用于服务健康、诊断和成本可见性的 Rust 工具。",
                    "负责安全就绪工作：访问管理、凭据轮换与到期跟踪、策略、事件响应和审计证据。",
                    "为运维和支持流程构建内部 Rust 与 Python 服务及工具。",
                  ],
                  tags: ["Rust", "Python", "PostgreSQL", "Kubernetes", "Google Cloud", "OpenTofu", "OpenTelemetry"],
                },
                {
                  organization: "GenesisNest",
                  title: "软件工程师",
                  period: "2025年1月 - 2025年7月",
                  location: "韩国城南",
                  summary: "为内部平台和某 K-pop 组合的官方粉丝应用开发后端服务和基础设施工具。",
                  highlights: [
                    "为 Hyundai、Kia 和 Genesis 官方应用后端贡献 API、缺陷修复和集成功能，在大型 Java 代码库中工作，并与供应商协调亚洲和欧洲数千万用户使用的服务。",
                    "使用 Rust 和 Python 构建国际化管道，将翻译内容转换并加载到企业数据库中，并与负责关联工厂系统数据集的数据库管理员协作。",
                    "为内部服务开发基础设施管理工具。",
                  ],
                  tags: ["Java", "Spring Boot", "Rust", "Python"],
                },
                {
                  organization: "pampam Inc.",
                  title: "合同软件工程师",
                  period: "2024年11月 - 2024年12月",
                  location: "韩国首尔",
                  highlights: [
                    "为 AI 辅助的 YouTube 分析平台设计 PostgreSQL 数据库模式和 Rust 后端，并主导其在 AWS 和 Google Cloud 上的部署。",
                    "构建覆盖数千万条评论和用户的 YouTube API 采集工作流，整合频道数据、转录文本和页面内容。",
                    "集成托管语言模型和服务器端开源模型，生成摘要和营销洞察。",
                  ],
                  tags: ["Rust", "PostgreSQL", "AWS", "Google Cloud"],
                },
                {
                  organization: "EAN Technology Co. Ltd.",
                  title: "后端软件工程师（负责人）",
                  period: "2023年8月 - 2024年8月",
                  location: "韩国首尔",
                  summary: "主导两个建筑与能源平台的后端、数据库和云基础设施开发。",
                  highlights: [],
                  projects: [
                    {
                      name: "Samsung C&T 全球工程中心数字孪生",
                      highlights: [
                        "将 Express 后端和云端微服务架构替换为经过优化的 Rust/Axum 单体服务，使每月基础设施支出从约 **5,000 美元降至 150 美元**。",
                        "编写约 **30,000 行**代码，约占后端的 75%，其中包括承载建筑工程领域逻辑、数据集成和实时操作的 **80 个 REST 端点**。",
                        "在双核、4 GB 的部署环境中，通过异步 Rust、多线程、缓存以及 PostgreSQL 模式与查询调优，将许多 P99 长达数秒的请求路径缩短至个位数或两位数毫秒。",
                        "在 Samsung C&T 总部现场，通过 BACnet 和 Modbus 集成数千个传感器、仪表以及暖通空调和水系统控制器。",
                        "在生产环境中运行九个月，未发生运行时停机，错误处理覆盖率约为 90%。",
                        "使用 GitHub runners、AWS CodeDeploy 和 Docker 构建 CI/CD；针对处理器调优的静态构建生成了约 20 MB 的可移植容器。",
                        "通过 ECS、EC2、SQS、SNS、S3 和 SageMaker 接入预测模型推理，落实 Samsung SDS 的安全要求，并将后端与合作工作室开发的 Unreal Engine 4 界面集成。",
                      ],
                    },
                    {
                      name: "AI 能源管理服务",
                      highlights: [
                        "从零设计 PostgreSQL 数据库模式和 Rust/Axum API，编写约 **25,000 行**代码，约占后端的 70%。",
                        "吸取数字孪生项目的经验，选择通过 tokio-postgres 直接编写 SQL 而非使用 ORM，以掌控查询和编译时间。",
                        "在由后端、前端和数据科学工程师组成的九人团队中，帮助两名来自 Node.js 和 Java 背景的工程师在约一个月内熟练使用 Rust。",
                      ],
                    },
                  ],
                  tags: ["Rust", "Axum", "PostgreSQL", "AWS", "Docker", "BACnet", "Modbus"],
                },
                {
                  organization: "Artifyc Inc.",
                  title: "软件工程实习生",
                  period: "2022年8月 - 2023年3月",
                  location: "德克萨斯州奥斯汀 · 远程",
                  highlights: [
                    "构建 Python Discord 机器人，通过 Reddit API 查找艺术委托需求并发布到社区频道，在 AWS Lambda 上每五分钟运行一次。",
                    "实现 Lambda 逻辑，处理跨司法管辖区的货币换算、收费和附加费计算。",
                    "参与 React 前端在布局、素材和文案方面的修改。",
                  ],
                  tags: ["Python", "AWS Lambda", "React"],
                },
              ],
            },
          ],
        },
        {
          id: "other",
          heading: "进入软件行业之前",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "韩国海洋科学技术院",
                  title: "笔译／口译",
                  period: "2023年6月至7月、2024年8月",
                  location: "韩国首尔 · 合同",
                  highlights: [
                    "在一场海洋研究会议上提供英语、韩语和挪威语口译，出席者包括挪威驻韩大使、挪威国务秘书、该院领导层和东远集团董事长。",
                    "为一场国际会议和一场关于联合项目的美国能源部活动翻译演讲稿，并辅导该院职员和院长。",
                  ],
                },
                {
                  organization: "韩国陆军 · 美国陆军第 2 步兵师",
                  title: "中士，KATUSA 联络兵兼教官",
                  period: "2016年3月 - 2017年12月",
                  location: "韩国东豆川",
                  highlights: [
                    "通过 [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army) 项目在该师司令部营服役，身处韩美联合环境。",
                    "为约 6,000 名美韩人员讲授体能、文化、历史和预防课程，并为其办理报到手续。",
                    "带领一支 13 人团队，在半岛 [2017 年危机](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis)期间保持战备状态。",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "物流装卸工",
                  period: "2020年 - 2022年",
                  location: "韩国 · 临时合同",
                  highlights: [
                    "在大学学习期间从事夜班仓库工作。",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "关于我",
      title: "项目",
      lede: "独立和开源作品，大多使用 Rust 编写。交互式演示位于[项目页面](/projects)。",
      sections: [
        {
          id: "featured",
          heading: "精选项目",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "命令行工作记录工具。目标、已完成任务、修正和撤回都以仅追加事件的形式记录，并维护带索引的当前投影；基于 Git 的归档可在多台机器之间合并历史。",
                  tags: ["事件溯源", "同步"],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "用于 Europa Universalis V 地图数据的桌面和浏览器应用，内置压缩数据集、预计算的搜索与排序索引、分类筛选器和可调整大小的表格。[在此试用](/eu5-locations-db)。",
                  tags: ["原生 + Web", "索引"],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "本站：发布、摄影、论坛、支持通话的实时聊天、Minecraft 地形探索器和 WebAssembly 演示，全部由一个 Rust 二进制程序提供。[了解构建方式](/about-blog)。",
                  tags: ["单体仓库", "自托管"],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "UUIDv4 生成与格式化工具，支持终端和文件输出、可选的重复检查，以及兼容 Python 和 JSON 的格式；在早期的本地基准测试中，速度约为 uuidgen 的三倍。",
                  tags: ["吞吐量"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "更多作品",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**：为 Rust 编写的 Minecraft 服务器提交的[上游修复](https://github.com/Pumpkin-MC/Pumpkin/pull/3447)，在导入和保存时保留 Paper 的主世界设置；目前等待审查。",
                "**Minecraft 地图工具**：支撑本站[地形探索器](/minecraft)的 Paper 插件和 Rust 服务，包括紧凑的二进制生物群系瓦片格式和基于种子的生物群系预测。",
                "**文件系统同步**：用于在异构文件系统之间双向同步文件夹并清理构建产物的 Rust 工具，支持试运行、操作日志、可选校验和，并防止意外的大规模删除。",
                "**impulsr**：YouTube 转录与评论采集工具，可为营销工作生成语言模型摘要。",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "我的构建方式",
          blocks: [
            {
              kind: "paragraph",
              text: "我偏好小而易于检查的系统：一个二进制程序够用时就只用一个，每个队列和缓存都有明确上限，数据格式根据访问模式而非习惯来选择。",
            },
            {
              kind: "paragraph",
              text: "先测量，再下结论；本站的[性能笔记](/about-blog/performance)说明了测量方式。",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "关于我",
      title: "个人背景",
      lede: "教育、研究、资质，以及软件之外的兴趣。",
      sections: [
        {
          id: "education",
          heading: "教育经历",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "成均馆大学",
                  title: "软件工程工学学士",
                  period: "2015年3月 - 2023年8月",
                  location: "韩国水原",
                  summary: "计算机科学与工程系。学位期间包含义务兵役。",
                  highlights: [
                    "除软件工程实践外，还修读了嵌入式软件、逻辑电路、操作系统、网络、底层编程和电子学等课程。",
                    "成均馆大学的历史可追溯至 1398 年，当时是朝鲜王朝的最高学府，如今是韩国顶尖的研究型大学之一。",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "研究",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**基于韩语自然语言处理模型的社交媒体社群情感分析。**将 KoBERT 应用于韩语社交媒体的情感分析与分类，考察 COVID-19 疫情初期的情绪变化。",
            },
            {
              kind: "paragraph",
              text: "该项目在学校毕业论文与项目评奖中获得铜奖，参评作品约 60 件。",
            },
          ],
        },
        {
          id: "awards",
          heading: "奖项与资质",
          blocks: [
            {
              kind: "table",
              columns: ["年份", "奖项与资质", "详情"],
              rows: [
                ["2023", "**铜奖**", "成均馆大学毕业论文与项目奖"],
                ["2023", "**IELTS 8.5 / 9.0**", "CEFR C2"],
                ["2021", "**第 7 名**", "Capstone Design and Idea Hackathon，韩国互联网信息学会"],
                ["2020", "**第 3 名**", "Fintech Hackathon，首尔科学技术大学"],
                ["2018", "**TOEFL 117 / 120**", "CEFR C2"],
              ],
            },
          ],
        },
        {
          id: "languages",
          heading: "语言",
          blocks: [
            {
              kind: "list",
              items: [
                "**韩语**：母语。",
                "**英语**：专业级流利（C2）。",
                "**法语、德语、西班牙语和普通话**：基础了解。",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "软件之外",
          blocks: [
            {
              kind: "list",
              items: [
                "**摄影**：自 2010 年起拍摄风景和人像，已归档约 30,000 张照片。部分作品见[图库](/photographs)。",
                "**新闻**：曾任成均馆大学英文杂志《Sungkyun Times》的学生记者，报道科技、安全和国际事务。",
                "**教学与志愿服务**：在社区中心教授英语，并在由残障人士经营的旧货店做志愿者。",
                "**翻译**：为公共机构提供口译和笔译；参见[工作经历](/about/experience#other)。",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
