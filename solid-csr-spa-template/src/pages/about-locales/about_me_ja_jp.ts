import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "2026年11月",
  chrome: {
    overview: "概要",
    updated: "最終更新",
    onThisPage: "このページの内容",
    sectionNavigation: "プロフィールの各ページ",
    previous: "前へ",
    next: "次へ",
    readingTime: "{minutes}分で読めます",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "プロフィール",
      lede: "Rust、PostgreSQL、クラウドプラットフォームを扱うバックエンド・インフラエンジニアです。基盤となるハードウェアを効果的に活かすサービスを構築し、その稼働を支えるデプロイ、監視、セキュリティにも取り組んでいます。",
      sections: [
        {
          id: "introduction",
          heading: "はじめに",
          blocks: [
            {
              kind: "paragraph",
              text: "コロラド州を拠点とする韓国人ソフトウェアエンジニアです。**Soundpatrol** では Rust エンジニアとして、主に Rust、Python、PostgreSQL、Kubernetes、Google Cloud を使い、サイバーセキュリティ、DevOps、Web サービス、開発者向けツールに取り組んでいます。",
            },
            {
              kind: "paragraph",
              text: "それ以前は、Samsung C&T の建物向けデジタルツインでバックエンドとインフラを主導し、Hyundai、Kia、Genesis のアプリのバックエンドに携わり、YouTube 分析プラットフォームのデータ収集を構築しました。2023年以降は、バックエンドサービス、データパイプライン、それらを支えるインフラを中心に活動しています。",
            },
            {
              kind: "paragraph",
              text: "最新のハードウェアとプログラミング言語を効果的に活かすサービスづくりを大切にしています。具体的には、丁寧なスキーマ設計、上限を設けたキューとキャッシュ、明示的なエラー処理、そして状態を確認でき復旧もできるデプロイです。同僚が Rust を身につける手助けをするのが好きで、自分の仕事に誇りを持つ人たちと働きたいと考えています。",
            },
          ],
        },
        {
          id: "results",
          heading: "主な成果",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "Samsung C&T デジタルツインの月額クラウド費用を約5,000ドルから150ドルに削減",
                },
                {
                  value: "80",
                  count: 80,
                  label: "デジタルツイン向けに実装した、建設エンジニアリングのドメインロジックを含む REST エンドポイント",
                },
                {
                  value: "9か月",
                  count: 9,
                  suffix: "か月",
                  label: "ランタイム停止なしでのデジタルツインの本番稼働期間",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "[PGO と BOLT](/about-blog/performance) によるこのサイトのバックエンドの混合読み取りスループット向上",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "マルチステージビルドで作成した静的リンクのサービスコンテナ",
                },
                {
                  value: "30,000",
                  label: "デジタルツインのバックエンドで書いたコード行数（全体の約75%）",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "使用技術",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "言語",
                  items: ["Rust", "Python", "TypeScript", "SQL", "Java", "C", "C++"],
                },
                {
                  label: "バックエンド",
                  items: ["Axum", "Tokio", "Diesel", "tokio-postgres", "Spring Boot", "FastAPI", "WebSockets", "gRPC", "Protocol Buffers"],
                },
                {
                  label: "データ",
                  items: ["PostgreSQL", "Valkey / Redis", "SQLite / Turso", "MySQL", "FAISS", "tantivy"],
                },
                {
                  label: "インフラ",
                  items: ["Kubernetes", "Helm", "Docker", "OpenTofu / Terraform", "GitHub Actions", "Google Cloud", "AWS", "Linux"],
                },
                {
                  label: "信頼性とセキュリティ",
                  items: ["OpenTelemetry", "IAM", "TLS", "Network policy", "Credential lifecycle", "Incident response"],
                },
                {
                  label: "インターフェースとデバイス",
                  items: ["SolidJS", "Leptos", "Slint", "WebAssembly", "WebRTC", "BACnet", "Modbus"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "さらに詳しく",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "職歴",
                  href: "/about/experience",
                  meta: "2016年 - 現在",
                  text: "Soundpatrol から最初のインターンシップまでの職歴と、それぞれのプロジェクトや成果。",
                },
                {
                  title: "プロジェクト",
                  href: "/about/projects",
                  meta: "Rust · オープンソース",
                  text: "Sillok、EU5 Location DB、このウェブサイトなど、個人およびオープンソースの取り組み。",
                },
                {
                  title: "バックグラウンド",
                  href: "/about/background",
                  meta: "学歴 · 研究",
                  text: "学歴、発表した研究、受賞歴、語学、ソフトウェア以外の活動。",
                },
                {
                  title: "このサイトの仕組み",
                  href: "/about-blog",
                  meta: "技術構成",
                  text: "cyhdev.com を支えるホスト、バックエンド、フロントエンド、サブシステム、パフォーマンス計測。",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "連絡先",
          blocks: [
            {
              kind: "list",
              items: [
                "**メール**：[younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**：[github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**：[linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "連絡にはメールが最も確実です。Minecraft サーバーに参加したい場合も、メールでお知らせください。",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "プロフィール",
      title: "職歴",
      lede: "米国のスタートアップ、韓国の企業顧客向けの案件、初期のインターンシップでのバックエンド・インフラ・セキュリティの仕事と、ソフトウェアの仕事に就く前の経歴です。",
      sections: [
        {
          id: "software",
          heading: "ソフトウェアエンジニアリング",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Rust エンジニア",
                  period: "2026年5月 - 現在",
                  location: "米国 · リモート",
                  summary: "音楽・音声検出企業で、サイバーセキュリティ、DevOps、Web サービス開発、開発者向けツールを担当。",
                  highlights: [
                    "Google Cloud 上の Kubernetes インフラ、Infrastructure as Code、そして短期間のみ有効で環境ごとにスコープを限定した認証情報を使う、レビュー済みのデプロイワークフローを保守。",
                    "本番データストアの移行を計画・実施し、各切り替えの前に移行後の状態を検証して復旧手順をリハーサル。",
                    "メトリクス、ログ、トレースを OpenTelemetry に集約し、サービスの健全性、診断、コストの可視化のための Rust ツールを開発。",
                    "セキュリティ体制の整備を担当：アクセス管理、認証情報のローテーションと有効期限の追跡、ポリシー、インシデント対応、監査用のエビデンス。",
                    "運用・サポート業務向けに、Rust と Python で社内サービスとツールを開発。",
                  ],
                  tags: ["Rust", "Python", "PostgreSQL", "Kubernetes", "Google Cloud", "OpenTofu", "OpenTelemetry"],
                },
                {
                  organization: "GenesisNest",
                  title: "ソフトウェアエンジニア",
                  period: "2025年1月 - 2025年7月",
                  location: "韓国・城南",
                  summary: "社内プラットフォームと K-pop グループの公式ファンアプリ向けのバックエンドサービスとインフラツール。",
                  highlights: [
                    "Hyundai、Kia、Genesis の公式アプリのバックエンドに API、不具合修正、連携機能を実装。大規模な Java コードベースで作業し、アジアとヨーロッパで数千万人が利用するサービスについてベンダーと調整しました。",
                    "翻訳済みコンテンツを変換して企業のデータベースに取り込む国際化パイプラインを Rust と Python で構築し、工場システムに接続されたデータセットを担当するデータベース管理者と調整。",
                    "社内サービス向けのインフラ管理ツールを開発。",
                  ],
                  tags: ["Java", "Spring Boot", "Rust", "Python"],
                },
                {
                  organization: "pampam Inc.",
                  title: "ソフトウェアエンジニア（契約）",
                  period: "2024年11月 - 2024年12月",
                  location: "韓国・ソウル",
                  highlights: [
                    "AI を活用した YouTube 分析プラットフォームの PostgreSQL スキーマと Rust バックエンドを設計し、AWS と Google Cloud へのデプロイを主導。",
                    "数千万件のコメントとユーザーを対象とする YouTube API の収集ワークフローを構築し、チャンネルデータ、文字起こし、ページコンテンツを組み合わせました。",
                    "ホスト型の言語モデルとサーバー上のオープンソースモデルを統合し、要約とマーケティングインサイトを生成。",
                  ],
                  tags: ["Rust", "PostgreSQL", "AWS", "Google Cloud"],
                },
                {
                  organization: "EAN Technology Co. Ltd.",
                  title: "バックエンドソフトウェアエンジニア（リード）",
                  period: "2023年8月 - 2024年8月",
                  location: "韓国・ソウル",
                  summary: "建物・エネルギー分野の2つのプラットフォームで、バックエンド、データベース、クラウドインフラの開発を主導。",
                  highlights: [],
                  projects: [
                    {
                      name: "Samsung C&T グローバルエンジニアリングセンターのデジタルツイン",
                      highlights: [
                        "Express バックエンドとクラウドのマイクロサービス構成を、最適化した Rust/Axum のモノリスに置き換え、月額インフラ費用を約**5,000ドルから150ドル**に削減。",
                        "バックエンドの約75%にあたる約**30,000行**を書き、建設エンジニアリングのドメインロジック、データ統合、リアルタイム処理を担う**80個の REST エンドポイント**を実装。",
                        "2コア・4 GB の環境で、非同期 Rust、マルチスレッド、キャッシュ、PostgreSQL のスキーマとクエリのチューニングにより、P99 が数秒かかっていた多くのリクエスト経路を1桁から2桁ミリ秒に短縮。",
                        "Samsung C&T 本社の現地で、数千台のセンサー、メーター、空調・水システムのコントローラーを BACnet と Modbus で統合。",
                        "ランタイム停止なしで9か月間本番稼働し、エラー処理の網羅率は約90%。",
                        "GitHub ランナー、AWS CodeDeploy、Docker で CI/CD を構築。プロセッサー向けに最適化した静的ビルドにより、約20 MB の移植性の高いコンテナを作成。",
                        "ECS、EC2、SQS、SNS、S3、SageMaker を通じて予測モデルの推論を連携し、Samsung SDS のセキュリティ要件を実装。さらに、パートナースタジオが制作した Unreal Engine 4 のインターフェースとバックエンドを統合。",
                      ],
                    },
                    {
                      name: "AI エネルギー管理サービス",
                      highlights: [
                        "PostgreSQL スキーマと Rust/Axum API をゼロから設計し、バックエンドの約70%にあたる約**25,000行**を作成。",
                        "デジタルツインでの教訓を活かし、クエリとコンパイル時間を制御するため、ORM ではなく tokio-postgres による直接の SQL を採用。",
                        "バックエンド、フロントエンド、データサイエンスのエンジニアからなる9人のチームで、Node.js と Java 出身のエンジニア2人が約1か月で Rust の実務に対応できるよう支援。",
                      ],
                    },
                  ],
                  tags: ["Rust", "Axum", "PostgreSQL", "AWS", "Docker", "BACnet", "Modbus"],
                },
                {
                  organization: "Artifyc Inc.",
                  title: "ソフトウェアエンジニアインターン",
                  period: "2022年8月 - 2023年3月",
                  location: "テキサス州オースティン · リモート",
                  highlights: [
                    "Reddit API でイラスト制作依頼を見つけてコミュニティチャンネルに投稿する Python 製 Discord bot を構築し、AWS Lambda で5分ごとに実行。",
                    "法域ごとの通貨換算、課金、追加料金計算を行う Lambda の処理を実装。",
                    "React フロントエンドのレイアウト、アセット、文言の改修に貢献。",
                  ],
                  tags: ["Python", "AWS Lambda", "React"],
                },
              ],
            },
          ],
        },
        {
          id: "other",
          heading: "ソフトウェア以前",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "韓国海洋科学技術院",
                  title: "翻訳者・通訳者",
                  period: "2023年6月～7月、2024年8月",
                  location: "韓国・ソウル · 契約",
                  highlights: [
                    "駐韓ノルウェー大使、ノルウェーの政務次官、研究院の幹部、Dongwon Group 会長が出席した海洋研究会議で、英語・韓国語・ノルウェー語の通訳を担当。",
                    "国際会議と、共同プロジェクトに関する米国エネルギー省のイベントに向けて、スピーチを翻訳し、研究院の職員と院長を指導。",
                  ],
                },
                {
                  organization: "韓国陸軍 · 米陸軍第2歩兵師団",
                  title: "軍曹、KATUSA 連絡兵・教官",
                  period: "2016年3月 - 2017年12月",
                  location: "韓国・東豆川",
                  highlights: [
                    "[KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army) プログラムを通じて師団の本部大隊に所属し、韓米合同の環境で勤務。",
                    "約6,000人の米国人・韓国人要員に体力、文化、歴史、予防の講習を行い、着任手続きを担当。",
                    "13人のチームを率い、朝鮮半島における[2017年の危機](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis)の間も即応態勢を維持。",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "物流荷積み作業員",
                  period: "2020年 - 2022年",
                  location: "韓国 · 単発契約",
                  highlights: [
                    "大学で学びながら、夜勤で倉庫作業に従事。",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "プロフィール",
      title: "プロジェクト",
      lede: "個人およびオープンソースの取り組みで、大半は Rust 製です。インタラクティブなデモは[プロジェクトページ](/projects)にあります。",
      sections: [
        {
          id: "featured",
          heading: "主なプロジェクト",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "コマンドラインの作業記録ツール。目標、完了したタスク、訂正、取り消しを追記専用のイベントとして記録し、インデックス付きのプロジェクションで現在の状態を保持します。Git ベースのアーカイブにより、複数のマシン間で履歴を統合します。",
                  tags: ["イベントソーシング", "同期"],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Europa Universalis V の地図データを扱うデスクトップ・ブラウザーアプリ。圧縮データセットを内蔵し、事前計算済みの検索・並べ替えインデックス、カテゴリーフィルター、サイズ変更可能なテーブルを備えています。[こちらで試せます](/eu5-locations-db)。",
                  tags: ["ネイティブ + Web", "インデックス"],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "このサイトです。投稿、写真、フォーラム、通話対応のライブチャット、Minecraft の地形エクスプローラー、WebAssembly デモを、1つの Rust バイナリで提供しています。[構築方法はこちら](/about-blog)。",
                  tags: ["モノレポ", "セルフホスト"],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "UUIDv4 の生成・整形ツール。ターミナルとファイルへの出力、任意の重複チェック、Python と JSON に対応した書式を備えています。初期のローカルベンチマークでは uuidgen の約3倍の速度でした。",
                  tags: ["スループット"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "その他の取り組み",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**：Rust 製 Minecraft サーバーへの[アップストリーム修正](https://github.com/Pumpkin-MC/Pumpkin/pull/3447)。インポート時と保存時に Paper のオーバーワールド設定を保持します。現在レビュー待ちです。",
                "**Minecraft の地図ツール**：このサイトの[地形エクスプローラー](/minecraft)を支える Paper プラグインと Rust サービス。コンパクトなバイナリ形式のバイオームタイルや、シード値に基づくバイオーム予測を含みます。",
                "**ファイルシステム同期**：異なるファイルシステム間での双方向フォルダー同期とビルド成果物の整理を行う Rust ユーティリティ。ドライラン、操作ジャーナル、任意のチェックサム、意図しない大量削除の防止機能を備えています。",
                "**impulsr**：YouTube の文字起こしとコメント収集を行い、マーケティング業務向けに言語モデルで要約を生成するツール。",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "開発の考え方",
          blocks: [
            {
              kind: "paragraph",
              text: "小さく、中身を確認しやすいシステムを好みます。1つのバイナリで足りるなら1つのバイナリにし、すべてのキューとキャッシュに明示的な上限を設け、データ形式は習慣ではなくアクセスパターンに合わせて選びます。",
            },
            {
              kind: "paragraph",
              text: "主張する前に計測します。計測方法はこのサイトの[パフォーマンスに関するメモ](/about-blog/performance)にまとめています。",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "プロフィール",
      title: "バックグラウンド",
      lede: "学歴、研究、資格、ソフトウェア以外の関心事。",
      sections: [
        {
          id: "education",
          heading: "学歴",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "成均館大学",
                  title: "ソフトウェア工学 学士（工学）",
                  period: "2015年3月 - 2023年8月",
                  location: "韓国・水原",
                  summary: "コンピューター科学工学科所属。在学期間には兵役義務の期間が含まれます。",
                  highlights: [
                    "ソフトウェア工学の実践に加え、組み込みソフトウェア、論理回路、オペレーティングシステム、ネットワーク、低水準プログラミング、電子工学を履修。",
                    "成均館大学は、1398年に朝鮮王朝の最高学府として設けられた機関を起源とし、韓国を代表する研究大学の一つです。",
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
              text: "**韓国語 NLP モデルに基づくソーシャルメディアコミュニティの感情分析。** KoBERT を韓国語ソーシャルメディアの感情分析と分類に適用し、COVID-19 流行初期の感情の変化を調べました。",
            },
            {
              kind: "paragraph",
              text: "このプロジェクトは、約60件の応募があった大学の卒業論文・プロジェクト賞で銅賞を受賞しました。",
            },
          ],
        },
        {
          id: "awards",
          heading: "受賞歴と資格",
          blocks: [
            {
              kind: "table",
              columns: ["年", "受賞・資格", "詳細"],
              rows: [
                ["2023", "**銅賞**", "成均館大学 卒業論文・プロジェクト賞"],
                ["2023", "**IELTS 8.5 / 9.0**", "CEFR C2"],
                ["2021", "**7位**", "Capstone Design and Idea Hackathon（韓国インターネット情報学会）"],
                ["2020", "**3位**", "Fintech Hackathon（ソウル科学技術大学校）"],
                ["2018", "**TOEFL 117 / 120**", "CEFR C2"],
              ],
            },
          ],
        },
        {
          id: "languages",
          heading: "語学",
          blocks: [
            {
              kind: "list",
              items: [
                "**韓国語**：母語。",
                "**英語**：ビジネスレベルで流暢（C2）。",
                "**フランス語、ドイツ語、スペイン語、中国語**：初歩的な知識。",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "ソフトウェア以外",
          blocks: [
            {
              kind: "list",
              items: [
                "**写真**：2010年から風景とポートレートを撮影しており、約30,000枚を保管しています。一部を[ギャラリー](/photographs)で公開しています。",
                "**ジャーナリズム**：大学の英字誌 Sungkyun Times の学生記者として、テクノロジー、セキュリティ、国際問題を取材。",
                "**教育とボランティア**：コミュニティセンターで英語を教え、障害のある方々が運営するリサイクルショップでボランティアをしました。",
                "**翻訳**：公的機関での通訳と翻訳。詳しくは[職歴](/about/experience#other)をご覧ください。",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
