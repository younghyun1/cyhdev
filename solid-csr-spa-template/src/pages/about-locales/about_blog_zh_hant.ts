import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "2026 年 11 月",
  chrome: {
    overview: "概覽",
    updated: "最後更新",
    onThisPage: "本頁內容",
    sectionNavigation: "技術架構頁面",
    previous: "上一頁",
    next: "下一頁",
    readingTime: "閱讀約需 {minutes} 分鐘",
  },
  pages: {
    overview: {
      eyebrow: "關於本站",
      title: "部落格技術架構",
      lede: "一個 Rust 二進位檔、一個 PostgreSQL 資料庫、一台小型主機。這些頁面說明本站如何建置、託管、確保安全與進行量測，以及為何採用這樣的架構。",
      sections: [
        {
          id: "at-a-glance",
          heading: "重點速覽",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53,889 req/s",
                  label: "在四個伺服器核心上啟用 PGO 與 BOLT 後，混合公開讀取吞吐量的中位數",
                },
                {
                  value: "~150 µs",
                  label: "透過 UNIX socket 觀察到的資料庫往返時間；透過 localhost TCP 則為 ~600 µs",
                },
                {
                  value: "130 KiB",
                  count: 130,
                  suffix: " KiB",
                  label: "初始前端資源的 gzip 預算，於打包建置時強制執行",
                },
                {
                  value: "8",
                  count: 8,
                  label: "介面語言，每種語言僅在選取時下載",
                },
                {
                  value: "0.15 ms",
                  label: "啟用 `TCP_NODELAY` 後 HTTP/2 圖磚回應完成時間的中位數，先前為 41 ms",
                },
                {
                  value: "1",
                  count: 1,
                  label: "同時提供 TLS、API、WebSockets、WebRTC 媒體與內嵌前端的程序",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "請求路徑",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "瀏覽器",
                  detail: "透過 TLS 的 HTTP/2 或 HTTP/1.1。前端資源以附指紋的不可變 URL 快取。",
                },
                {
                  label: "連線准入",
                  detail: "TCP 連線在 TLS 之前進行准入：總計最多 4,096 條，每個位址或 IPv6 /64 最多 64 條。",
                },
                {
                  label: "rustls",
                  detail: "TLS 在 Rust 程序內終止，每個 socket 都啟用 `TCP_NODELAY`。",
                },
                {
                  label: "Axum",
                  detail: "依序經過安全標頭、工作階段、速率限制、壓縮與本文大小限制，再交給功能處理常式。",
                },
                {
                  label: "快取 · Diesel",
                  detail: "有上限的記憶體內快取負責熱門讀取，其餘請求透過非同步 Diesel 處理。",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "位於同一台主機，透過 UNIX socket 連線，並使用 io_uring 非同步 I/O。",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "章節",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "基礎設施",
                  href: "/about-blog/infrastructure",
                  meta: "主機 · Gentoo · 網路",
                  text: "主機、作業系統與檔案系統、網路，以及為何沒有反向代理。",
                },
                {
                  title: "後端",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "服務技術堆疊、資料模型、執行期限制、安全標頭與 API 契約。",
                },
                {
                  title: "前端",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "應用程式外殼、載入預算、在地化、設計系統與動態效果。",
                },
                {
                  title: "子系統",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "使用程序內 SFU 的即時聊天、Minecraft 探索器、Geo-IP 查詢與 WebAssembly。",
                },
                {
                  title: "效能",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "最佳化建置、量錯對象的效能測試，以及修正後的數據。",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "變更如何驗證",
          blocks: [
            {
              kind: "list",
              items: [
                "以針對原生與 WebAssembly 目標的 Clippy 檢查作為實作階段的關卡。",
                "透過 cargo xtask 在本機執行格式檢查、原生單元測試、OpenAPI 契約漂移檢查，以及前端的型別檢查、lint、單元測試與建置。",
                "在 Chromium 與 WebKit 上執行 Playwright，包括 axe 無障礙檢查，以及最窄至 320 CSS 像素的行動版重排檢查。",
                "針對拋棄式 PostgreSQL 執行個體執行整合測試與遷移回滾測試，並在本機執行 gitleaks 機密掃描。",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "技術架構",
      title: "基礎設施",
      lede: "家中的一台小型主機、一套以原始碼為基礎的 Linux 發行版，而且不放任何流量用不到的層。",
      sections: [
        {
          id: "host",
          heading: "主機",
          blocks: [
            {
              kind: "paragraph",
              text: "本站執行於我住處的一台[迷你伺服器](https://store.minisforum.com/products/minisforum-um690l-slim)：配備最高可加速至 4.9 GHz 的八核心 Ryzen 9 行動處理器、32 GB 記憶體與 1 TB NVMe SSD。約 400 美元的價格，比租用一年效能明顯較弱的雲端硬體還便宜，也反映了我對自行託管的熱愛。",
            },
            {
              kind: "paragraph",
              text: "它執行前後端整合的伺服器、PostgreSQL 與一台 Minecraft 伺服器。想來玩的話，請寄電子郵件給我。",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo 與 btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) 並非伺服器的常見選擇；在工作上，我會選擇 ext4 上的 Debian stable。在這裡，我喜歡針對手上的處理器最佳化，親自編譯套件。",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) 提供快照、壓縮與寫入時複製，但寫入時複製會讓資料庫檔案嚴重碎片化，因此 PostgreSQL 與 Minecraft 的資料目錄不啟用寫入時複製。一項[檔案系統效能測試](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison)顯示，預設設定的 btrfs 落後於 ext4 與 XFS；我猜停用寫入時複製能弭平大部分差距，這會是一項有趣的量測。",
            },
          ],
        },
        {
          id: "network",
          heading: "網路",
          blocks: [
            {
              kind: "paragraph",
              text: "流量透過 1 Gbps 有線 Xfinity 連線進入；業者提供的數據機不支援 2.5 Gb 乙太網路，不過本站不會受到影響。DNS 由 Route 53 託管。",
            },
            {
              kind: "paragraph",
              text: "主機內部沒有反向代理、容器執行環境或服務網格。PostgreSQL 直接在作業系統上執行，一個 Rust 二進位檔同時提供前端與 API。一般 HTTP 請求也由同一個程序重新導向至 HTTPS。",
            },
          ],
        },
        {
          id: "sockets",
          heading: "UNIX socket",
          blocks: [
            {
              kind: "paragraph",
              text: "伺服器透過 UNIX socket 而非 localhost TCP 連線至 PostgreSQL，完全繞過網路堆疊。[效能測試](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/)顯示交易處理速率幾乎翻倍；在這裡，我觀察到的往返時間低至 **150 µs**，而透過 localhost 約為 **600 µs**。這種作法很老派，與多數雲端部署無關，但速度更快。",
            },
            {
              kind: "paragraph",
              text: "2025 年 9 月推出的 PostgreSQL 18 新增了[非同步 I/O](https://pganalyze.com/blog/postgres-18-async-io)，本站已啟用 io_uring，讀取速度明顯提升。",
            },
          ],
        },
        {
          id: "deploy",
          heading: "建置與部署",
          blocks: [
            {
              kind: "list",
              items: [
                "前端建置產物連同預先壓縮的 zstd 與 gzip 版本一起內嵌於後端二進位檔，因此正式環境中不執行任何 Node.js 程序。",
                "最佳化建置可透過設定檔引導最佳化進行訓練，並以 BOLT 進行連結後最佳化；詳見[效能](/about-blog/performance)。",
                "認證資訊、憑證與執行期資料都不進入 Git，變更離開本機前會先經過 gitleaks 掃描。",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "技術架構",
      title: "後端",
      lede: "建構於 Axum 與 Tokio 之上的 Rust，透過非同步 Diesel 存取 PostgreSQL，並為用戶端能佔用的一切資源設下明確上限。",
      sections: [
        {
          id: "stack",
          heading: "技術堆疊",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "執行環境", items: ["Axum", "Tokio", "hyper", "axum-server", "rustls", "mimalloc"] },
                { label: "資料", items: ["PostgreSQL 18", "Diesel", "diesel-async", "tantivy", "bitcode", "zstd"] },
                { label: "整合", items: ["openidconnect", "lettre", "Amazon S3", "webrtc-rs", "image + AVIF"] },
                { label: "API 契約", items: ["utoipa", "OpenAPI", "Swagger UI", "Generated TypeScript"] },
              ],
            },
            {
              kind: "paragraph",
              text: "服務是共用狀態的單一程序，包含記憶體內工作階段、啟動時快取同步與排程維護工作。全域配置器為 mimalloc；既然主機有真正的多核心，配置器的行為就值得認真看待。",
            },
          ],
        },
        {
          id: "data",
          heading: "資料模型",
          blocks: [
            {
              kind: "paragraph",
              text: "在我所見的韓國新創與企業工作中，MySQL 與 MariaDB 被視為唯一正經的關聯式資料庫。但在我在意的面向上，從原生 UUID 到二進位 JSON，PostgreSQL 早已超越它們。",
            },
            {
              kind: "list",
              items: [
                "**文章**包含 slug、摘要、JSONB 中繼資料、發布狀態、反正規化計數器與標籤對應。留言透過可為空值的父參照串成討論串，文章投票與留言投票分別存放在獨立的資料表中。",
                "**帳號**保存國家與語言，搭配角色與權限資料表，個人檔案圖片則依版本存放在獨立的資料表中，而非放在頻繁存取的使用者資料列上。",
                "**主鍵**採用依時間排序的 UUIDv7：不像流水號 ID 那樣可被列舉，對 B-tree 區域性也比隨機的 UUIDv4 友善得多。",
                "**搜尋**使用持久化至磁碟的 tantivy 索引，涵蓋文章標題與標籤，並與仍作為權威資料來源的 PostgreSQL 保持一致。",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "執行期限制",
          blocks: [
            {
              kind: "list",
              items: [
                "連線在 TLS 之前透過共用限制器准入：最多 4,096 條開啟中的連線，每個 IPv4 位址或 IPv6 /64 最多 64 條。被拒絕的 socket 不經交握即關閉。",
                "HTTP/1 請求標頭必須在 20 秒內送達；HTTP/2 連線每 30 秒 ping 一次，允許 128 個並行串流。",
                "每個快取都有固定上限，以及可回源讀取的權威資料來源；沒有任何快取會隨用戶端控制的鍵成長。例如，WebAssembly 套件快取最多佔用 256 MiB。",
                "影像編碼等阻塞工作在有上限的工作執行緒集區中執行，而非佔用 Tokio 工作執行緒。",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "安全標頭",
          blocks: [
            {
              kind: "paragraph",
              text: "Content-Security-Policy 在啟動時建構一次。內嵌指令碼只能透過由編譯進二進位檔的 `index.html` 計算出的 SHA-256 雜湊放行，因此首次繪製前執行的主題指令碼不會與政策脫節。主來源從不建立 WebAssembly 執行個體：展示與 EU5 應用程式在各自政策下的沙箱框架中執行。",
            },
            {
              kind: "paragraph",
              text: "工作階段 cookie 設有 `Secure` 與 `HttpOnly`，超級使用者操作會在伺服器端檢查角色，帳號可連結 OpenID Connect 身分，正式環境的 Swagger UI 需要驗證才能存取。",
            },
          ],
        },
        {
          id: "contracts",
          heading: "API 契約與靜態資源",
          blocks: [
            {
              kind: "list",
              items: [
                "處理常式附有 OpenAPI 註解，前端的 TypeScript 用戶端由此產生；產生的檔案一旦漂移，cargo xtask openapi 就會失敗。",
                "Vite 的資訊清單與靜態資源一同內嵌。只有列於資訊清單且附指紋的檔案才會套用一年期的 `immutable` 快取政策；HTML 一律重新驗證。",
                "不存在的資源路徑會回傳附 `no-store` 的 404，而非應用程式外殼，因此過期頁面無法以指令碼 URL 快取 HTML。",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "技術架構",
      title: "前端",
      lede: "在用戶端算繪的 SolidJS 2 應用程式，編譯為靜態資源並內嵌於伺服器二進位檔。",
      sections: [
        {
          id: "stack",
          heading: "技術堆疊",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "應用程式", items: ["SolidJS 2", "Solid Router 2", "TypeScript", "Vite 8"] },
                { label: "樣式", items: ["Tailwind CSS 4", "IBM Plex Sans", "IBM Plex Mono", "OKLCH"] },
                { label: "元件", items: ["Toast UI Editor", "Leaflet", "Chart.js", "highlight.js", "DOMPurify"] },
                { label: "測試", items: ["Vitest", "Playwright", "axe-core"] },
              ],
            },
            {
              kind: "paragraph",
              text: "Solid 的細粒度訊號只更新依賴於已變更值的 DOM 節點，其 2.0 候選版本可讓非同步讀取暫停至最近的載入邊界。路由器對每個頁面都採延遲載入，正式環境中沒有 JavaScript 伺服器：靜態檔案由 Rust 二進位檔直接提供。",
            },
          ],
        },
        {
          id: "budget",
          heading: "載入預算",
          blocks: [
            {
              kind: "list",
              items: [
                "一個 Vite 外掛從進入點區塊開始追蹤靜態匯入，若初始指令碼與樣式的 gzip 大小超過 **130 KiB**，建置即失敗。",
                "初始區塊只包含英文介面文字。其他語言以及各語言的關於頁面都是獨立的區塊，在選取時才載入。",
                "在節流條件下的行動裝置效能測試目標為：LCP 不超過 2.5 s、互動不超過 200 ms、CLS 不超過 0.1。",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "在地化",
          blocks: [
            {
              kind: "paragraph",
              text: "介面支援英文、韓文、法文、西班牙文、簡體中文、繁體中文、日文與德文。約 650 個鍵的目錄會與後端的註冊表比對驗證，預留位置在翻譯後也必須保留。這些關於頁面是具型別的資料模組，因此譯文不可能在無聲無息間漏掉英文版中的章節。",
            },
          ],
        },
        {
          id: "design",
          heading: "設計系統",
          blocks: [
            {
              kind: "paragraph",
              text: "整體風格溫暖而具技術感：米色紙張或純黑底色、細線邊框、技術內容使用等寬字型，並且一次只用一種強調色。色彩皆為語意化 CSS 變數，因此元件不需要另外的深色模式變體。強調色預設集以 OKLCH 定義，可在不同色相間維持一致的感知亮度；淺色或深色模式與強調色都會在首次繪製前套用。",
            },
          ],
        },
        {
          id: "motion",
          heading: "動態效果",
          blocks: [
            {
              kind: "list",
              items: [
                "章節顯現效果與閱讀進度列使用 CSS 捲動驅動動畫，在支援的瀏覽器中於主執行緒之外執行。",
                "統計數字透過註冊的 `@property` 整數遞增顯示，最終數值保留在標記中，供輔助技術讀取。",
                "主題切換透過 View Transitions API 從切換鈕向外展開。",
                "首頁的等高線背景是一個小型 WebGL 著色器，在頁面閒置後載入，僅在效能足夠的桌面瀏覽器上執行，限制影格率，並在離開畫面時暫停。",
                "當系統要求減少動態效果時，所有裝飾性效果都會略過。",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "技術架構",
      title: "子系統",
      lede: "本站中超出部落格嚴格所需、更有意思的部分。",
      sections: [
        {
          id: "live-chat",
          heading: "即時聊天與通話",
          blocks: [
            {
              kind: "paragraph",
              text: "聊天透過 WebSocket 運作，使用精簡的二進位編解碼器、內容審核與有上限的歷史記錄。語音與視訊通話經過一個以 webrtc-rs 建構、在 Axum 程序內執行的選擇性轉送單元（SFU）：每個瀏覽器只維持一條對等連線，SFU 透過依發布劃分的有上限通道轉送已編碼媒體而不轉碼，並限制參與人數。[開啟即時聊天](/live-chat)。",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Minecraft 地形探索器",
          blocks: [
            {
              kind: "paragraph",
              text: "一個 Paper 外掛開放僅限同一使用者存取的 UNIX socket，用於有上限的世界查詢，一次只處理一項操作，並嚴格限制訊框大小。[探索器](/minecraft)以自己的 Leaflet 用戶端繪製 squaremap 圖磚，而不執行所產生地圖的 JavaScript。",
            },
            {
              kind: "paragraph",
              text: "生態域疊加層使用 **CYBM**（一種以 gzip 壓縮傳輸的精簡二進位圖磚格式），或在私有 `cyBM` 區塊中攜帶精確生態域識別資訊的 PNG，因此游標停留時從不需要依像素顏色猜測。未探索地形的種子碼預測來自一個以 Pumpkin 建構的獨立世界生成工作程序。",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Geo-IP 查詢",
          blocks: [
            {
              kind: "paragraph",
              text: "[訪客地圖](/visitor-board)與 [IP 查詢](/geo-ip-db)不需呼叫外部 API 即可定位位址。一個前處理工具將 IP2Location LITE CSV 檔案轉換為以範圍起始位址為鍵的 `BTreeMap`，以 bitcode 編碼，再以 zstd 第 22 級壓縮。",
            },
            {
              kind: "table",
              columns: ["資料集", "CSV", "打包檔"],
              numericColumns: [1, 2],
              rows: [
                ["IPv4", "312.5 MB", "24.9 MB"],
                ["IPv6", "794.2 MB", "42.4 MB"],
              ],
            },
            {
              kind: "paragraph",
              text: "啟動時，伺服器一次只解碼一個打包檔以控制記憶體峰值，並對重複出現的地名進行字串駐留。查詢時先找出起始位址不大於目標位址的最後一個範圍，再檢查其結束邊界。",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "上傳的[展示套件](/projects)會先暫存至磁碟，以魔術位元組識別格式，解壓縮後限制在 50 MiB 以內，並從有上限的快取提供給沙箱框架。工作區也會建置 Rust 展示，包括打磚塊遊戲與光線追蹤器。",
            },
            {
              kind: "paragraph",
              text: "[EU5 Location DB](/eu5-locations-db) 是編譯為 WebAssembly 的 Slint 應用程式。頁面透過一個小型訊息協定傳遞淺色或深色模式，因此內嵌應用程式的配色會隨網站切換。",
            },
          ],
        },
        {
          id: "photography",
          heading: "攝影處理流程",
          blocks: [
            {
              kind: "paragraph",
              text: "上傳的影像在固定的像素上限內解碼，並以最慢的預設編碼為 AVIF，用處理時間換取更小的檔案。EXIF 資料中的位置可在發布前於地圖上確認，同時執行的處理工作不超過兩項。",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "技術架構",
      title: "效能",
      lede: "對後端進行設定檔引導最佳化與 BOLT 最佳化，並以不會成為瓶頸的負載產生器量測。",
      sections: [
        {
          id: "approach",
          heading: "方法",
          blocks: [
            {
              kind: "paragraph",
              text: "最佳化建置以具代表性的工作負載進行訓練。插樁後的二進位檔針對一個拋棄式測試夾具記錄效能剖析資料，該夾具提供模擬的 PostgreSQL、SMTP、S3、OpenID Connect 與 Minecraft 端點。LLVM 以這些剖析資料重新建置伺服器（PGO），接著 BOLT 依取樣的執行情形重新排列最終二進位檔的程式碼配置。所有變體都使用 LTO、框架指標與重新建置的標準函式庫。",
            },
          ],
        },
        {
          id: "correction",
          heading: "量到自己的效能測試",
          blocks: [
            {
              kind: "paragraph",
              text: "第一輪測試由 Playwright 產生負載，結果看似出現效能退化。但這些退化並不真實：未限定範圍的 Playwright API 呼叫會搜尋先前的測試步驟來尋找父項，因此用戶端隨著執行推進而越來越慢。即使面對一個極簡的 HTTP 伺服器，光是用戶端本身就在九個樣本中從 2,684 降到 1,314 requests/s。",
            },
            {
              kind: "paragraph",
              text: "負載產生改用 oha。Playwright 仍會檢查每條路由的回應，但在計時區間之外進行。",
            },
          ],
        },
        {
          id: "setup",
          heading: "測試環境",
          blocks: [
            {
              kind: "list",
              items: [
                "伺服器、測試夾具提供者與 PostgreSQL 綁定至 AMD Ryzen 7 9800X3D 的四個核心；負載用戶端綁定至另外四個核心。",
                "透過集區化的 HTTP/1.1 使用 HTTPS，共 32 條連線，完整讀取回應，逾時期限為兩秒。",
                "輪替順序進行三輪，每個變體五個樣本，每個樣本前先送出 10,000 個暖機請求，並從全新的資料庫快照開始。",
                "計時請求共 27,000,000 個，全部回傳 HTTP 200。",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "結果",
          blocks: [
            {
              kind: "bars",
              caption: "每個變體 15 個樣本的每秒請求數中位數。",
              groups: [
                {
                  label: "十條路由的公開讀取",
                  items: [
                    { label: "基準", value: 44874.12, display: "44,874" },
                    { label: "PGO", value: 52925.46, display: "52,925" },
                    { label: "PGO + BOLT", value: 53888.61, display: "53,889", emphasis: true },
                  ],
                },
                {
                  label: "小型健康檢查 JSON",
                  items: [
                    { label: "基準", value: 272837.41, display: "272,837" },
                    { label: "PGO", value: 308685.26, display: "308,685", emphasis: true },
                    { label: "PGO + BOLT", value: 306276.64, display: "306,277" },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "十條路由混合負載的尾端延遲。",
              columns: ["變體", "p95 (ms)", "p99 (ms)"],
              numericColumns: [1, 2],
              rows: [
                ["基準", "2.380", "2.967"],
                ["PGO", "2.020", "2.524"],
                ["PGO + BOLT", "1.987", "2.483"],
              ],
            },
            {
              kind: "paragraph",
              text: "PGO 使混合吞吐量提升 **17.9%**，PGO 搭配 BOLT 提升 **20.1%**，兩種最佳化建置在每一輪輪替中都勝出。在混合讀取上，BOLT 比 PGO 再提升 1.8%，而在極簡的健康檢查路由上慢了 0.8%。這些是四核心的結果；正式環境的容量還取決於網路、資料庫工作集與實際流量。",
            },
          ],
        },
        {
          id: "transport",
          heading: "小修正也很重要",
          blocks: [
            {
              kind: "paragraph",
              text: "axum-server 預設的 Rustls 接受器沒有設定 `TCP_NODELAY`，因此拆成標頭訊框與資料訊框的 HTTP/2 回應可能需要等待延遲確認。在每個已接受的 socket 上啟用後，透過重複使用的連線傳輸的 59,571 位元組圖磚回應變化如下。",
            },
            {
              kind: "table",
              columns: ["TCP_NODELAY", "協定", "第一個位元組時間中位數", "完成時間中位數"],
              numericColumns: [2, 3],
              rows: [
                ["關閉", "HTTP/2", "0.290 ms", "40.981 ms"],
                ["開啟", "HTTP/2", "0.113 ms", "0.152 ms"],
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
