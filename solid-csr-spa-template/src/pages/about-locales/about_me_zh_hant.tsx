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
    <main lang="zh-Hant" class={`${pageStyles.page} about-page`}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">關於我</h1>
          <p class="text-xs text-ink-muted">
            最後更新：2026 年 9 月
          </p>
        </div>

        <section class="mb-12">
          <h2 class="text-xl font-bold mb-4 border-b border-line pb-2">
            1. 簡介
          </h2>

          <div class="mb-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div>
              <h3 class="text-lg font-semibold">
                Young Hyun Chi / 지영현 / 池營賢 / 池营贤
              </h3>

              <p class="text-ink-muted">
                後端與基礎設施工程師 · Rust / 平台可靠性 / 資訊安全
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
                  alt="個人肖像"
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
            <h3 class="font-bold mb-2">聯絡方式</h3>
            <ul class="list-none space-y-2 text-sm">
              <li>
                <span class="w-20 inline-block font-medium text-ink-muted">
                  電子郵件：
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
            我是一名常駐科羅拉多州的後端工程師，專精於 Rust 服務、
            PostgreSQL 資料庫與雲端基礎設施。在 Soundpatrol，我負責
            網路安全、DevOps、網路服務與開發者工具，使用 Rust、
            Python、PostgreSQL、Kubernetes 與 GCP。
          </p>
          <p class="mb-4 leading-relaxed">
            過去的專案包括整合數千個環境感測器、部署於 AWS 的 Samsung C&amp;T 數位孿生系統；
            運用字幕、留言與語言模型的 YouTube 頻道分析平台；
            以及 Hyundai、Kia、Genesis 應用程式與某 K-pop 團體粉絲應用程式的後端及資料
            管線。
          </p>
          <p class="mb-4 leading-relaxed">
            我相信服務應善用現代硬體與程式語言。我喜歡深入處理並行、資料庫查詢、
            記憶體使用與部署等細節，而非把雲端資源視為工程能力的無限替代品。
            我也喜歡自我託管、打造 Rust 工具，並協助同事學習 Rust。


          </p>
          <p class="mb-4 leading-relaxed">
            我很珍惜曾跨足不同領域、與多元文化背景的同事共事的經歷。2016 至 2017 年，
            我在朝韓邊境附近的美國陸軍第 2 步兵師服役 21 個月，與美韓軍方人員共事。
            大學期間，我從事過新聞、攝影與翻譯，也做過夜班倉儲及物流工作。
            自 2023 年起，我的職涯專注於後端工程、資料管線及維持其運作的基礎設施。



          </p>
        </section>

        {/* 2) Professional Career */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            2. 軟體工程職涯
          </h2>

          <div class="space-y-8">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Soundpatrol</h3>
                <span class="text-sm font-mono text-ink-muted">美國 · 遠端 | 2026 年 5 月－至今</span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">Rust 工程師</p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>使用 Rust 與 Python 開發網路服務及內部工具。</li>
                <li>維護雲端基礎設施、Kubernetes 部署及 CI/CD 管線。</li>
                <li>負責監控、存取控制、憑證管理及其他網路安全措施。</li>
              </ul>
            </div>

            {/* GenesisNest */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">GenesisNest</h3>
                <span class="text-sm font-mono text-ink-muted">
                  韓國京畿道城南市 | 2025 年 1 月－2025 年 7 月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                軟體工程師
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  為內部服務及 K-pop 團體官方應用程式開發基礎設施管理工具，並熟悉
                  Spring Boot 生態系。

                </li>
                <li>
                  以承包工程師身分參與 Hyundai Motor Company 的 Hyundai/Kia/Genesis
                  官方應用程式後端，並加入大型 Java 程式碼庫。
                </li>
                <li>
                  實作新 API 並修復錯誤，與供應商協調，支援亞洲及歐洲數千萬使用者所用的
                  服務。

                </li>
                <li>
                  為其國際化工作建立資料管線，撰寫多個 Rust 與 Python 腳本，自動將翻譯內容
                  整合至 Hyundai 資料庫；並與處理大型工廠系統相關資料集的企業 DBA 協作。



                </li>
              </ul>
            </div>

            {/* pampam */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">pampam Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  韓國首爾 | 2024 年 11 月－2024 年 12 月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                約聘軟體工程師
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  為 AI YouTube 分析平台設計 PostgreSQL schema 與 Rust 後端，並主導在 AWS
                  及 GCP 上的部署工作。

                </li>
                <li>
                  主導透過 YouTube API 大規模蒐集資料（數千萬則留言及使用者）。

                </li>
                <li>
                  運用大型語言模型及伺服器端開源模型產生洞察與摘要。

                </li>
              </ul>
            </div>

            {/* EAN Technology */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">EAN Technology Co. Ltd</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Seoul, South Korea | Aug 2023 - Aug 2024
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                後端軟體工程師（主管）
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>Samsung C&T 數位孿生專案：</b> 淘汰失效的微服務架構，改以高度最佳化的 Axum（Rust）單體架構取代，
                  每月雲端費用由約 5,000 美元降至約 150 美元。


                </li>
                <li>
                  貢獻約 30,000 行程式碼（約佔 75%），並實作 80 個含複雜領域邏輯的端點。

                </li>
                <li>
                  在九個月的正式環境運行期間，達成零次執行期停機及約 90% 的錯誤處理
                  涵蓋率。
                </li>
                <li>
                  整合數千個即時實體感測器（BACnet、Modbus）及 Samsung C&amp;T 總部的
                  內部部署伺服器。
                </li>
                <li>
                  最佳化 PostgreSQL 查詢與 schema，在受限硬體（2 核心、4GB RAM）上將 P99
                  延遲降至數十毫秒；先前部分未最佳化查詢需耗時數秒。

                </li>
                <li>
                  使用 GitHub Runners、AWS CodeDeploy 與 Docker 建置 CI/CD。透過 MUSL、
                  靜態連結及多階段建置與 scratch 映像，產生約 20MB 的 Docker 映像，提升
                  安全性並縮短冷啟動時間。

                </li>
                <li>
                  指導同事學習 Rust，協助他們在兩個月內從 Node.js/Java 轉換過來。

                </li>
              </ul>
            </div>

            {/* Artifyc */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Artifyc Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  美國德州奧斯汀（遠端）| 2022 年 8 月－2023 年 3 月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                軟體工程實習生
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  使用 Python 與 Reddit API 實作 Discord 機器人，擷取並發布藝術委託需求。

                </li>
                <li>
                  在 AWS Lambda 上開發無伺服器後端邏輯，用於全球貨幣換算與附加費計算。

                </li>
                <li>
                  協助修改 React 前端並整合素材。
                </li>
              </ul>
            </div>
          </div>
        </section>


        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">精選專案</h2>
          <ul class="list-disc list-inside text-sm space-y-3 text-ink">
            <li><a href="https://github.com/younghyun1/sillok" class={pageStyles.link}>Sillok</a>：以 Rust 撰寫的結構化工作紀錄 CLI，具備僅附加歷史記錄、索引化 Turso/SQLite 投影、修正功能，以及透過 Git 在多台電腦間同步。</li>
            <li><a href="https://github.com/younghyun1/eu5-location-filter" class={pageStyles.link}>EU5 Location DB</a>：以 Rust 與 Slint 打造的桌面及 WebAssembly 應用程式，內嵌壓縮地圖資料、預先計算索引、可搜尋篩選器及可調整大小的表格。</li>
            <li><a href="https://github.com/younghyun1/cyhdev" class={pageStyles.link}>cyhdev.com</a>：此網站採用 Rust/Axum、PostgreSQL 與 SolidJS，提供發文、攝影、討論、聊天及瀏覽器展示功能。</li>
            <li><a href="https://github.com/younghyun1/oohid" class={pageStyles.link}>oohid</a>：以 Rust 撰寫的 UUID 產生器，支援自訂格式、輸出至檔案及選擇性重複檢查。</li>
          </ul>
        </section>

        {/* 3) Non-IT Career */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            3. 非 IT 職涯
          </h2>

          <div class="space-y-6">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">
                  韓國海洋科學技術院
                </h3>
                <span class="text-sm font-mono text-ink-muted">
                  韓國首爾 | 2023 年 6 至 7 月、2024 年 8 月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                筆譯／口譯
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  為挪威大使及東遠集團董事長參與的高階會議提供英語、韓語及挪威語口譯。


                </li>
                <li>
                  受託處理高度敏感的國家及商業交易細節，以及與海事研發相關的討論。


                </li>
                <li>
                  為美國能源部活動指導 KIMST 人員及主管。

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
                    美國陸軍
                  </a>{" "}
                  (
                  <a
                    href="https://en.wikipedia.org/wiki/2nd_Infantry_Division_(United_States)"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    第 2 步兵師
                  </a>
                  ) /{" "}
                  <a
                    href="https://en.wikipedia.org/wiki/Republic_of_Korea_Army"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    韓國陸軍
                  </a>
                </h3>
                <span class="text-sm font-mono text-ink-muted">
                  <a
                    href="https://en.wikipedia.org/wiki/Dongducheon"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    東豆川
                  </a>
                  ，韓國 | 2016 年 3 月－2017 年 12 月
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
                | | 教官 | 中士
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  擔任聯絡兵及教官，為約 6,000 名美韓人員提供訓練並辦理行政報到。


                </li>
                <li>
                  教授體能、文化、歷史及安全／預防課程。

                </li>
                <li>
                  在期間維持作戰整備；並帶領一支 13 人小隊。{" "}
                  <a
                    href="https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    2017 crisis
                  </a>

                </li>
              </ul>
            </div>

            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Coupang Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  韓國 | 2020－2022 年
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                裝卸員
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  夜班期間在物流倉庫從事高強度體力工作。

                </li>
              </ul>
            </div>
          </div>
        </section>

        {/* 4) Academics */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            4. 學歷
          </h2>

          <div class="mb-6">
            <div class="flex flex-wrap justify-between items-baseline mb-2">
              <h3 class="text-lg font-bold">Sungkyunkwan University</h3>
              <span class="text-sm font-mono text-ink-muted">
                韓國水原 | 2015 年 3 月－2023 年 8 月
              </span>
            </div>
            <p class="text-sm font-medium mb-2 text-ink-muted">
              B. Eng in 軟體工程師ing
            </p>
            <div class="space-y-2 text-sm text-ink">
              <p>
                修畢涵蓋電腦科學、網頁與嵌入式軟體工程及電子工程的完整課程，著重邏輯、
                低階語言、作業系統、網路工程及軟體工程實務。


              </p>
              <p>
                成均館大學是韓國歷史最悠久的大學，於 1398 年創立，當時是朝鮮王朝依儒家
                傳統設立的國家官僚學院。韓國光復後，該校一直是韓國主要的學術與歷史機構
                之一；自我就讀以來也持續名列世界百大大學。



              </p>
            </div>
          </div>

          <div>
            <h4 class="font-bold mb-2 text-md">出版品</h4>
            <ul class="list-disc list-inside text-sm space-y-2 text-ink">
              <li>
                <b>
                  以韓語 NLP 模型對社群媒體社群進行情緒分析

                </b>{" "}
                （IMCOM 2023，IEEE Xplore）。
                <br />
                <span class="ml-5 block text-xs text-ink-muted mt-1">
                  使用 KoBERT 分析 COVID 疫情初期的社群媒體資料情緒，並獲畢業專題銅獎。


                </span>
              </li>
            </ul>
          </div>
        </section>

        {/* 5) Volunteer work and interests */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            5. 志工服務與興趣
          </h2>

          <div class="grid md:grid-cols-2 gap-8">
            <div>
              <h3 class="font-bold mb-3 text-lg">個人專案</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>
                  <b>oohid：</b>以 Rust 撰寫的 UUIDv4 命令列產生器，速度約為 libuuid 的 3 倍，支援重複檢查及 Python/JSON 格式。
                </li>
                <li>
                  <b>impulsr：</b> 用於行銷摘要的 YouTube 轉錄與留言蒐集工具。
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-3 text-lg">志工服務</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>在社區中心教授英文。</li>
                <li>在由身心障礙者經營的二手商店擔任志工。</li>
              </ul>
            </div>
          </div>
        </section>

        {/* 6) Hobbies */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            6. 興趣
          </h2>

          <div class="space-y-4">
            <div>
              <h3 class="font-bold inline mr-2">攝影</h3>
              <span class="text-sm text-ink-muted">
                自 2010 年起拍攝風景與人像的業餘攝影師，已典藏約 30,000 張照片。
              </span>
              <div class="mt-2">
                <a
                  href="/photographs"
                  class={`inline-flex items-center ${pageStyles.link} font-medium`}
                >
                  瀏覽攝影作品集 →
                </a>
              </div>
            </div>

            <div>
              <h3 class="font-bold inline mr-2">新聞工作</h3>
              <p class="text-sm text-ink mt-1">
                曾任《Sungkyun Times》學生記者。
              </p>
            </div>
          </div>
        </section>

        {/* 7) Qualifications and Other Skills */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            7. 資格與其他技能
          </h2>

          <div class="grid md:grid-cols-2 gap-8 mb-8">
            <div>
              <h3 class="font-bold mb-2">資格與獎項</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>TOEFL:</b> 117/120 (CEFR C2)
                </li>
                <li>
                  <b>IELTS:</b> 8.5/9.0 (CEFR C2)
                </li>
                <li>
                  <b>第 7 名：</b> 2021 年 Capstone Design &amp; Idea Hackathon
                </li>
                <li>
                  <b>第 3 名：</b> 2020 年 Fintech Hackathon（SNUST）
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-2">語言</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>韓語（母語）</li>
                <li>英語（C2）</li>
                <li>初階法語、德語、西班牙語、華語</li>
              </ul>
            </div>
          </div>
        </section>
      </section>
    </main>
  );
}
