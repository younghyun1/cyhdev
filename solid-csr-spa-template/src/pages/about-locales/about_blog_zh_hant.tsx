import { RepositorySourceMap } from "../../components/RepositorySourceMap";
import { pageStyles } from "../../styles/pageStyles";

export default function AboutBlog() {
  return (
    <main lang="zh-Hant" class={pageStyles.page}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">
            部落格技術架構
          </h1>
          <p class="text-xs text-ink-muted">
            最後更新：2026-08-30
            <br />
          </p>
        </div>

        <section class="space-y-8">
          {/* 0) On-Prem */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              0) 主機、作業系統、檔案系統與網路設定
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              本站託管於我住處的一台迷你伺服器，透過 1Gbps 有線 Xfinity 網路連線。
              {" "}
              <a
                href="https://store.minisforum.com/products/minisforum-um690l-slim"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                主機
              </a>{" "}
              主機配備最高運作頻率 4.9GHz 的八核心 Ryzen 9 行動處理器、32GB 6400MT/s
              記憶體及 1TB NVMe SSD。約 400 美元的價格非常划算；相較於一年左右支付同等
              金額給 AWS 卻只換得效能弱得多的硬體，更顯如此，也反映了我對自我託管的熱愛。
              它執行整合式前後端伺服器、postgreSQL 資料庫及 Minecraft 伺服器。想來玩的話，
              寄封電子郵件給我。


            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-1) Gentoo
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Gentoo 並非企業伺服器常見的作業系統選擇；若在公司工作，我大概會採用 Debian
              Stable、ext4 檔案系統及一般資料庫引擎。不過我個人很喜歡動手調校並自行編譯
              軟體，也喜歡為 CPU 架構最佳化套件的建置與安裝，這些都仰賴優秀的


              {" "}
              <a
                href="https://www.gentoo.org/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                Gentoo
              </a>{" "}
              專案。
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-2) 在資料庫、前後端與 Minecraft 主機上使用 btrfs
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
              是具備快照、CoW、壓縮等進階功能的現代檔案系統。然而，由於 CoW 會造成大量
              碎片化，它並非資料庫主機的最佳選擇；我已將 PostgreSQL 與 Minecraft 資料目錄
              排除於 CoW 之外，來減輕此問題。

              {" "}
              <a
                href="https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                一項 PostgreSQL 檔案系統效能測試
              </a>{" "}
              顯示預設狀態的 btrfs 效能不佳；不過我猜停用 CoW 後，表現可能足以媲美 ext4
              與 xfs。這會是個有趣的基準測試。


            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-3) 內外部網路設定
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              申請 Xfinity 的 2Gbps 有線電視網路服務時，我沒想到他們竟會省到提供一台實際上 <em>不</em>
              支援 2.5Gbps Ethernet 的數據機兼路由器。因此只能用 1Gbps，不過我想不到這
              會如何影響我的小網站。Route 53 為我的網域提供 DNS 服務。



              <br />
              <br />
              在內部網路中，確實沒有理由使用反向代理、容器化或分散式服務工具；作業系統上
              只跑著一般的 postgreSQL 引擎，一個 Rust 二進位程式同時擔任前端與 API 伺服器，
              並透過 UNIX socket 連線至資料庫，實際速度遠勝於

              {" "}
              <a
                href="https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                如這裡所述，直接使用 localhost。
              </a>{" "}
              TPS 可接近翻倍，而且略過多餘的網路堆疊也能讓延遲減半；我觀察到資料庫與伺服器
              之間的延遲最低約 150 微秒。使用 localhost 通常約為 600 微秒。這種情境與雲端
              企業服務設定無關，不過挺有意思。老派作法，速度更快！2025 年 11 月推出的
              PostgreSQL 18 也以 io_uring 支援的形式引入非同步 I/O，目前已啟用，大幅加快讀取速度。
              {" "}
              <a
                href="https://pganalyze.com/blog/postgres-18-async-io"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                io_uring 支援
              </a>{" "}
            </p>
          </section>

          {/* 1) PostgreSQL */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              1) 資料
            </h2>
            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-1) PostgreSQL 18
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              在韓國的新創與企業職涯中，我注意到一種文化，認為 MySQL 或 MariaDB 才是值得
              使用的 RDBMS，但我始終不明白原因。幾位資深主管告訴我，過去 PostgreSQL
              完全不被視為正經選項。這很不可思議，因為過去十多年來，它在效能、擴充性、
              資料型別與工具方面大幅進步，在許多面向甚至可說已超越 MySQL，尤其是 UUID
              資料型別支援與 JSON 資料的二進位編碼。




            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-2) Schema 重點（部落格與驗證）
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Schema 刻意保持樸實，這是好事：使用者、記憶體中的工作階段、電子郵件驗證權杖、
              密碼重設權杖、文章、留言、投票資料表、標籤、個人檔案圖片，以及少數地理與
              國際化支援資料表。部落格並非只有一張
              <code>posts</code> 資料表再祈禱一切順利。文章包含 slug、摘要、JSONB 中繼資料、發布狀態、
              反正規化計數器及標籤對應。留言透過可為空值的父留言 ID 串成討論串。文章與
              留言投票分別存放於專用資料表，讓查詢路徑更簡單，也避免日後出現荒謬的條件
              判斷。



              <br />
              <br />
              驗證設計同樣務實：使用者紀錄會保存國家與語言，讓網站不只收下電子郵件就
              忘記你的存在。我不想讓授權設計陷入死角，因此設有角色與權限資料表。個人
              檔案圖片以版本形式存於獨立資料表，而非硬塞進使用者資料列；這讓替換圖片的
              邏輯更乾淨，也避免無關欄位拖累常用的使用者資料列。



            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-3) UUIDv7
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              幾乎所有面向使用者的資料都以 UUID 為鍵，這不是偶然。我不喜歡暴露連號 ID，
              讓任何人都能推測資料列數量，或像 2009 年那樣列舉資源。按時間排序的 UUID
              也比完全隨機的 UUIDv4 更適合索引，尤其在寫入量並非紙上談兵時更有幫助。
              換句話說：全域唯一、難以猜測，也更利於資料區域性。新增資料的模式更穩定、
              B-tree 變動較少，資料庫也不必花太多時間做可避免的維護。




            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-4) Diagram (request + data path)
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              這是一張非常華麗的企業架構圖；我還沒畫方框，所以先以文字呈現：

              <br />
              <br />
              瀏覽器請求 -&gt; Axum 路由器 -&gt; 中介層鏈（記錄、驗證／工作階段查詢、
              速率限制、本文大小檢查）-&gt; 處理常式 -&gt; Diesel 非同步查詢或記憶體快取
              查詢 -&gt; 經 UNIX socket 連線至 PostgreSQL -&gt; DTO 回應 -&gt; 壓縮後的
              HTTPS 回應傳回瀏覽器。

              <br />
              <br />
              靜態資源走的路徑更短。建置完成的 SolidJS 應用程式直接嵌入 Rust 二進位檔，
              若瀏覽器支援，便透過內容協商提供 zstd/gzip 壓縮版本。正式環境沒有常駐的 Node
              程序，也不需獨立的靜態檔案主機；從「收到請求」到「送出位元組」之間沒有
              額外轉送。

            </p>
          </section>

          {/* 2) Backend */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              2) 後端（Rust）
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              後端是以 Rust、Axum、Tokio、Diesel 與 PostgreSQL 建構的服務，並以 mimalloc 作為
              全域配置器。既然自我託管的機器配有真正的多核心，就該稍微認真看待配置器
              行為。服務以 rustls 直接終止 TLS、提供內嵌 SPA、提供驗證／部落格／攝影／
              國際化的 JSON API，也透過 WebSockets 將主機統計資料推送至伺服器儀表板。路由器
              包含請求壓縮、目前較寬鬆的 CORS 設定、相當寬裕的速率限制，以及正式環境中
              僅限驗證使用者存取的 Swagger UI。




              <br />
              <br />
              在應用程式層面，系統採單一程序搭配共用伺服器狀態物件、記憶體內工作階段
              管理、啟動時快取同步及排程維護工作。驗證使用安全的 HTTP-only cookie、電子
              郵件驗證、密碼重設權杖，並明確檢查超級使用者操作的角色權限。更有意思的是
              延遲表現：資料庫流量留在 UNIX socket 上，連線池依實體核心數調整大小，部落格
              清單讀取大多來自快取，回應組裝則以批次方式補齊資料，不會退化成一堆零碎
              查詢。這讓整個堆疊以我在意的方式保持快速：更少跳轉、更少複製、更少往返，
              也不必等抽象層自我慶賀完畢。






            </p>
          </section>

          {/* 3) 前端 */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              3) 前端
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              前端是使用 Vite 與 TypeScript 建置的用戶端 SolidJS SPA。路由採延遲載入，狀態
              管理維持簡潔，整個應用程式會編譯為靜態資源，部署時再打包進 Rust 二進位檔。
              因此正式環境不需照料 JavaScript 伺服器程序；Rust 伺服器只要盡可能有效率地
              提供靜態檔案即可。



              <br />
              <br />
              部落格介面兼顧實用，也不使用玩具級編輯器。Markdown 撰寫由 Toast UI Editor
              負責；貼上或上傳的圖片會直接透過攝影上傳 API 處理，讓編寫文章不再令人
              痛苦。搜尋支援標題查詢及標籤，頁面可透過查詢參數導覽；驗證狀態在用戶端
              追蹤，但由伺服器端強制執行；應用程式也會檢查回應標頭，以顯示伺服器建置資訊。
              樣式以 Tailwind 搭配共用頁面樣式系統建構。Solid 的執行期負擔輕，DOM 變動
              也很少，這正是我對 UI 層的期待：做好本分，不妨礙使用者。





            </p>
          </section>

          {/* 4) Network & HTTPS */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              4) HTTPS、路由與安全防護
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              HTTPS 由 Rust 伺服器透過 rustls 直接處理。純 HTTP 請求會重新導向 HTTPS，
              cookie 會設為 secure 與 HTTP-only，正式環境的 cookie 網域也限制在本站網域，
              不會任意擴散。支援時會以 zstd 或 gzip 提供靜態資源；SPA 備援路由讓深層連結
              正常運作，不需透過獨立的反向代理層。



              <br />
              <br />
              安全防護並不花俏，但該有的都有：上傳請求本文大小限制、中介層記錄、受保護
              路由的驗證門檻、敏感端點的超級使用者檢查、降低濫用風險的速率限制，以及
              供用戶端請求使用的 API 金鑰。更重要的是，部署型態本身簡單易懂：一個二進位
              檔、一個資料庫、一台主機，應用程式直接處理 TLS，也幾乎沒有神祕延遲或設定
              漂移的藉口。這種架構或許不時髦，但快速、可觀測，而且持續維持低額外負擔。




            </p>
          </section>

          <RepositorySourceMap />
        </section>
      </section>
    </main>
  );
}
