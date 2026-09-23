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
    <main lang="ja-JP" class={`${pageStyles.page} about-page`}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">プロフィール</h1>
          <p class="text-xs text-ink-muted">
            最終更新: 2026年9月
          </p>
        </div>

        <section class="mb-12">
          <h2 class="text-xl font-bold mb-4 border-b border-line pb-2">
            1. はじめに
          </h2>

          <div class="mb-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div>
              <h3 class="text-lg font-semibold">
                Young Hyun Chi / 지영현 / 池營賢 / 池营贤
              </h3>

              <p class="text-ink-muted">
                バックエンド・インフラエンジニア · Rust / プラットフォーム信頼性 / セキュリティ
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
                  alt="プロフィール写真"
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
            <h3 class="font-bold mb-2">連絡先</h3>
            <ul class="list-none space-y-2 text-sm">
              <li>
                <span class="w-20 inline-block font-medium text-ink-muted">
                  メール:
                </span>
                <a href="mailto:younghyun1@gmail.com" class={pageStyles.link}>
                  younghyun1@gmail.com
                </a>
              </li>
              <li>
                <span class="w-20 inline-block font-medium text-ink-muted">
                  GitHub:
                </span>
                <a href="https://github.com/younghyun1" class={pageStyles.link}>
                  github.com/younghyun1
                </a>
              </li>
              <li>
                <span class="w-20 inline-block font-medium text-ink-muted">
                  LinkedIn:
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
            コロラド州を拠点とするバックエンドエンジニアで、Rust サービス、PostgreSQL データベース、クラウドインフラを専門としています。Soundpatrol では Rust、Python、PostgreSQL、Kubernetes、GCP を使い、サイバーセキュリティ、DevOps、Web サービス、開発者向けツールに取り組んでいます。
          </p>
          <p class="mb-4 leading-relaxed">
            これまでに、Samsung C&amp;T 向けに数千台の環境センサーを統合した AWS 上のデジタルツイン、文字起こし・コメント・言語モデルを活用する YouTube チャンネル分析基盤、Hyundai、Kia、Genesis のアプリや K-pop グループのファンアプリのバックエンドとデータパイプラインを手がけました。
          </p>
          <p class="mb-4 leading-relaxed">
            最新のハードウェアとプログラミング言語を効果的に活用するサービスづくりを大切にしています。クラウド資源を無制限に使うのではなく、並行処理、データベースクエリ、メモリ使用量、デプロイの細部まで考えるのが好きです。セルフホスティングや Rust ツールの開発、同僚が Rust を学ぶ手助けも楽しんでいます。
          </p>
          <p class="mb-4 leading-relaxed">
            さまざまな分野で働き、多様な文化や背景を持つ同僚と仕事をしてきたことを誇りに思います。2016年から2017年にかけて21か月間、北朝鮮との国境近くで米陸軍第2歩兵師団に所属し、アメリカと韓国の軍人と共に勤務しました。大学時代には報道、写真、翻訳に携わり、夜勤の倉庫・物流業務も経験しました。2023年以降はバックエンドエンジニアリング、データパイプライン、およびそれらを支えるインフラを中心に活動しています。
          </p>
        </section>

        {/* 2) 職務経歴 */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            2. 開発者としての経歴
          </h2>

          <div class="space-y-8">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Soundpatrol</h3>
                <span class="text-sm font-mono text-ink-muted">米国 · リモート | 2026年5月 - 現在</span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">Rust エンジニア</p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>Rust と Python で Web サービスと社内ツールを開発。</li>
                <li>クラウドインフラ、Kubernetes のデプロイ、CI/CD パイプラインを保守。</li>
                <li>監視、アクセス制御、認証情報管理などのサイバーセキュリティ対策を担当。</li>
              </ul>
            </div>

            {/* GenesisNest */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">GenesisNest</h3>
                <span class="text-sm font-mono text-ink-muted">
                  韓国・城南 | 2025年1月 - 2025年7月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                ソフトウェアエンジニア
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  社内サービスと K‑Pop グループ公式アプリ向けのインフラ管理ツールを開発。Spring Boot エコシステムの知識を深めました。
                </li>
                <li>
                  Hyundai Motor Company の Hyundai/Kia/Genesis 公式アプリのバックエンド開発を受託し、大規模な Java コードベースに参加。
                </li>
                <li>
                  アジアとヨーロッパで数千万人が利用するサービスについて、ベンダーと連携しながら API の新規実装と不具合修正を担当。
                </li>
                <li>
                  国際化対応のデータパイプラインを構築し、翻訳コンテンツを Hyundai のデータベースへ取り込む処理を自動化する Rust と Python のスクリプトを作成。工場システムに接続された大規模データセットを扱う企業の DBA と調整しました。
                </li>
              </ul>
            </div>

            {/* pampam */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">pampam Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  韓国・ソウル | 2024年11月 - 2024年12月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                ソフトウェアエンジニア（契約）
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  AI ベースの YouTube 分析プラットフォーム向けに PostgreSQL スキーマと Rust バックエンドを設計し、AWS と GCP へのデプロイを主導。
                </li>
                <li>
                  YouTube API を使った大規模データ収集（数千万件のコメント・ユーザー）を主導。
                </li>
                <li>
                  LLM とサーバー上の OSS モデルを活用し、分析結果と要約を生成。
                </li>
              </ul>
            </div>

            {/* EAN Technology */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">EAN Technology Co. Ltd</h3>
                <span class="text-sm font-mono text-ink-muted">
                  韓国・ソウル | 2023年8月 - 2024年8月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                バックエンドソフトウェアエンジニア（リード）
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>Samsung C&T デジタルツインプロジェクト:</b> 機能不全に陥ったマイクロサービス構成を廃止し、高度に最適化した Axum (Rust) のモノリスに置き換え。月々のクラウド費用を約5,000ドルから約150ドルに削減。
                </li>
                <li>
                  約30,000行（コード全体の約75%）を実装し、複雑なドメインロジックを持つ80個のエンドポイントを開発。
                </li>
                <li>
                  9か月間の本番稼働でランタイムの停止ゼロを達成し、エラー処理の網羅率を約90%に維持。
                </li>
                <li>
                  Samsung C&amp;T 本社の数千台のリアルタイム物理センサー (BACnet、Modbus) とオンプレミスサーバーを統合。
                </li>
                <li>
                  PostgreSQL のクエリとスキーマを最適化。リソースが限られた環境（2コア、RAM 4GB）で、一部の未最適化クエリが数秒かかっていた状態から P99 レイテンシを数十ミリ秒に短縮。
                </li>
                <li>
                  GitHub Runners、AWS CodeDeploy、Docker を活用した CI/CD を実装。MUSL、静的リンク、'scratch' イメージを使ったマルチステージビルドで、セキュリティとコールドスタート効率を高めた約20MBの Docker イメージを作成。
                </li>
                <li>
                  同僚の Rust 習得を支援し、2か月以内に Node.js/Java から移行できるよう指導。
                </li>
              </ul>
            </div>

            {/* Artifyc */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Artifyc Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  米国テキサス州オースティン（リモート） | 2022年8月 - 2023年3月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                ソフトウェアエンジニアインターン
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Reddit API を使ってイラスト制作依頼を収集・投稿する Python 製 Discord bot を実装。
                </li>
                <li>
                  世界各国の通貨換算と追加料金計算を行うサーバーレスのバックエンド処理を AWS Lambda 上に開発。
                </li>
                <li>
                  React フロントエンドの改修とアセット統合を支援。
                </li>
              </ul>
            </div>
          </div>
        </section>


        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">主なプロジェクト</h2>
          <ul class="list-disc list-inside text-sm space-y-3 text-ink">
            <li><a href="https://github.com/younghyun1/sillok" class={pageStyles.link}>Sillok</a>: 追記専用の履歴、インデックス付き Turso/SQLite ビュー、訂正機能、Git によるマシン間同期を備えた、作業記録用の Rust CLI。</li>
            <li><a href="https://github.com/younghyun1/eu5-location-filter" class={pageStyles.link}>EU5 Location DB</a>: 圧縮した地図データと事前計算済みインデックス、検索可能なフィルター、サイズ変更可能なテーブルを備えた Rust と Slint のデスクトップ・WebAssembly アプリ。</li>
            <li><a href="https://github.com/younghyun1/cyhdev" class={pageStyles.link}>cyhdev.com</a>: Rust/Axum、PostgreSQL、SolidJS で構築したこのサイト。投稿、写真、ディスカッション、チャット、ブラウザー上のデモを提供します。</li>
            <li><a href="https://github.com/younghyun1/oohid" class={pageStyles.link}>oohid</a>: 書式の設定、ファイル出力、任意の重複チェックに対応する Rust 製 UUID ジェネレーター。</li>
          </ul>
        </section>

        {/* 3) IT以外の職歴 */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            3. IT以外の職歴
          </h2>

          <div class="space-y-6">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">
                  韓国海洋科学技術院 (KIMST)
                </h3>
                <span class="text-sm font-mono text-ink-muted">
                  韓国・ソウル | 2023年6月～7月、2024年8月
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                翻訳者・通訳者
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  ノルウェー大使と Dongwon Group 会長が出席するハイレベルな会議で、英語・韓国語・ノルウェー語の通訳を担当。
                </li>
                <li>
                  海洋研究開発に関する機密性の高い政府・企業間取引や協議の詳細を任されました。
                </li>
                <li>
                  米国エネルギー省のイベントに向けて、KIMST の職員と理事を指導。
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
                    米陸軍
                  </a>{" "}
                  (
                  <a
                    href="https://en.wikipedia.org/wiki/2nd_Infantry_Division_(United_States)"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    第2歩兵師団
                  </a>
                  ) /{" "}
                  <a
                    href="https://en.wikipedia.org/wiki/Republic_of_Korea_Army"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    韓国陸軍
                  </a>
                </h3>
                <span class="text-sm font-mono text-ink-muted">
                  <a
                    href="https://en.wikipedia.org/wiki/Dongducheon"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    Dongducheon
                  </a>
                  （韓国） | 2016年3月 - 2017年12月
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
                | 教官 | 軍曹
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  連絡兵兼教官として勤務。約6,000人のアメリカ人・韓国人要員への教育と事務手続きを担当。
                </li>
                <li>
                  体力づくり、文化、歴史、安全・予防に関する講習を実施。
                </li>
                <li>
                  <a
                    href="https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    2017年の危機
                  </a>
                  の期間中も即応態勢を維持し、13人のチームを率いました。
                </li>
              </ul>
            </div>

            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Coupang Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  韓国 | 2020年 - 2022年
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                荷積み作業員
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  夜勤で物流倉庫の集中的な肉体労働に従事。
                </li>
              </ul>
            </div>
          </div>
        </section>

        {/* 4) 学歴 */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            4. 学歴
          </h2>

          <div class="mb-6">
            <div class="flex flex-wrap justify-between items-baseline mb-2">
              <h3 class="text-lg font-bold">成均館大学 (Sungkyunkwan University)</h3>
              <span class="text-sm font-mono text-ink-muted">
                韓国・水原 | 2015年3月 - 2023年8月
              </span>
            </div>
            <p class="text-sm font-medium mb-2 text-ink-muted">
              ソフトウェア工学 学士
            </p>
            <div class="space-y-2 text-sm text-ink">
              <p>
                コンピューターサイエンス、Web・組み込みソフトウェア工学、電子工学を横断する総合課程を修了。論理学、低水準言語、オペレーティングシステム、ネットワーク工学に加え、ソフトウェア工学の実践にも重点を置きました。
              </p>
              <p>
                Sungkyunkwan University は韓国最古の大学で、1398年に朝鮮王朝の儒教に基づく官吏養成機関として創設されました。解放以降、韓国を代表する学術・歴史機関の一つとなり、私の在学中から世界大学ランキング上位100校に継続して選ばれています。
              </p>
            </div>
          </div>

          <div>
            <h4 class="font-bold mb-2 text-md">論文・発表</h4>
            <ul class="list-disc list-inside text-sm space-y-2 text-ink">
              <li>
                <b>
                  韓国語 NLP モデルに基づくソーシャルメディアコミュニティの感情分析
                </b>{" "}
                (IMCOM 2023, IEEE Xplore).
                <br />
                <span class="ml-5 block text-xs text-ink-muted mt-1">
                  COVID-19 流行初期のソーシャルメディアデータを KoBERT で感情分析しました。卒業制作で銅賞を受賞。
                </span>
              </li>
            </ul>
          </div>
        </section>

        {/* 5) ボランティア活動と関心 */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            5. ボランティア活動と関心
          </h2>

          <div class="grid md:grid-cols-2 gap-8">
            <div>
              <h3 class="font-bold mb-3 text-lg">個人プロジェクト</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>
                  <b>oohid:</b> Rust 製の CLI UUIDv4 ジェネレーター。libuuid の約3倍の速度で、重複チェックと Python/JSON 形式に対応。
                </li>
                <li>
                  <b>impulsr:</b> マーケティング要約向けの YouTube 文字起こし・コメント収集ツール。
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-3 text-lg">ボランティア活動</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>コミュニティセンターで英語を教えました。</li>
                <li>障害のある方々が運営するリサイクルショップでボランティアをしました。</li>
              </ul>
            </div>
          </div>
        </section>

        {/* 6) 趣味 */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            6. 趣味
          </h2>

          <div class="space-y-4">
            <div>
              <h3 class="font-bold inline mr-2">写真</h3>
              <span class="text-sm text-ink-muted">
                2010年から風景・ポートレートを撮影。約30,000枚の写真を保管。
              </span>
              <div class="mt-2">
                <a
                  href="/photographs"
                  class={`inline-flex items-center ${pageStyles.link} font-medium`}
                >
                  写真ポートフォリオを見る →
                </a>
              </div>
            </div>

            <div>
              <h3 class="font-bold inline mr-2">ジャーナリズム</h3>
              <p class="text-sm text-ink mt-1">
                学生時代に Sungkyun Times の記者を務めました。
              </p>
            </div>
          </div>
        </section>

        {/* 7) 資格とその他のスキル */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            7. 資格とその他のスキル
          </h2>

          <div class="grid md:grid-cols-2 gap-8 mb-8">
            <div>
              <h3 class="font-bold mb-2">資格・受賞歴</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>TOEFL:</b> 117/120 (CEFR C2)
                </li>
                <li>
                  <b>IELTS:</b> 8.5/9.0 (CEFR C2)
                </li>
                <li>
                  <b>7位:</b> 2021年 Capstone Design & Idea Hackathon
                </li>
                <li>
                  <b>3位:</b> 2020年 Fintech Hackathon (SNUST)
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-2">語学</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>韓国語（母語）</li>
                <li>英語（C2）</li>
                <li>初歩的なフランス語、ドイツ語、スペイン語、中国語</li>
              </ul>
            </div>
          </div>
        </section>
      </section>
    </main>
  );
}
