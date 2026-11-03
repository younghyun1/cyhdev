import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "2026 年 11 月",
  chrome: {
    overview: "概覽",
    updated: "最後更新",
    onThisPage: "本頁內容",
    sectionNavigation: "關於我頁面",
    previous: "上一頁",
    next: "下一頁",
    readingTime: "閱讀約需 {minutes} 分鐘",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "關於我",
      lede: "我是一名後端與基礎設施工程師，使用 Rust、PostgreSQL 與雲端平台。我打造能善用底層硬體的服務，也負責維持服務穩定運作的部署、監控與資訊安全工作。",
      sections: [
        {
          id: "introduction",
          heading: "簡介",
          blocks: [
            {
              kind: "paragraph",
              text: "我是一名常駐科羅拉多州的韓國軟體工程師。在 **Soundpatrol**，我擔任 Rust 工程師，負責資訊安全、DevOps、Web 服務與開發者工具，主要使用 Rust、Python、PostgreSQL、Kubernetes 與 Google Cloud。",
            },
            {
              kind: "paragraph",
              text: "在此之前，我主導 Samsung C&T 建築數位孿生的後端與基礎設施工作，參與 Hyundai、Kia 與 Genesis 應用程式後端的開發，並為一個 YouTube 分析平台建置資料蒐集系統。自 2023 年起，我的工作重心是後端服務、資料管線及其底層基礎設施。",
            },
            {
              kind: "paragraph",
              text: "我相信服務應善用現代硬體與程式語言。實際上，這代表嚴謹的資料庫 schema 設計、有上限的佇列與快取、明確的錯誤處理，以及可檢查、可復原的部署。我樂於協助同事上手 Rust，也希望與以自身技藝為榮的人共事。",
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
                  label: "Samsung C&T 數位孿生每月雲端支出的降幅，從約 5,000 美元降至 150 美元",
                },
                {
                  value: "80",
                  count: 80,
                  label: "為數位孿生實作的 REST 端點，包含營建工程領域邏輯",
                },
                {
                  value: "9 個月",
                  count: 9,
                  suffix: " 個月",
                  label: "數位孿生在正式環境中運作且未發生執行期停機的時間",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "本站後端透過 [PGO 與 BOLT](/about-blog/performance) 獲得的混合讀取吞吐量提升",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "以多階段建置產生的靜態連結服務容器",
                },
                {
                  value: "30,000",
                  label: "為數位孿生後端撰寫的程式碼行數，約佔其程式碼的 75%",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "技術工具箱",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "程式語言",
                  items: ["Rust", "Python", "TypeScript", "SQL", "Java", "C", "C++"],
                },
                {
                  label: "後端",
                  items: ["Axum", "Tokio", "Diesel", "tokio-postgres", "Spring Boot", "FastAPI", "WebSockets", "gRPC", "Protocol Buffers"],
                },
                {
                  label: "資料",
                  items: ["PostgreSQL", "Valkey / Redis", "SQLite / Turso", "MySQL", "FAISS", "tantivy"],
                },
                {
                  label: "基礎設施",
                  items: ["Kubernetes", "Helm", "Docker", "OpenTofu / Terraform", "GitHub Actions", "Google Cloud", "AWS", "Linux"],
                },
                {
                  label: "可靠性與資訊安全",
                  items: ["OpenTelemetry", "IAM", "TLS", "Network policy", "Credential lifecycle", "Incident response"],
                },
                {
                  label: "介面與裝置",
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
                  title: "工作經歷",
                  href: "/about/experience",
                  meta: "2016 年 - 至今",
                  text: "從 Soundpatrol 到第一份實習的各段經歷，以及每段經歷背後的專案與成果。",
                },
                {
                  title: "專案",
                  href: "/about/projects",
                  meta: "Rust · 開源",
                  text: "Sillok、EU5 Location DB、本網站，以及其他獨立與開源作品。",
                },
                {
                  title: "個人背景",
                  href: "/about/background",
                  meta: "學歷 · 研究",
                  text: "學歷、已發表的研究、獎項、語言能力，以及軟體以外的經歷。",
                },
                {
                  title: "本站如何運作",
                  href: "/about-blog",
                  meta: "技術架構",
                  text: "cyhdev.com 背後的主機、後端、前端、子系統與效能量測。",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "聯絡方式",
          blocks: [
            {
              kind: "list",
              items: [
                "**電子郵件**：[younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**：[github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**：[linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "電子郵件是聯絡我最可靠的方式。若想加入 Minecraft 伺服器，也請在信中說明。",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "關於我",
      title: "工作經歷",
      lede: "在美國新創公司、為韓國企業客戶以及早期實習中從事的後端、基礎設施與資訊安全工作，以及投入軟體業之前的經歷。",
      sections: [
        {
          id: "software",
          heading: "軟體工程",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Rust 工程師",
                  period: "2026 年 5 月 - 至今",
                  location: "美國 · 遠端",
                  summary: "為一家音樂與音訊偵測公司負責資訊安全、DevOps、Web 服務開發與開發者工具。",
                  highlights: [
                    "維護 Google Cloud 上的 Kubernetes 基礎設施、基礎設施即程式碼，以及經過審查、使用短效且依環境限定範圍之認證資訊的部署工作流程。",
                    "規劃並執行正式環境資料儲存的遷移，在每次切換前驗證遷移後的狀態並演練復原。",
                    "將指標、日誌與追蹤整合至 OpenTelemetry，並開發用於服務健康狀態、診斷與成本可視性的 Rust 工具。",
                    "負責資安整備：存取管理、認證資訊輪替與到期追蹤、政策、事件應變及稽核證據。",
                    "為維運與支援流程開發內部 Rust 與 Python 服務及工具。",
                  ],
                  tags: ["Rust", "Python", "PostgreSQL", "Kubernetes", "Google Cloud", "OpenTofu", "OpenTelemetry"],
                },
                {
                  organization: "GenesisNest",
                  title: "軟體工程師",
                  period: "2025 年 1 月 - 2025 年 7 月",
                  location: "韓國城南",
                  summary: "為內部平台及某 K-pop 團體的官方粉絲應用程式開發後端服務與基礎設施工具。",
                  highlights: [
                    "為 Hyundai、Kia 與 Genesis 官方應用程式後端貢獻 API、錯誤修正與整合功能，在大型 Java 程式碼庫中工作，並就亞洲與歐洲數千萬人使用的服務與廠商協調。",
                    "以 Rust 與 Python 建置國際化管線，將翻譯內容轉換並載入企業資料庫，並與負責工廠系統相關資料集的資料庫管理員協作。",
                    "為內部服務開發基礎設施管理工具。",
                  ],
                  tags: ["Java", "Spring Boot", "Rust", "Python"],
                },
                {
                  organization: "pampam Inc.",
                  title: "約聘軟體工程師",
                  period: "2024 年 11 月 - 2024 年 12 月",
                  location: "韓國首爾",
                  highlights: [
                    "為 AI 輔助的 YouTube 分析平台設計 PostgreSQL schema 與 Rust 後端，並主導其在 AWS 與 Google Cloud 上的部署。",
                    "建置涵蓋數千萬則留言與使用者的 YouTube API 蒐集工作流程，整合頻道資料、逐字稿與頁面內容。",
                    "整合託管型語言模型與伺服器端開源模型，產生摘要與行銷洞察。",
                  ],
                  tags: ["Rust", "PostgreSQL", "AWS", "Google Cloud"],
                },
                {
                  organization: "EAN Technology Co. Ltd.",
                  title: "後端軟體工程師（負責人）",
                  period: "2023 年 8 月 - 2024 年 8 月",
                  location: "韓國首爾",
                  summary: "主導兩個建築與能源平台的後端、資料庫與雲端基礎設施開發。",
                  highlights: [],
                  projects: [
                    {
                      name: "Samsung C&T 全球工程中心數位孿生",
                      highlights: [
                        "將 Express 後端與雲端微服務架構替換為經過最佳化的 Rust/Axum 單體服務，使每月基礎設施支出從約 **5,000 美元降至 150 美元**。",
                        "撰寫約 **30,000 行**程式碼，約佔後端的 75%，其中包括承載營建工程領域邏輯、資料整合與即時作業的 **80 個 REST 端點**。",
                        "在雙核心、4 GB 的部署環境中，透過非同步 Rust、多執行緒、快取，以及 PostgreSQL schema 與查詢調校，將許多 P99 長達數秒的請求路徑縮短至個位數或兩位數毫秒。",
                        "在 Samsung C&T 總部現場，透過 BACnet 與 Modbus 整合數千個感測器、儀表，以及空調與水系統控制器。",
                        "在正式環境中運作九個月，未發生執行期停機，錯誤處理涵蓋率約 90%。",
                        "使用 GitHub runners、AWS CodeDeploy 與 Docker 建置 CI/CD；針對處理器調校的靜態建置產生約 20 MB 的可攜式容器。",
                        "透過 ECS、EC2、SQS、SNS、S3 與 SageMaker 串接預測模型推論，落實 Samsung SDS 的資安要求，並將後端與合作工作室打造的 Unreal Engine 4 介面整合。",
                      ],
                    },
                    {
                      name: "AI 能源管理服務",
                      highlights: [
                        "從零設計 PostgreSQL schema 與 Rust/Axum API，撰寫約 **25,000 行**程式碼，約佔後端的 70%。",
                        "汲取數位孿生專案的經驗，選擇透過 tokio-postgres 直接撰寫 SQL 而非使用 ORM，以掌控查詢與編譯時間。",
                        "在由後端、前端與資料科學工程師組成的九人團隊中，協助兩名來自 Node.js 與 Java 背景的工程師在約一個月內熟練運用 Rust。",
                      ],
                    },
                  ],
                  tags: ["Rust", "Axum", "PostgreSQL", "AWS", "Docker", "BACnet", "Modbus"],
                },
                {
                  organization: "Artifyc Inc.",
                  title: "軟體工程實習生",
                  period: "2022 年 8 月 - 2023 年 3 月",
                  location: "德州奧斯汀 · 遠端",
                  highlights: [
                    "建置 Python Discord 機器人，透過 Reddit API 尋找藝術委託需求並發布至社群頻道，在 AWS Lambda 上每五分鐘執行一次。",
                    "實作 Lambda 邏輯，處理跨司法管轄區的貨幣換算、收費與附加費計算。",
                    "參與 React 前端在版面、素材與文案方面的修改。",
                  ],
                  tags: ["Python", "AWS Lambda", "React"],
                },
              ],
            },
          ],
        },
        {
          id: "other",
          heading: "投入軟體業之前",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "韓國海洋科學技術院",
                  title: "筆譯／口譯",
                  period: "2023 年 6 至 7 月、2024 年 8 月",
                  location: "韓國首爾 · 約聘",
                  highlights: [
                    "在一場海洋研究會議上提供英語、韓語與挪威語口譯，出席者包括挪威駐韓大使、挪威國務秘書、該院領導階層及東遠集團董事長。",
                    "為一場國際會議及一場關於合作專案的美國能源部活動翻譯演講稿，並指導該院職員與院長。",
                  ],
                },
                {
                  organization: "韓國陸軍 · 美國陸軍第 2 步兵師",
                  title: "中士，KATUSA 聯絡兵兼教官",
                  period: "2016 年 3 月 - 2017 年 12 月",
                  location: "韓國東豆川",
                  highlights: [
                    "透過 [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army) 計畫在該師司令部營服役，身處韓美聯合環境。",
                    "為約 6,000 名美韓人員講授體能、文化、歷史與預防課程，並為其辦理報到手續。",
                    "帶領一支 13 人團隊，在半島 [2017 年危機](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis)期間維持戰備。",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "物流裝卸員",
                  period: "2020 年 - 2022 年",
                  location: "韓國 · 臨時約聘",
                  highlights: [
                    "大學就學期間從事夜班倉庫工作。",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "關於我",
      title: "專案",
      lede: "獨立與開源作品，大多以 Rust 撰寫。互動式展示位於[專案頁面](/projects)。",
      sections: [
        {
          id: "featured",
          heading: "精選專案",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "命令列工作紀錄工具。目標、已完成任務、修正與撤回都以僅附加事件的形式記錄，並維護附索引的目前投影；以 Git 為基礎的封存可在多台電腦之間合併歷史。",
                  tags: ["事件溯源", "同步"],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "用於 Europa Universalis V 地圖資料的桌面與瀏覽器應用程式，內嵌壓縮資料集、預先計算的搜尋與排序索引、分類篩選器及可調整大小的表格。[在此試用](/eu5-locations-db)。",
                  tags: ["原生 + Web", "索引"],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "本站：發文、攝影、論壇、支援通話的即時聊天、Minecraft 地形探索器與 WebAssembly 展示，全部由一個 Rust 二進位檔提供。[了解建置方式](/about-blog)。",
                  tags: ["單一儲存庫", "自行託管"],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "UUIDv4 產生與格式化工具，支援終端機與檔案輸出、選用的重複檢查，以及相容 Python 與 JSON 的格式；在早期的本機效能測試中，速度約為 uuidgen 的三倍。",
                  tags: ["吞吐量"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "其他作品",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**：為以 Rust 撰寫的 Minecraft 伺服器提交的[上游修正](https://github.com/Pumpkin-MC/Pumpkin/pull/3447)，在匯入與儲存時保留 Paper 的主世界設定；目前等待審查。",
                "**Minecraft 地圖工具**：支撐本站[地形探索器](/minecraft)的 Paper 外掛與 Rust 服務，包括精簡的二進位生態域圖磚格式，以及以種子碼為基礎的生態域預測。",
                "**檔案系統同步**：在異質檔案系統之間雙向同步資料夾並清理建置產物的 Rust 工具，支援試執行、操作日誌、選用的檢查碼，並防止非預期的大量刪除。",
                "**impulsr**：YouTube 逐字稿與留言蒐集工具，可為行銷工作產生語言模型摘要。",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "我的開發方式",
          blocks: [
            {
              kind: "paragraph",
              text: "我偏好小而易於檢查的系統：一個二進位檔夠用時就只用一個，每個佇列與快取都有明確上限，資料格式依存取模式而非習慣來選擇。",
            },
            {
              kind: "paragraph",
              text: "先量測，再下結論。當發現某個效能測試量到的是自己的用戶端而非伺服器時，這項修正也會記錄下來；本站的[效能筆記](/about-blog/performance)就是一例。",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "關於我",
      title: "個人背景",
      lede: "學歷、研究、資格，以及軟體以外的興趣。",
      sections: [
        {
          id: "education",
          heading: "學歷",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "成均館大學",
                  title: "軟體工程工學學士",
                  period: "2015 年 3 月 - 2023 年 8 月",
                  location: "韓國水原",
                  summary: "資訊工程學系。修業期間包含義務役。",
                  highlights: [
                    "除軟體工程實務外，也修習嵌入式軟體、邏輯電路、作業系統、網路、低階程式設計與電子學等課程。",
                    "成均館大學的歷史可追溯至 1398 年，當時為朝鮮王朝的最高學府，如今是韓國頂尖的研究型大學之一。",
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
              text: "**以韓語 NLP 模型對社群媒體社群進行情緒分析。**將 KoBERT 應用於韓語社群媒體的情緒分析與分類，探討 COVID-19 疫情初期的情緒變化。",
            },
            {
              kind: "paragraph",
              text: "此專題在校內畢業論文與專題評選中獲得銅獎，參賽作品約 60 件。",
            },
          ],
        },
        {
          id: "awards",
          heading: "獎項與資格",
          blocks: [
            {
              kind: "table",
              columns: ["年份", "獎項與資格", "說明"],
              rows: [
                ["2023", "**銅獎**", "成均館大學畢業論文與專題獎"],
                ["2023", "**IELTS 8.5 / 9.0**", "CEFR C2"],
                ["2021", "**第 7 名**", "Capstone Design and Idea Hackathon，韓國網際網路資訊學會"],
                ["2020", "**第 3 名**", "Fintech Hackathon，首爾科學技術大學"],
                ["2018", "**TOEFL 117 / 120**", "CEFR C2"],
              ],
            },
          ],
        },
        {
          id: "languages",
          heading: "語言",
          blocks: [
            {
              kind: "list",
              items: [
                "**韓語**：母語。",
                "**英語**：專業級流利（C2）。",
                "**法語、德語、西班牙語與華語**：基礎認識。",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "軟體之外",
          blocks: [
            {
              kind: "list",
              items: [
                "**攝影**：自 2010 年起拍攝風景與人像，已典藏約 30,000 張照片。部分作品收錄於[相簿](/photographs)。",
                "**新聞**：曾任成均館大學英文雜誌《Sungkyun Times》的學生記者，報導科技、安全與國際事務。",
                "**教學與志工服務**：在社區中心教授英文，並在由身心障礙者經營的二手商店擔任志工。",
                "**翻譯**：為公家機關提供口譯與筆譯；請參閱[工作經歷](/about/experience#other)。",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
