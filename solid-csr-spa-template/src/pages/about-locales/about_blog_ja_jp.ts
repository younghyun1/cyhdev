import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "2026年11月",
  chrome: {
    overview: "概要",
    updated: "最終更新",
    onThisPage: "このページの内容",
    sectionNavigation: "技術構成の各ページ",
    previous: "前へ",
    next: "次へ",
    readingTime: "{minutes}分で読めます",
  },
  pages: {
    overview: {
      eyebrow: "このサイトについて",
      title: "ブログの技術構成",
      lede: "Rust バイナリ1つ、PostgreSQL データベース1つ、小さなマシン1台。ここでは、このサイトの構築、ホスティング、セキュリティ、計測の方法と、この構成を選んだ理由を説明します。",
      sections: [
        {
          id: "at-a-glance",
          heading: "要点",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53,889 req/s",
                  label: "サーバー4コアで PGO と BOLT を適用した、公開読み取り混合負荷のスループット中央値",
                },
                {
                  value: "~150 µs",
                  label: "UNIX ソケット経由で観測したデータベースの往復時間（localhost TCP では ~600 µs）",
                },
                {
                  value: "0",
                  label: "本番環境の Node.js プロセス数。フロントエンドはサーバーバイナリに埋め込まれています",
                },
                {
                  value: "8",
                  count: 8,
                  label: "UI の対応言語。各言語は選択時にのみダウンロード",
                },
                {
                  value: "4,096",
                  label: "TLS の前に受け付ける同時接続数。アドレスまたは IPv6 /64 ごとに最大64",
                },
                {
                  value: "1",
                  count: 1,
                  label: "TLS、API、WebSockets、WebRTC メディア、埋め込みフロントエンドをまとめて提供するプロセス",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "リクエストの経路",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "ブラウザー",
                  detail: "TLS 上の HTTP/2 または HTTP/1.1。フロントエンドはフィンガープリント付きの不変 URL でキャッシュされます。",
                },
                {
                  label: "接続受付",
                  detail: "TCP 接続は TLS の前に受け付けを判定します。上限は合計4,096、アドレスまたは IPv6 /64 ごとに64です。",
                },
                {
                  label: "rustls",
                  detail: "TLS は Rust プロセス内で終端し、すべてのソケットで `TCP_NODELAY` を有効にしています。",
                },
                {
                  label: "Axum",
                  detail: "セキュリティヘッダー、セッション、レート制限、圧縮、本文サイズ制限を経て、機能ごとのハンドラーへ。",
                },
                {
                  label: "キャッシュ · Diesel",
                  detail: "頻繁な読み取りには上限付きのメモリー内キャッシュが応答し、それ以外は非同期 Diesel を通ります。",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "同じホスト上で UNIX ソケット経由で接続し、io_uring による非同期 I/O を使用。",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "各章",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "インフラ",
                  href: "/about-blog/infrastructure",
                  meta: "ホスト · Gentoo · ネットワーク",
                  text: "マシン、OS とファイルシステム、ネットワーク、そしてリバースプロキシを置かない理由。",
                },
                {
                  title: "バックエンド",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "サービスのスタック、データモデル、実行時の上限、セキュリティヘッダー、API 契約。",
                },
                {
                  title: "フロントエンド",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "アプリケーションシェル、読み込み、ローカライズ、デザインシステム、モーション。",
                },
                {
                  title: "サブシステム",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "プロセス内 SFU を使ったライブチャット、Minecraft エクスプローラー、Geo-IP 検索、WebAssembly。",
                },
                {
                  title: "パフォーマンス",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "最適化ビルド、その計測方法、そして結果。",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "変更の検証方法",
          blocks: [
            {
              kind: "list",
              items: [
                "実装の関門として、ネイティブと WebAssembly の両ターゲットで Clippy を実行。",
                "フォーマット、ネイティブのユニットテスト、OpenAPI 契約のずれ、フロントエンドの型チェック、lint、ユニットテスト、ビルドを cargo xtask でローカルに検証。",
                "Chromium と WebKit での Playwright テスト。axe によるアクセシビリティ検査と、320 CSS ピクセルまでのモバイルリフロー検査を含みます。",
                "使い捨ての PostgreSQL インスタンスに対する統合テストとマイグレーションのロールバックテスト、さらにローカルでの gitleaks によるシークレットスキャン。",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "技術構成",
      title: "インフラ",
      lede: "自宅の小さなマシン、ソースベースの Linux ディストリビューション、そしてトラフィックに不要な層は置かない構成です。",
      sections: [
        {
          id: "host",
          heading: "ホストマシン",
          blocks: [
            {
              kind: "paragraph",
              text: "このサイトは自宅の[ミニサーバー](https://store.minisforum.com/products/minisforum-um690l-slim)で動いています。最大 4.9 GHz までブーストする8コアの Ryzen 9 モバイルプロセッサー、32 GB の RAM、1 TB の NVMe SSD を搭載しています。約400ドルで、明らかに性能の劣るクラウドのハードウェアを1年借りるより安く、セルフホスティングへの私の熱意の表れでもあります。",
            },
            {
              kind: "paragraph",
              text: "バックエンドとフロントエンドを統合したサーバー、PostgreSQL、Minecraft サーバーを動かしています。遊びたい方はメールをください。",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo と btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) はサーバー用途として定番の選択ではありません。仕事なら ext4 上の Debian stable を選ぶでしょう。ここでは、使っているプロセッサーに合わせて最適化したパッケージを自分でコンパイルすることを楽しんでいます。",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) にはスナップショット、圧縮、コピーオンライトがありますが、コピーオンライトはデータベースファイルをひどく断片化させます。そのため、PostgreSQL と Minecraft のデータディレクトリはその対象から外しています。[ファイルシステムのベンチマーク](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison)では、標準設定の btrfs は ext4 や XFS に後れを取っています。コピーオンライトを無効にすれば差の大半は埋まると考えており、計測してみると面白そうです。",
            },
          ],
        },
        {
          id: "network",
          heading: "ネットワーク",
          blocks: [
            {
              kind: "paragraph",
              text: "トラフィックは 1 Gbps の有線 Xfinity 回線から届きます。プロバイダーのモデムは 2.5 Gb Ethernet に対応していませんが、このサイトで影響が出ることはないでしょう。DNS は Route 53 でホストしています。",
            },
            {
              kind: "paragraph",
              text: "マシン内部にはリバースプロキシ、コンテナランタイム、サービスメッシュはありません。PostgreSQL は OS 上で直接動き、1つの Rust バイナリがフロントエンドと API を提供します。平文の HTTP リクエストも同じプロセスが HTTPS へリダイレクトします。",
            },
          ],
        },
        {
          id: "sockets",
          heading: "UNIX ソケット",
          blocks: [
            {
              kind: "paragraph",
              text: "サーバーは localhost の TCP ではなく UNIX ソケットで PostgreSQL に接続し、ネットワークスタックを完全に迂回します。[ベンチマーク](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/)ではトランザクション処理量がほぼ倍になっており、ここでも localhost 経由の約 **600 µs** に対し、最短 **150 µs** の往復時間を観測しています。昔ながらの方法で、ほとんどのクラウド環境には関係ありませんが、こちらのほうが速いのです。",
            },
            {
              kind: "paragraph",
              text: "2025年9月にリリースされた PostgreSQL 18 では[非同期 I/O](https://pganalyze.com/blog/postgres-18-async-io) が追加されました。ここでは io_uring を有効にしており、読み取りが目に見えて速くなります。",
            },
          ],
        },
        {
          id: "deploy",
          heading: "ビルドとデプロイ",
          blocks: [
            {
              kind: "list",
              items: [
                "フロントエンドのビルド成果物は、事前圧縮した zstd 版と gzip 版とともにバックエンドのバイナリに埋め込まれるため、本番環境では Node.js プロセスは動きません。",
                "最適化ビルドでは、プロファイルガイド最適化による学習と、BOLT によるリンク後最適化を行えます。詳しくは[パフォーマンス](/about-blog/performance)をご覧ください。",
                "認証情報、証明書、実行時データは Git に含めず、変更がマシンの外に出る前に gitleaks のスキャンでツリーを検査します。",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "技術構成",
      title: "バックエンド",
      lede: "Axum と Tokio 上の Rust、非同期 Diesel 経由の PostgreSQL、そしてクライアントが保持できるあらゆるものへの明示的な上限。",
      sections: [
        {
          id: "stack",
          heading: "スタック",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "ランタイム", items: ["Axum", "Tokio", "hyper", "axum-server", "rustls", "mimalloc"] },
                { label: "データ", items: ["PostgreSQL 18", "Diesel", "diesel-async", "tantivy", "bitcode", "zstd"] },
                { label: "外部連携", items: ["openidconnect", "lettre", "Amazon S3", "webrtc-rs", "image + AVIF"] },
                { label: "API 契約", items: ["utoipa", "OpenAPI", "Swagger UI", "Generated TypeScript"] },
              ],
            },
            {
              kind: "paragraph",
              text: "サービスは状態を共有する単一プロセスで、メモリー内のセッション、起動時のキャッシュ同期、定期メンテナンスジョブを備えています。グローバルアロケーターは mimalloc です。実コアを持つマシンなら、アロケーターの挙動は真剣に考える価値があります。",
            },
          ],
        },
        {
          id: "data",
          heading: "データモデル",
          blocks: [
            {
              kind: "paragraph",
              text: "私が見てきた韓国のスタートアップや企業の現場では、MySQL と MariaDB だけが本格的なリレーショナルデータベースとして扱われていました。しかし PostgreSQL は、ネイティブの UUID からバイナリ JSON まで、私が重視する分野でとうに両者を上回っています。",
            },
            {
              kind: "list",
              items: [
                "**投稿**はスラッグ、要約、JSONB のメタデータ、公開状態、非正規化したカウンター、タグの対応関係を持ちます。コメントは NULL 許容の親参照でスレッド化し、投稿とコメントの投票は別々のテーブルに分けています。",
                "**アカウント**は国と言語、ロールと権限のテーブルを持ち、プロフィール写真は頻繁に参照されるユーザー行ではなく専用のテーブルでバージョン管理します。",
                "**キー**は時系列順の UUIDv7 です。連番 ID のように列挙できず、ランダムな UUIDv4 よりも B-tree の局所性をはるかに損ないにくい方式です。",
                "**検索**には、投稿のタイトルとタグを対象とするディスク永続化した tantivy インデックスを使います。正となるデータは引き続き PostgreSQL で、インデックスはそれと整合するよう保たれます。",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "実行時の上限",
          blocks: [
            {
              kind: "list",
              items: [
                "接続は共有のリミッターにより TLS の前に受け付けを判定します。上限は同時接続4,096、IPv4 アドレスまたは IPv6 /64 ごとに64です。拒否したソケットはハンドシェイクなしで閉じます。",
                "HTTP/1 のリクエストヘッダー部は20秒以内に届く必要があります。HTTP/2 接続には30秒ごとに ping を送り、同時ストリームは128まで許可します。",
                "各キャッシュには固定の上限と、読み取りを委ねられる正のデータソースがあり、クライアントが制御するキーに応じて増えるキャッシュはありません。たとえば WebAssembly バンドルのキャッシュは最大 256 MiB です。",
                "画像エンコードなどのブロッキング処理は、Tokio のワーカーではなく上限付きのワーカープールで実行します。",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "セキュリティヘッダー",
          blocks: [
            {
              kind: "paragraph",
              text: "Content-Security-Policy は起動時に一度だけ構築します。インラインスクリプトは、バイナリに組み込まれた `index.html` から計算した SHA-256 ハッシュによってのみ許可されるため、初回描画前に実行するテーマ用スクリプトがポリシーとずれることはありません。メインのオリジンでは WebAssembly をインスタンス化しません。デモと EU5 アプリケーションは、それぞれ独自のポリシーを持つサンドボックス化されたフレームで動きます。",
            },
            {
              kind: "paragraph",
              text: "セッション Cookie には `Secure` と `HttpOnly` を付け、スーパーユーザー操作はサーバー側でロールを確認します。アカウントには OpenID Connect の ID を連携でき、本番環境の Swagger UI には認証が必要です。",
            },
          ],
        },
        {
          id: "contracts",
          heading: "API 契約とアセット",
          blocks: [
            {
              kind: "list",
              items: [
                "ハンドラーには OpenAPI アノテーションを付け、そこからフロントエンドの TypeScript クライアントを生成します。生成ファイルにずれがあれば cargo xtask openapi が失敗します。",
                "Vite のマニフェストはアセットとともに埋め込まれます。1年間の `immutable` キャッシュポリシーが付くのは、マニフェストに記載されたフィンガープリント付きのファイルだけで、HTML は常に再検証されます。",
                "存在しないアセットのパスには、アプリケーションシェルではなく `no-store` 付きの 404 を返すため、古いページがスクリプトの URL で HTML をキャッシュすることはありません。",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "技術構成",
      title: "フロントエンド",
      lede: "クライアント側でレンダリングする SolidJS 2 アプリケーション。静的アセットにコンパイルし、サーバーのバイナリに埋め込んでいます。",
      sections: [
        {
          id: "stack",
          heading: "スタック",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "アプリケーション", items: ["SolidJS 2", "Solid Router 2", "TypeScript", "Vite 8"] },
                { label: "スタイル", items: ["Tailwind CSS 4", "IBM Plex Sans", "IBM Plex Mono", "OKLab"] },
                { label: "コンポーネント", items: ["Toast UI Editor", "Leaflet", "Chart.js", "highlight.js", "DOMPurify"] },
                { label: "テスト", items: ["Vitest", "Playwright", "axe-core"] },
              ],
            },
            {
              kind: "paragraph",
              text: "Solid のきめ細かなシグナルは、変更された値に依存する DOM ノードだけを更新します。また 2.0 のリリース候補では、非同期の読み取りを最も近いローディング境界までサスペンドできます。ルーターはすべてのページを遅延読み込みし、本番環境に JavaScript サーバーはありません。静的ファイルは Rust バイナリが配信します。",
            },
          ],
        },
        {
          id: "loading",
          heading: "読み込み",
          blocks: [
            {
              kind: "list",
              items: [
                "各ページ、ホームページのチャットパネルと等高線シェーダーは別々のチャンクで、ビルドは最初に読み込まれる部分の gzip サイズを報告します。",
                "初期チャンクに含まれる UI テキストは英語のみです。その他の言語と、各言語の紹介ページは別のチャンクに分かれ、選択時に読み込まれます。",
                "スロットリングしたモバイル環境でのパフォーマンステストでは、LCP 2.5 s 以下、インタラクション 200 ms 以下、CLS 0.1 以下を目標にしています。",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "ローカライズ",
          blocks: [
            {
              kind: "paragraph",
              text: "UI は英語、韓国語、フランス語、スペイン語、簡体字中国語、繁体字中国語、日本語、ドイツ語に対応しています。約740個のキーからなるカタログはバックエンドのレジストリと照合して検証され、プレースホルダーは翻訳後も保持されなければなりません。これらの紹介ページは型付きのデータモジュールなので、英語版にあるセクションを翻訳で黙って省くことはできません。",
            },
          ],
        },
        {
          id: "design",
          heading: "デザインシステム",
          blocks: [
            {
              kind: "paragraph",
              text: "温かみがありつつ技術的な見た目を目指しています。クリーム色の紙か純粋な黒、細い罫線、技術的な要素には等幅フォント、そしてアクセントカラーは一度に1色だけ。色はセマンティックな CSS 変数なので、コンポーネントにダークモード用のバリエーションは不要です。アクセントのプリセットはページ背景とのコントラストをテストで確認し、訪問者が入力した RGB の色はモードごとに読みやすく調整します。ライト・ダークモードとアクセントは初回描画の前に適用します。",
            },
          ],
        },
        {
          id: "motion",
          heading: "モーション",
          blocks: [
            {
              kind: "list",
              items: [
                "セクションの表示演出と読書進捗バーには CSS のスクロール駆動アニメーションを使い、対応環境ではメインスレッド外で動作します。",
                "統計値は登録済みの `@property` 整数でカウントアップし、支援技術向けに最終値をマークアップに含めています。",
                "テーマの切り替えは、View Transitions API によってトグルから広がるように表示されます。",
                "ホームページの等高線の背景は小さな WebGL シェーダーです。ページがアイドルになってから読み込まれ、十分な性能のあるデスクトップブラウザーでのみ動作し、フレームレートに上限を設け、画面外では一時停止します。",
                "サイトの設定でこれらをすべてオフにできます。アニメーションは既定でオンで、OS のモーションを減らす設定には従いません。",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "技術構成",
      title: "サブシステム",
      lede: "ブログとして厳密に必要な範囲を超えて興味深い、このサイトの構成要素です。",
      sections: [
        {
          id: "live-chat",
          heading: "ライブチャットと通話",
          blocks: [
            {
              kind: "paragraph",
              text: "チャットは、コンパクトなバイナリコーデック、モデレーション、上限付きの履歴を備えた WebSocket 上で動きます。音声・ビデオ通話は、webrtc-rs を使って Axum プロセス内で動く Selective Forwarding Unit を経由します。各ブラウザーが保持するピア接続は1つで、SFU はエンコード済みのメディアをトランスコードせずに、配信ごとの上限付きチャンネルを通じて転送し、参加者数にも上限があります。[ライブチャットを開く](/live-chat)。",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Minecraft 地形エクスプローラー",
          blocks: [
            {
              kind: "paragraph",
              text: "Paper プラグインが同一ユーザーのみアクセスできる UNIX ソケットを公開し、上限付きのワールド問い合わせに応じます。処理は一度に1つで、フレームサイズにも厳しい上限があります。[エクスプローラー](/minecraft)は、生成された地図の JavaScript を実行せず、独自の Leaflet クライアントで squaremap のタイルを描画します。",
            },
            {
              kind: "paragraph",
              text: "バイオームのオーバーレイには、gzip 圧縮で配信するコンパクトなバイナリタイル形式 **CYBM**、または独自の `cyBM` チャンクに正確なバイオーム ID を格納した PNG を使います。そのため、ホバー時にピクセルの色からバイオームを推測することはありません。未探索の地形に対するシード値ベースの予測は、Pumpkin をもとにした別のワールド生成ワーカーが行います。",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Geo-IP 検索",
          blocks: [
            {
              kind: "paragraph",
              text: "[訪問者マップ](/visitor-board)と[IP 検索](/geo-ip-db)は、外部 API を呼ばずにアドレスの位置を特定します。前処理ツールが IP2Location LITE の CSV ファイルを範囲の開始アドレスをキーとする `BTreeMap` に変換し、bitcode でエンコードしてから zstd のレベル22で圧縮します。",
            },
            {
              kind: "table",
              columns: ["データセット", "CSV", "バンドル"],
              numericColumns: [1, 2],
              rows: [
                ["IPv4", "312.5 MB", "24.9 MB"],
                ["IPv6", "794.2 MB", "42.4 MB"],
              ],
            },
            {
              kind: "paragraph",
              text: "起動時、サーバーはピークメモリーを抑えるためにバンドルを1つずつデコードし、繰り返し現れる地名をインターンします。検索では、開始アドレスが対象アドレス以下である最後の範囲を見つけ、その終端を確認します。",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "アップロードされた[デモバンドル](/projects)はディスクに一時保存され、マジックバイトで形式を判別し、展開後 50 MiB までに制限したうえで、上限付きキャッシュからサンドボックス化されたフレーム内に配信されます。ワークスペースでは、ブロック崩しやレイトレーサーなどの Rust デモもビルドしています。",
            },
            {
              kind: "paragraph",
              text: "[EU5 Location DB](/eu5-locations-db) は WebAssembly にコンパイルした Slint アプリケーションです。ページが小さなメッセージプロトコルでライト・ダークモードを伝えるため、埋め込まれたアプリの配色もサイトに合わせて切り替わります。",
            },
          ],
        },
        {
          id: "photography",
          heading: "写真処理パイプライン",
          blocks: [
            {
              kind: "paragraph",
              text: "アップロードされた画像は固定のピクセル上限内でデコードし、処理時間と引き換えにファイルを小さくするため、最も遅いプリセットで AVIF にエンコードします。EXIF データの位置情報は公開前に地図上で確認でき、同時に実行される処理ジョブは2つまでです。",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "技術構成",
      title: "パフォーマンス",
      lede: "バックエンドへのプロファイルガイド最適化と BOLT の適用を、ボトルネックにならない負荷生成ツールで計測しました。",
      sections: [
        {
          id: "approach",
          heading: "手法",
          blocks: [
            {
              kind: "paragraph",
              text: "最適化ビルドは代表的なワークロードで学習させます。計装したバイナリが、PostgreSQL、SMTP、S3、OpenID Connect、Minecraft の合成エンドポイントを備えた使い捨てのフィクスチャに対してプロファイルを記録します。LLVM がそのプロファイルでサーバーを再ビルドし（PGO）、続いて BOLT がサンプリングした実行情報をもとに最終バイナリのコード配置を並べ替えます。すべてのバリアントで LTO、フレームポインター、再ビルドした標準ライブラリを共通に使っています。",
            },
          ],
        },
        {
          id: "setup",
          heading: "計測環境",
          blocks: [
            {
              kind: "list",
              items: [
                "サーバー、フィクスチャのプロバイダー、PostgreSQL を AMD Ryzen 7 9800X3D の4コアに固定し、負荷クライアントは残りの4コアに固定。",
                "32接続をプールした HTTP/1.1 上の HTTPS で、レスポンスを最後まで読み取り、期限は2秒。",
                "順序を入れ替えた3ラウンドで、バリアントごとに5サンプルを取得。各サンプルの前に10,000件のウォームアップリクエストを送り、新しいデータベーススナップショットから開始。",
                "計測したリクエストは合計27,000,000件で、すべて HTTP 200 を返しました。",
                "負荷は oha で生成し、Playwright は計測時間の外で各ルートのレスポンスを確認します。",
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
              caption: "バリアントごとの15サンプルにおける毎秒リクエスト数の中央値。",
              groups: [
                {
                  label: "10ルートの公開読み取り",
                  items: [
                    { label: "ベースライン", value: 44874.12, display: "44,874" },
                    { label: "PGO", value: 52925.46, display: "52,925" },
                    { label: "PGO + BOLT", value: 53888.61, display: "53,889", emphasis: true },
                  ],
                },
                {
                  label: "小さなヘルスチェック JSON",
                  items: [
                    { label: "ベースライン", value: 272837.41, display: "272,837" },
                    { label: "PGO", value: 308685.26, display: "308,685", emphasis: true },
                    { label: "PGO + BOLT", value: 306276.64, display: "306,277" },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "10ルート混合負荷のテールレイテンシ。",
              columns: ["バリアント", "p95 (ms)", "p99 (ms)"],
              numericColumns: [1, 2],
              rows: [
                ["ベースライン", "2.380", "2.967"],
                ["PGO", "2.020", "2.524"],
                ["PGO + BOLT", "1.987", "2.483"],
              ],
            },
            {
              kind: "paragraph",
              text: "混合負荷のスループットは PGO で **17.9%**、PGO と BOLT の併用で **20.1%** 向上し、どちらの最適化ビルドも順序を入れ替えたすべてのラウンドで上回りました。BOLT は混合読み取りで PGO よりさらに1.8%向上し、単純なヘルスチェックのルートでは0.8%遅くなりました。これは4コアでの結果であり、本番環境の処理能力はネットワーク、データベースのワーキングセット、実際のトラフィックにも左右されます。",
            },
          ],
        },
        {
          id: "corrections",
          heading: "訂正",
          blocks: [
            {
              kind: "list",
              items: [
                "当初は `TCP_NODELAY` を設定していませんでした。axum-server の標準の Rustls アクセプターでは無効のままのため、HTTP/2 レスポンスが遅延 ACK を約40 ms 待つことがありました。現在は受け付けたすべてのソケットで設定しています。",
                "最初のベンチマークは Playwright から負荷をかけていましたが、実行が進むにつれてクライアントが遅くなり、性能低下があるかのような誤った結果が出ました。その数値は破棄しました。",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
