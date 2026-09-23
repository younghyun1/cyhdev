import { RepositorySourceMap } from "../../components/RepositorySourceMap";
import { pageStyles } from "../../styles/pageStyles";

export default function AboutBlog() {
  return (
    <main lang="ja-JP" class={pageStyles.page}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">
            ブログの技術構成
          </h1>
          <p class="text-xs text-ink-muted">
            最終更新: 2026-08-30
            <br />
          </p>
        </div>

        <section class="space-y-8">
          {/* 0) オンプレミス */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              0) ホストマシン、OS、ファイルシステム、ネットワーク構成
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              このサイトは自宅に設置したミニサーバーで、1Gbps の有線 Xfinity 回線を通じてホストしています。{" "}
              <a
                href="https://store.minisforum.com/products/minisforum-um690l-slim"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                ホストマシン
              </a>{" "}
              には最大 4.9GHz で動作する8コア Ryzen 9 モバイルプロセッサー、6400MT/s の RAM 32GB、ストレージ用の NVMe SSD 1TB を搭載しています。約400ドルで、AWS でかなり性能の低い同等構成を1年ほど借りる費用と比べても非常にお得です。セルフホスティングが好きな理由の一つでもあります。このマシンではバックエンドとフロントエンドを統合したサーバー、PostgreSQL データベース、Minecraft サーバーを動かしています。一緒に遊びたい方はメールをください。
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-1) Gentoo
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              企業サーバーで一般的に選ばれる OS ではありません。勤務先のサーバーなら、Debian Stable と ext4 ファイルシステムに、標準的なデータベースエンジンを組み合わせたでしょう。ただ、私は自分で試行錯誤してソフトウェアをコンパイルしたり、CPU アーキテクチャに最適化したパッケージをビルド・インストールしたりするのが好きです。それを可能にしてくれる素晴らしい{" "}
              <a
                href="https://www.gentoo.org/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                Gentoo
              </a>{" "}
              プロジェクトです。
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-2) データベース・バックエンド/フロントエンド・Minecraft ホストでの btrfs 利用
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
              は、スナップショット、CoW、圧縮など多くの機能を備えた最新のファイルシステムです。しかし、CoW による大幅な断片化が起きるため、データベースエンジンのホストには最適とは言えません。PostgreSQL と Minecraft のデータディレクトリを CoW の対象外にして影響を抑えています。{" "}
              <a
                href="https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                PostgreSQL のファイルシステムベンチマーク
              </a>{" "}
              によると、標準設定の btrfs は高い性能を発揮しません。ただし、CoW を無効にすれば ext4 や xfs と同等になるのではないかと思います。興味深いベンチマークになりそうです。
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-3) 内部・外部ネットワーク構成
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Xfinity の 2Gbps ケーブルインターネットに申し込んだとき、提供されるモデム兼ルーターが 2.5Gbps Ethernet に<em>対応していない</em>ほど安価なものだとは知りませんでした。そのため 1Gbps で使うことになりますが、この小さなサイトで問題になる状況は想像できません。ドメインの DNS には Route 53 を使っています。
              <br />
              <br />
              内部ではリバースプロキシ、コンテナ化、分散サービス用ツールを使う理由は特にありません。OS 上で PostgreSQL エンジンを直接動かし、フロントエンドサーバーと API サーバーを兼ねる Rust バイナリから UNIX ソケット経由で DB に接続しています。この方法は、次の説明にあるとおり{" "}
              <a
                href="https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                localhost を使うだけの場合より、はるかに高速です。
              </a>{" "}
              TPS はほぼ倍になり、余分なネットワークスタックを通らないためレイテンシも半減します。DB とサーバー間で最短 150 マイクロ秒のレイテンシを計測しました。localhost では通常約600マイクロ秒でした。クラウドのエンタープライズ構成では参考にならないケースですが、面白い結果です。昔ながらの方法ですが、こちらのほうが速いのです。2025年11月にリリースされた PostgreSQL 18 では、非同期 I/O の機能として{" "}
              <a
                href="https://pganalyze.com/blog/postgres-18-async-io"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                io_uring サポート
              </a>{" "}
              も導入され、これを有効にしています。読み取りがかなり高速になります。
            </p>
          </section>

          {/* 1) PostgreSQL */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              1) データ
            </h2>
            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-1) PostgreSQL 18
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              韓国のスタートアップや企業で働く中で、MySQL か MariaDB だけが価値ある RDBMS と見なされる風潮に気づきましたが、その理由はいまだによく分かりません。以前の上司から、かつて PostgreSQL は本格的な選択肢と見なされていなかったと聞きました。しかし、この10年ほどで性能、拡張性、データ型、ツールが大きく進歩し、特に UUID データ型のサポートや JSON データのバイナリーエンコードなど、多くの点で MySQL を上回っていることを考えると不思議です。
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-2) スキーマの要点（ブログ + 認証）
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              スキーマは良い意味で意図的に地味です。ユーザー、メモリー上のセッション、メール確認トークン、パスワード再設定トークン、投稿、コメント、投票テーブル、タグ、プロフィール写真に加え、地理情報や国際化を支えるテーブルが少数あります。ブログは単に <code>posts</code> テーブルに頼るだけではありません。投稿にはスラッグ、要約、JSONB 形式のメタデータ、公開状態、非正規化したカウンター、タグの対応関係を持たせています。コメントは親コメント ID を NULL 許容にしてスレッド化します。投票は投稿用とコメント用の専用テーブルに分けており、クエリを単純にし、後々の不必要な条件分岐を避けています。
              <br />
              <br />
              認証も実用性を重視しています。ユーザーレコードに国と言語を保存し、メールアドレスを尋ねてそのまま忘れてしまうだけのサイトにはしません。認可設計で行き詰まらないよう、ロールと権限のテーブルを用意しています。プロフィール写真はユーザー行に詰め込まず、専用テーブルでバージョン管理することで、差し替え処理をすっきりさせ、頻繁に参照するユーザーレコードに無関係なデータが増えないようにしています。
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-3) UUIDv7
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              ユーザーに見えるデータのほぼすべてに UUID を割り当てています。これは意図的な選択です。行数を推測したり、2009年のようにリソースを列挙したりできる連番 ID を公開したくありません。時系列順 UUID は完全にランダムな UUIDv4 よりインデックスとの相性がよく、実際に書き込みがある場合に役立ちます。つまり、グローバルに一意で推測しにくく、データの局所性を損ないにくい方式です。挿入パターンが安定し、B-tree の更新が減り、データベースが避けられる後処理に費やす時間も短くなります。
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-4) Diagram (request + data path)
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              まだ箱を描くのが面倒なので、ここではテキストで紹介する華やかなエンタープライズアーキテクチャ図です:
              <br />
              <br />
              ブラウザーからのリクエスト -&gt; Axum ルーター -&gt; ミドルウェアチェーン（ログ記録、認証・セッション照会、レート制限、本文サイズ検査） -&gt; ハンドラー -&gt; Diesel の非同期クエリまたはメモリーキャッシュ参照 -&gt; UNIX ソケット経由の PostgreSQL -&gt; DTO レスポンス -&gt; 圧縮 HTTPS レスポンスをブラウザーに返却。
              <br />
              <br />
              静的アセットの経路はさらに短くなります。ビルド済み SolidJS アプリは Rust バイナリに直接埋め込まれ、ブラウザーが対応していればコンテンツネゴシエーションにより zstd/gzip で配信されます。本番環境に Node プロセスは残らず、静的ファイル用のホストも不要です。「リクエストを受け取る」から「バイト列を送る」まで余分な経由もありません。
            </p>
          </section>

          {/* 2) Backend */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              2) バックエンド (Rust)
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              バックエンドは Axum、Tokio、Diesel、PostgreSQL を基盤とする Rust サービスです。実コアを備えたマシンでセルフホストするなら、アロケーターの挙動にも多少こだわろうと考え、mimalloc をグローバルアロケーターにしています。rustls で TLS を直接終端し、埋め込み済み SPA を配信します。認証、ブログ、写真、国際化向けの JSON API を提供し、WebSockets でホスト統計をサーバーダッシュボードへ送信します。ルーターにはリクエスト圧縮、現時点では寛容な CORS、比較的余裕のあるレート制限を設定し、本番環境では Swagger UI を認証の背後に置いています。
              <br />
              <br />
              アプリケーションは共有サーバー状態オブジェクトを持つ単一プロセスを中心に設計し、メモリー上でのセッション管理、起動時のキャッシュ同期、定期メンテナンスジョブを備えています。認証には Secure 属性と HTTP-only 属性を付けた Cookie、メール確認、パスワード再設定トークンを使い、スーパーユーザー操作ではロールを明示的に確認します。注目しているのはレイテンシです。データベース通信は UNIX ソケットを通し、接続プールは物理コア数に応じて調整します。ブログ一覧の読み取りは主にキャッシュから行い、レスポンスの組み立て時には細かな照会を大量に発行する代わりに、データの補強をまとめて処理します。余計な経由、コピー、往復を減らし、抽象化の自己満足が終わるのを待たずに済むという、私が重視する形でスタックを高速に保っています。
            </p>
          </section>

          {/* 3) Frontend */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              3) フロントエンド
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              フロントエンドは Vite と TypeScript で構築したクライアント側の SolidJS SPA です。ルートは遅延読み込みし、状態管理はできるだけ単純にしています。全体を静的アセットにコンパイルしてデプロイ時に Rust バイナリへまとめるため、本番環境で JavaScript サーバープロセスの面倒を見る必要はありません。最初の配信も Rust サーバーが静的ファイルをできるだけ効率よく返すだけです。
              <br />
              <br />
              ブログ UI には実用性と、おもちゃのようなエディターは使いたくないというこだわりを込めています。Markdown の編集には Toast UI Editor を使い、貼り付け・アップロードした画像は写真アップロード API に直接渡すことで、快適に記事を作成できるようにしています。タイトルとタグで検索でき、ページ移動にはクエリパラメーターを使います。認証状態はクライアント側で追跡しつつサーバー側でも強制し、レスポンスヘッダーを確認してサーバーのビルド情報を表示します。スタイルには Tailwind と共通ページスタイルシステムを使っています。Solid は実行時の負荷が軽く、DOM の更新も控えめです。余計な存在感を出さない UI 層に求めている特性です。
            </p>
          </section>

          {/* 4) Network & HTTPS */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              4) HTTPS、ルーティング、安全対策
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              HTTPS は Rust サーバーが rustls で直接処理します。通常の HTTP リクエストは HTTPS に転送し、Cookie には Secure と HTTP-only 属性を付け、本番環境で使用する Cookie のドメインはサイトのドメインに限定します。対応ブラウザーには静的アセットを zstd または gzip で配信します。SPA のフォールバックルーティングにより、別のリバースプロキシを介さずにディープリンクが機能します。
              <br />
              <br />
              特別に珍しいものではありませんが、安全対策を設けています。アップロードのリクエスト本文サイズ制限、ミドルウェアでのログ記録、保護されたルートでの認証確認、機密性の高いエンドポイントでのスーパーユーザー確認、悪用を抑えるレート制限、クライアントリクエスト用の API キーに対応しています。さらに、デプロイ構成そのものを把握しやすくしています。バイナリ1つ、データベース1つ、ホスト1台、アプリケーション上の TLS で構成し、原因不明のレイテンシや設定のずれが生じる余地を減らしています。流行の構成ではありませんが、高速で状態を把握しやすく、余分な負荷も抑えられます。
            </p>
          </section>

          <RepositorySourceMap />
        </section>
      </section>
    </main>
  );
}
