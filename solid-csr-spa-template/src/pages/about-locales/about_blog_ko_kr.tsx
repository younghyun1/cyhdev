import { RepositorySourceMap } from "../../components/RepositorySourceMap";
import { pageStyles } from "../../styles/pageStyles";

export default function AboutBlog() {
  return (
    <main lang="ko-KR" class={pageStyles.page}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">
            블로그 기술 구성
          </h1>
          <p class="text-xs text-ink-muted">
            마지막 업데이트: 2026-08-30
            <br />
          </p>
        </div>

        <section class="space-y-8">
          {/* 0) 온프레미스 */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              0) 호스트 장비, 운영체제, 파일 시스템 및 네트워크 구성
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              사이트는 집에 둔 미니 서버에서 1Gbps 유선 Xfinity 회선을 통해 호스팅됩니다. {" "}
              <a
                href="https://store.minisforum.com/products/minisforum-um690l-slim"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                호스트
              </a>{" "}
              에는 최대 4.9GHz로 작동하는 8코어 Ryzen 9 모바일 프로세서와 6400MT/s 메모리 32GB, 저장 장치인 NVMe SSD 1TB가 들어 있습니다. 약 400달러라는 가격은 AWS에서 성능이 훨씬 낮은 동급 장비를 1년가량 빌리는 비용에 비하면 매우 경제적이며, 제가 직접 호스팅을 즐기는 이유도 보여 줍니다. 이 서버에서는 통합 백엔드·프런트엔드 서버, postgreSQL 데이터베이스, Minecraft 서버를 실행합니다. 함께 플레이하고 싶다면 이메일을 보내 주세요.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-1) Gentoo
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              기업 서버에서 흔히 선택하는 운영체제는 아닙니다. 회사 서버를 구성한다면 Debian Stable과 ext4 파일 시스템, 평범한 데이터베이스 엔진을 택했을 겁니다. 하지만 저는 직접 소프트웨어를 다듬고 컴파일하며 CPU 아키텍처에 맞춰 패키지를 빌드하고 설치하는 일을 즐깁니다. 이 모든 것을 가능하게 해 주는 훌륭한{" "}
              <a
                href="https://www.gentoo.org/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                Gentoo
              </a>{" "}
              프로젝트 덕분입니다.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-2) 데이터베이스·백엔드/프런트엔드·Minecraft 호스트에서 btrfs 사용
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
              는 스냅샷, CoW, 압축 등 다양한 기능을 갖춘 현대적인 파일 시스템입니다. 하지만 CoW로 인해 단편화가 심해질 수 있어 데이터베이스 엔진을 운영하는 호스트에 가장 적합한 선택은 아닙니다. PostgreSQL과 Minecraft 데이터 디렉터리에서는 CoW를 제외해 이 문제를 줄였습니다. {" "}
              <a
                href="https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                PostgreSQL 파일 시스템 벤치마크
              </a>{" "}
              를 보면 기본 설정의 btrfs 성능은 그리 좋지 않습니다. 다만 CoW를 끄면 ext4나 xfs와 비슷한 수준이 될 수도 있다고 생각합니다. 직접 측정해 보면 흥미로울 것 같습니다.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-3) 내부 및 외부 네트워크 구성
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Xfinity의 2Gbps 케이블 인터넷 서비스에 가입할 때, 제공되는 모뎀 겸 라우터가 <em>2.5Gbps 이더넷을 지원하지 않는다는 사실</em>을 몰랐습니다. 결국 1Gbps로 써야 하지만, 제 작은 웹사이트에서 문제가 될 상황은 떠오르지 않습니다. 도메인의 DNS는 Route 53에서 제공합니다.
              <br />
              <br />
              내부 구성에는 리버스 프록시, 컨테이너화, 분산 서비스 도구를 쓸 이유가 없습니다. OS에서 postgreSQL을 직접 실행하고, 프런트엔드와 API 서버를 겸하는 Rust 바이너리가 UNIX 소켓으로 DB에 연결됩니다. 이 방식은{" "}
              <a
                href="https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                여기에서 설명하듯 localhost를 사용하는 것
              </a>{" "}
              보다 훨씬 빠릅니다. TPS가 거의 두 배에 이를 수 있고 불필요한 네트워크 계층을 거치지 않아 지연 시간도 절반으로 줄어듭니다. DB와 서버 사이에서 최저 150마이크로초의 지연 시간을 관찰했습니다. localhost를 사용했을 때는 보통 약 600마이크로초였습니다. 클라우드 엔터프라이즈 서비스 구성과는 관련이 적지만, 재미있는 결과입니다. 오래된 방식이지만 더 빠릅니다! 2025년 11월에 나온 PostgreSQL 18에는 비동기 I/O도 추가되었습니다. {" "}
              <a
                href="https://pganalyze.com/blog/postgres-18-async-io"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                io_uring 지원
              </a>{" "}
              을 활성화해 읽기 속도를 꽤 높였습니다.
            </p>
          </section>

          {/* 1) PostgreSQL */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              1) 데이터
            </h2>
            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-1) PostgreSQL 18
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              한국의 스타트업과 기업에서 일하면서 MySQL이나 MariaDB만 쓸 만한 RDBMS로 여기는 문화를 봤지만, 아직도 그 이유를 잘 모르겠습니다. 예전 상사 몇 분은 PostgreSQL이 과거에는 진지한 선택지로 취급되지 않았다고 말했습니다. 지난 10여 년간 성능, 확장성, 데이터 유형, 도구가 크게 발전했고 특히 UUID 데이터 유형 지원과 JSON 데이터의 바이너리 인코딩 등 여러 면에서 MySQL을 앞섰다는 점을 생각하면 의아합니다.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-2) 스키마 주요 내용 (블로그 및 인증)
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              스키마는 좋은 의미에서 의도적으로 단순합니다. 사용자, 메모리 내 세션, 이메일 확인 토큰, 비밀번호 재설정 토큰, 게시물, 댓글, 투표 테이블, 태그, 프로필 사진, 그리고 지리·다국어 지원 테이블 몇 개로 구성됩니다. 블로그는 단순히 <code>posts</code> 테이블 하나에 기대는 구조가 아닙니다. 게시물에는 슬러그, 요약, JSONB 메타데이터, 게시 상태, 비정규화 카운터, 태그 매핑이 있습니다. 댓글은 nullable 부모 댓글 ID를 통해 계층을 이루며, 게시물 투표와 댓글 투표는 전용 테이블로 분리해 쿼리 경로를 단순하게 하고 나중에 생길 불필요한 조건문을 줄입니다.
              <br />
              <br />
              인증 구성도 실용적입니다. 사용자 레코드에 국가와 언어를 저장해 이메일만 받고 사용자를 잊어버리는 데 그치지 않습니다. 권한 설계에서 막다른 길에 몰리지 않도록 역할 및 권한 테이블도 둡니다. 프로필 사진은 사용자 행에 넣는 대신 별도 테이블에서 버전을 관리하므로 교체 로직이 단순해지고, 자주 읽는 사용자 레코드에 무관한 내용이 섞이지 않습니다.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-3) UUIDv7
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              사용자에게 보이는 거의 모든 항목은 UUID를 키로 사용하며, 이는 의도적인 선택입니다. 누구나 행 수를 추측하거나 리소스를 열거할 수 있는 순차 ID를 공개하고 싶지 않습니다. 시간 순서 UUID는 완전히 무작위인 UUIDv4보다 인덱스에 더 잘 맞아 쓰기가 실제로 발생할 때 유리합니다. 전역적으로 고유하고 추측하기 어려우며 데이터 지역성에도 덜 불리합니다. 삽입 패턴이 안정되고 B-tree 변경이 줄어 데이터베이스가 불필요한 관리 작업에 쓰는 시간도 감소합니다.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-4) 다이어그램 (요청 및 데이터 경로)
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              상자를 그릴 엄두를 내지 못해 텍스트로 옮긴 근사한 엔터프라이즈 아키텍처 다이어그램입니다:
              <br />
              <br />
              브라우저 요청 -&gt; Axum 라우터 -&gt; 미들웨어 체인 (로깅, 인증/세션 조회, 요청 제한, 본문 크기 검사) -&gt; 핸들러 -&gt; Diesel 비동기 쿼리 또는 메모리 캐시 조회 -&gt; UNIX 소켓을 통한 PostgreSQL -&gt; DTO 응답 -&gt; 압축된 HTTPS 응답을 브라우저로 반환.
              <br />
              <br />
              정적 에셋은 더 짧은 경로를 거칩니다. 빌드된 SolidJS 앱은 Rust 바이너리에 직접 포함되며 브라우저가 지원하면 콘텐츠 협상으로 zstd/gzip 압축을 사용합니다. 운영 환경에 Node 프로세스가 남지 않고 별도의 정적 파일 호스트도 필요하지 않습니다. 요청이 도착한 뒤 응답 바이트를 보내기까지 추가 경로가 없습니다.
            </p>
          </section>

          {/* 2) Backend */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              2) 백엔드 (Rust)
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              백엔드는 Axum, Tokio, Diesel, PostgreSQL 기반 Rust 서비스이며, 실제 코어가 있는 장비에서 직접 호스팅하는 만큼 할당자 동작도 신경 써서 전역 할당자로 mimalloc을 사용합니다. rustls로 TLS를 직접 종료하고 내장 SPA를 제공하며 인증, 블로그, 사진, 다국어 JSON API를 노출하고 WebSockets로 서버 대시보드에 호스트 통계를 전송합니다. 라우터에는 요청 압축, 현재는 관대한 CORS 정책, 비교적 넉넉한 요청 제한이 포함되며 운영 환경에서는 Swagger UI를 인증 뒤에 둡니다.
              <br />
              <br />
              애플리케이션은 공유 서버 상태 객체를 사용하는 단일 프로세스 구조에 메모리 내 세션 관리, 시작 시 캐시 동기화, 예약 유지보수 작업을 더했습니다. 인증에는 보안 HTTP 전용 쿠키, 이메일 확인, 비밀번호 재설정 토큰, 슈퍼유저 작업에 대한 명시적인 역할 검사가 쓰입니다. 특히 지연 시간을 줄이는 방식에 신경 썼습니다. 데이터베이스 트래픽은 UNIX 소켓을 통하고, 연결 풀 크기는 물리 코어 수에 맞춰 조정하며, 블로그 목록은 대부분 캐시에서 읽습니다. 응답 데이터는 작은 조회를 잔뜩 반복하는 대신 배치로 보강합니다. 제가 중요하게 여기는 방식으로 빠르게 동작하도록 경유, 복사, 왕복을 줄이고 불필요한 대기를 없앴습니다.
            </p>
          </section>

          {/* 3) Frontend */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              3) 프런트엔드
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              프런트엔드는 Vite와 TypeScript로 만든 클라이언트 측 SolidJS SPA입니다. 경로는 지연 로딩하고 상태는 단순하게 관리합니다. 전체 앱은 정적 에셋으로 컴파일되어 배포 시 Rust 바이너리에 포함됩니다. 따라서 운영 환경에서 JavaScript 서버 프로세스를 따로 관리할 필요 없이 Rust 서버가 정적 파일을 효율적으로 전달합니다.
              <br />
              <br />
              블로그 UI에는 실용성과 장난감 같은 편집기는 쓰지 않겠다는 제 고집이 함께 담겼습니다. Markdown 작성에는 Toast UI Editor를 사용하고, 붙여 넣거나 업로드한 이미지는 사진 업로드 API로 바로 전달해 글을 편하게 작성할 수 있습니다. 검색은 제목과 태그를 지원하고 페이지는 쿼리 매개변수로 이동합니다. 인증 상태는 클라이언트에서 추적하지만 서버에서 강제하며, 응답 헤더를 확인해 서버 빌드 정보도 표시합니다. Tailwind 기반 스타일과 공통 페이지 스타일 시스템을 사용합니다. Solid는 런타임이 가볍고 DOM 변경이 적어, 제 역할이 방해되지 않는 것인 UI 계층에 잘 맞습니다.
            </p>
          </section>

          {/* 4) Network & HTTPS */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              4) HTTPS, 라우팅 및 안전 장치
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              HTTPS는 Rust 서버가 rustls로 직접 처리합니다. 일반 HTTP 요청은 HTTPS로 전환하고 쿠키에는 Secure 및 HTTP-only 속성을 설정하며, 운영 환경의 쿠키 도메인은 사이트 도메인으로 제한합니다. 지원되는 경우 정적 에셋은 zstd 또는 gzip으로 제공하고 SPA 대체 라우팅으로 별도의 리버스 프록시 없이도 딥 링크가 작동합니다.
              <br />
              <br />
              특별히 복잡한 안전 장치는 아니지만 필요한 보호 기능을 갖췄습니다. 업로드 요청 본문 크기 제한, 미들웨어 로깅, 보호 경로의 인증 검사, 민감한 엔드포인트의 슈퍼유저 확인, 악용을 어렵게 하는 요청 제한, 클라이언트 요청용 API 키 지원이 있습니다. 무엇보다 배포 구성이 단순해 이해하기 쉽습니다. 바이너리 하나, 데이터베이스 하나, 호스트 하나, 앱에서 처리하는 TLS 구성으로 지연 시간이나 설정 드리프트가 불투명해질 여지를 줄였습니다. 유행하는 아키텍처는 아니지만 빠르고 관측 가능하며 오버헤드가 적습니다.
            </p>
          </section>

          <RepositorySourceMap />
        </section>
      </section>
    </main>
  );
}
