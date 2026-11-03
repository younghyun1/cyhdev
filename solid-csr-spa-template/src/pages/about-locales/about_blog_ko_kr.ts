import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "2026년 11월",
  chrome: {
    overview: "개요",
    updated: "마지막 업데이트",
    onThisPage: "이 페이지의 내용",
    sectionNavigation: "기술 구성 페이지",
    previous: "이전",
    next: "다음",
    readingTime: "읽는 데 {minutes}분",
  },
  pages: {
    overview: {
      eyebrow: "이 사이트에 대해",
      title: "블로그 기술 구성",
      lede: "Rust 바이너리 하나, PostgreSQL 데이터베이스 하나, 작은 장비 한 대. 이 페이지들에서는 사이트를 어떻게 구축하고 호스팅하고 보호하고 측정하는지, 그리고 왜 이런 형태로 구성했는지 설명합니다.",
      sections: [
        {
          id: "at-a-glance",
          heading: "한눈에 보기",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53,889 req/s",
                  label: "서버 코어 4개에서 PGO와 BOLT를 적용한 혼합 공개 읽기 처리량 중앙값",
                },
                {
                  value: "~150 µs",
                  label: "UNIX 소켓을 통한 데이터베이스 왕복 시간 관측값, localhost TCP에서는 약 600 µs",
                },
                {
                  value: "130 KiB",
                  count: 130,
                  suffix: " KiB",
                  label: "번들 빌드 시 강제하는 초기 프런트엔드 에셋의 gzip 예산",
                },
                {
                  value: "8",
                  count: 8,
                  label: "선택할 때만 내려받는 인터페이스 언어",
                },
                {
                  value: "0.15 ms",
                  label: "`TCP_NODELAY` 활성화 후 HTTP/2 타일 완료 시간 중앙값, 이전에는 41 ms",
                },
                {
                  value: "1",
                  count: 1,
                  label: "TLS, API, WebSockets, WebRTC 미디어, 내장 프런트엔드를 모두 제공하는 프로세스",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "요청 경로",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "브라우저",
                  detail: "TLS 위의 HTTP/2 또는 HTTP/1.1. 프런트엔드는 핑거프린트가 붙은 불변 URL로 캐시됩니다.",
                },
                {
                  label: "연결 허용",
                  detail: "TCP 연결은 TLS 이전에 허용 여부가 결정됩니다: 전체 4,096개, 주소 또는 IPv6 /64당 64개.",
                },
                {
                  label: "rustls",
                  detail: "TLS는 Rust 프로세스 내부에서 종료되며, 모든 소켓에 `TCP_NODELAY`를 설정합니다.",
                },
                {
                  label: "Axum",
                  detail: "보안 헤더, 세션, 요청 속도 제한, 압축, 본문 크기 제한을 거친 뒤 기능 핸들러로 전달됩니다.",
                },
                {
                  label: "캐시 · Diesel",
                  detail: "상한이 있는 메모리 내 캐시가 자주 읽는 요청에 응답하고, 나머지는 모두 비동기 Diesel을 거칩니다.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "같은 호스트에서 UNIX 소켓으로 연결되며, io_uring 비동기 I/O를 사용합니다.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "목차",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "인프라",
                  href: "/about-blog/infrastructure",
                  meta: "호스트 · Gentoo · 네트워크",
                  text: "장비, 운영체제와 파일 시스템, 네트워크, 그리고 리버스 프록시가 없는 이유를 다룹니다.",
                },
                {
                  title: "백엔드",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "서비스 스택, 데이터 모델, 런타임 상한, 보안 헤더, API 계약을 다룹니다.",
                },
                {
                  title: "프런트엔드",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "애플리케이션 셸, 로딩 예산, 다국어화, 디자인 시스템, 모션을 다룹니다.",
                },
                {
                  title: "하위 시스템",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "프로세스 내 SFU를 사용하는 라이브 채팅, Minecraft 탐색기, Geo-IP 조회, WebAssembly를 다룹니다.",
                },
                {
                  title: "성능",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "최적화 빌드, 엉뚱한 것을 측정한 벤치마크, 그리고 바로잡은 수치를 다룹니다.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "변경 사항 검증 방식",
          blocks: [
            {
              kind: "list",
              items: [
                "구현 단계의 관문으로 네이티브 및 WebAssembly 대상에 Clippy를 실행합니다.",
                "서식, 네이티브 단위 테스트, OpenAPI 계약 드리프트, 프런트엔드 타입 검사, 린트, 단위 테스트, 빌드를 확인하는 검증 워크플로를 실행합니다.",
                "Chromium과 WebKit에서 Playwright를 실행하며, axe 접근성 검사와 320 CSS 픽셀까지의 모바일 리플로 검사를 포함합니다.",
                "일회용 PostgreSQL 인스턴스를 대상으로 통합 테스트와 마이그레이션 롤백 테스트를 수행하고, 로컬에서 gitleaks 비밀 정보 스캔을 실행합니다.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "기술 구성",
      title: "인프라",
      lede: "집에 둔 작은 장비, 소스 기반 Linux 배포판, 그리고 트래픽에 필요 없는 계층은 두지 않는 구성입니다.",
      sections: [
        {
          id: "host",
          heading: "호스트",
          blocks: [
            {
              kind: "paragraph",
              text: "사이트는 제 거주지에 있는 [미니 서버](https://store.minisforum.com/products/minisforum-um690l-slim)에서 실행됩니다. 최대 4.9 GHz로 부스트되는 8코어 Ryzen 9 모바일 프로세서, 32 GB RAM, 1 TB NVMe SSD를 갖췄습니다. 가격은 약 400달러로, 눈에 띄게 성능이 낮은 클라우드 장비를 1년 쓰는 비용보다 저렴하며, 직접 호스팅을 좋아하는 제 취향도 반영되어 있습니다.",
            },
            {
              kind: "paragraph",
              text: "이 장비에서는 통합 백엔드·프런트엔드 서버, PostgreSQL, Minecraft 서버를 실행합니다. 함께 플레이하고 싶다면 이메일을 보내 주세요.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo와 btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/)는 서버용으로 흔한 선택은 아닙니다. 업무 환경이라면 ext4 위에 Debian stable을 고를 것입니다. 하지만 여기서는 사용하는 프로세서에 정확히 맞춘 최적화로 패키지를 직접 컴파일하는 일을 즐깁니다.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs)는 스냅샷, 압축, 쓰기 시 복사(copy-on-write)를 제공하지만, 쓰기 시 복사는 데이터베이스 파일을 심하게 단편화합니다. 그래서 PostgreSQL과 Minecraft 데이터 디렉터리는 이 기능에서 제외했습니다. [파일 시스템 벤치마크](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison)를 보면 기본 설정의 btrfs는 ext4와 XFS에 뒤처집니다. 쓰기 시 복사를 끄면 그 격차가 대부분 좁혀질 것으로 보는데, 직접 측정해 보면 흥미로울 것입니다.",
            },
          ],
        },
        {
          id: "network",
          heading: "네트워크",
          blocks: [
            {
              kind: "paragraph",
              text: "트래픽은 1 Gbps 유선 Xfinity 회선으로 들어옵니다. 통신사 모뎀이 2.5 Gb 이더넷을 지원하지 않지만, 이 사이트에서는 차이를 느낄 일이 없습니다. DNS는 Route 53에서 호스팅합니다.",
            },
            {
              kind: "paragraph",
              text: "장비 내부에는 리버스 프록시, 컨테이너 런타임, 서비스 메시가 없습니다. PostgreSQL은 운영체제에서 직접 실행되고, Rust 바이너리 하나가 프런트엔드와 API를 제공합니다. 일반 HTTP 요청은 같은 프로세스가 HTTPS로 리디렉션합니다.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "UNIX 소켓",
          blocks: [
            {
              kind: "paragraph",
              text: "서버는 localhost TCP 대신 UNIX 소켓으로 PostgreSQL에 연결해 네트워크 스택을 완전히 건너뜁니다. [벤치마크](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/)에서는 트랜잭션 처리율이 거의 두 배가 되며, 여기서는 localhost의 약 **600 µs**에 비해 최저 **150 µs**의 왕복 시간을 관측했습니다. 구식이고 대부분의 클라우드 배포와는 무관하지만, 더 빠릅니다.",
            },
            {
              kind: "paragraph",
              text: "2025년 9월에 출시된 PostgreSQL 18에는 [비동기 I/O](https://pganalyze.com/blog/postgres-18-async-io)가 추가되었고, 여기서는 io_uring을 활성화해 읽기 속도가 눈에 띄게 빨라졌습니다.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "빌드와 배포",
          blocks: [
            {
              kind: "list",
              items: [
                "프런트엔드 빌드는 미리 압축한 zstd 및 gzip 사본과 함께 백엔드 바이너리에 내장되므로, 운영 환경에서는 Node.js 프로세스가 실행되지 않습니다.",
                "최적화 빌드는 프로필 기반 최적화로 학습시키고 BOLT로 링크 후 최적화할 수 있습니다. [성능](/about-blog/performance)을 참고하세요.",
                "자격 증명, 인증서, 런타임 데이터는 Git에 넣지 않으며, 변경 사항이 장비를 떠나기 전에 gitleaks 스캔으로 트리를 검사합니다.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "기술 구성",
      title: "백엔드",
      lede: "Axum과 Tokio 기반의 Rust, 비동기 Diesel을 거치는 PostgreSQL, 그리고 클라이언트가 열어 둘 수 있는 모든 자원에 대한 명시적인 상한.",
      sections: [
        {
          id: "stack",
          heading: "스택",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "런타임", items: ["Axum", "Tokio", "hyper", "axum-server", "rustls", "mimalloc"] },
                { label: "데이터", items: ["PostgreSQL 18", "Diesel", "diesel-async", "tantivy", "bitcode", "zstd"] },
                { label: "연동", items: ["openidconnect", "lettre", "Amazon S3", "webrtc-rs", "image + AVIF"] },
                { label: "계약", items: ["utoipa", "OpenAPI", "Swagger UI", "Generated TypeScript"] },
              ],
            },
            {
              kind: "paragraph",
              text: "서비스는 메모리 내 세션, 시작 시 캐시 동기화, 예약된 유지보수 작업을 포함한 공유 상태를 갖는 단일 프로세스입니다. 전역 할당자는 mimalloc입니다. 실제 코어가 있는 장비라면 할당자 동작을 진지하게 고려할 가치가 있습니다.",
            },
          ],
        },
        {
          id: "data",
          heading: "데이터 모델",
          blocks: [
            {
              kind: "paragraph",
              text: "제가 경험한 한국 스타트업과 기업에서는 MySQL과 MariaDB만 진지한 관계형 데이터베이스로 여겼습니다. PostgreSQL은 네이티브 UUID부터 바이너리 JSON까지, 제가 중요하게 여기는 영역에서 이미 오래전에 이들을 앞질렀습니다.",
            },
            {
              kind: "list",
              items: [
                "**게시물**에는 슬러그, 요약, JSONB 메타데이터, 게시 상태, 비정규화 카운터, 태그 매핑이 있습니다. 댓글은 nullable 부모 참조로 계층을 이루며, 게시물 투표와 댓글 투표는 테이블이 분리되어 있습니다.",
                "**계정**은 국가와 언어, 역할 및 권한 테이블을 가지며, 버전이 관리되는 프로필 사진은 자주 읽는 사용자 행이 아닌 별도 테이블에 둡니다.",
                "**키**는 시간 순서 UUIDv7 값입니다. 순차 ID처럼 열거할 수 없고, 무작위 UUIDv4보다 B-tree 지역성에 훨씬 유리합니다.",
                "**검색**은 게시물 제목과 태그에 대한 디스크 저장 tantivy 인덱스를 사용하며, 기준 데이터로 남는 PostgreSQL과 일관성을 유지합니다.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "런타임 상한",
          blocks: [
            {
              kind: "list",
              items: [
                "연결은 공유 제한기를 통해 TLS 이전에 허용됩니다: 열린 연결 4,096개, IPv4 주소 또는 IPv6 /64당 64개. 거부된 소켓은 핸드셰이크 없이 닫힙니다.",
                "HTTP/1 요청 헤드는 20초 안에 도착해야 하며, HTTP/2 연결은 30초마다 핑을 보내고 동시 스트림을 128개까지 허용합니다.",
                "각 캐시에는 고정된 상한과 캐시 미스 시 읽어 올 원본이 있으며, 클라이언트가 제어하는 키에 따라 커지는 캐시는 없습니다. 예를 들어 WebAssembly 번들 캐시는 최대 256 MiB까지 보관합니다.",
                "이미지 인코딩 같은 블로킹 작업은 Tokio 워커가 아닌 상한이 있는 워커 풀에서 실행합니다.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "보안 헤더",
          blocks: [
            {
              kind: "paragraph",
              text: "Content-Security-Policy는 시작 시 한 번 생성됩니다. 인라인 스크립트는 바이너리에 컴파일된 `index.html`에서 계산한 SHA-256 해시로만 허용되므로, 첫 페인트 전에 실행되는 테마 스크립트가 정책과 어긋날 수 없습니다. 메인 오리진은 WebAssembly를 인스턴스화하지 않습니다: 데모와 EU5 애플리케이션은 각자의 정책이 적용된 샌드박스 프레임에서 실행됩니다.",
            },
            {
              kind: "paragraph",
              text: "세션 쿠키에는 `Secure`와 `HttpOnly`가 설정되고, 슈퍼유저 작업은 서버에서 역할을 검사하며, 계정에 OpenID Connect 신원을 연결할 수 있고, 운영 환경의 Swagger UI는 인증이 필요합니다.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "계약과 에셋",
          blocks: [
            {
              kind: "list",
              items: [
                "핸들러에는 OpenAPI 어노테이션이 있고, 프런트엔드의 TypeScript 클라이언트는 이를 바탕으로 생성되며, 생성된 파일이 어긋나면 CI가 실패합니다.",
                "Vite 매니페스트는 에셋과 함께 내장됩니다. 매니페스트에 등록되고 핑거프린트가 붙은 파일만 1년짜리 `immutable` 캐시 정책을 받으며, HTML은 항상 재검증합니다.",
                "존재하지 않는 에셋 경로는 애플리케이션 셸 대신 `no-store`와 함께 404를 반환하므로, 오래된 페이지가 스크립트 URL 아래에 HTML을 캐시할 수 없습니다.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "기술 구성",
      title: "프런트엔드",
      lede: "정적 에셋으로 컴파일되어 서버 바이너리에 내장되는, 클라이언트 렌더링 SolidJS 2 애플리케이션입니다.",
      sections: [
        {
          id: "stack",
          heading: "스택",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "애플리케이션", items: ["SolidJS 2", "Solid Router 2", "TypeScript", "Vite 8"] },
                { label: "스타일링", items: ["Tailwind CSS 4", "IBM Plex Sans", "IBM Plex Mono", "OKLCH"] },
                { label: "컴포넌트", items: ["Toast UI Editor", "Leaflet", "Chart.js", "highlight.js", "DOMPurify"] },
                { label: "테스트", items: ["Vitest", "Playwright", "axe-core"] },
              ],
            },
            {
              kind: "paragraph",
              text: "Solid의 세분화된 시그널은 변경된 값에 의존하는 DOM 노드만 갱신하며, 2.0 릴리스 후보에서는 비동기 읽기가 가장 가까운 로딩 경계까지 일시 중단될 수 있습니다. 라우터는 모든 페이지를 지연 로딩하고, 운영 환경에는 JavaScript 서버가 없습니다: Rust 바이너리가 정적 파일을 내줍니다.",
            },
          ],
        },
        {
          id: "budget",
          heading: "로딩 예산",
          blocks: [
            {
              kind: "list",
              items: [
                "Vite 플러그인이 진입 청크의 정적 import를 따라가며, 초기 스크립트와 스타일이 gzip 기준 **130 KiB**를 넘으면 빌드를 실패시킵니다.",
                "초기 청크에는 영어 인터페이스 텍스트만 포함됩니다. 다른 언어와 각 언어의 소개 페이지는 선택할 때 로드되는 별도 청크입니다.",
                "스로틀링한 모바일 성능 테스트는 LCP 2.5초 이하, 상호작용 200 ms 이하, CLS 0.1 이하를 목표로 합니다.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "다국어화",
          blocks: [
            {
              kind: "paragraph",
              text: "인터페이스는 영어, 한국어, 프랑스어, 스페인어, 중국어 간체와 번체, 일본어, 독일어를 지원합니다. 약 650개 키로 이루어진 카탈로그는 백엔드 레지스트리와 대조해 검증되며, 자리 표시자는 번역 후에도 유지되어야 합니다. 이 소개 페이지는 타입이 지정된 데이터 모듈이므로, 번역에서 영어에 있는 섹션을 조용히 빠뜨릴 수 없습니다.",
            },
          ],
        },
        {
          id: "design",
          heading: "디자인 시스템",
          blocks: [
            {
              kind: "paragraph",
              text: "따뜻하면서도 기술적인 분위기를 지향합니다: 크림색 종이나 순수한 검정 배경, 가는 테두리, 기술적인 내용에는 고정폭 글꼴, 한 번에 하나의 강조 색상. 색상은 의미 기반 CSS 변수이므로 컴포넌트에 다크 모드 변형이 필요 없습니다. 강조 색상 프리셋은 색조가 달라도 체감 명도를 일정하게 유지하는 OKLCH로 정의하며, 라이트·다크 모드와 강조 색상은 첫 페인트 전에 적용됩니다.",
            },
          ],
        },
        {
          id: "motion",
          heading: "모션",
          blocks: [
            {
              kind: "list",
              items: [
                "섹션 등장 효과와 읽기 진행 표시줄은 CSS 스크롤 기반 애니메이션을 사용하며, 지원되는 환경에서는 메인 스레드 밖에서 실행됩니다.",
                "통계 수치는 등록된 `@property` 정수로 카운트업되며, 보조 기술을 위해 최종 값이 마크업에 들어 있습니다.",
                "테마 전환은 View Transitions API를 통해 토글 버튼에서부터 퍼져 나갑니다.",
                "홈 페이지의 등고선 배경은 작은 WebGL 셰이더로, 페이지가 유휴 상태가 된 뒤 로드되고, 성능이 충분한 데스크톱 브라우저에서만 실행되며, 프레임 속도를 제한하고 화면 밖에서는 일시 정지합니다.",
                "시스템이 동작 줄이기를 요청하면 장식 효과는 모두 생략합니다.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "기술 구성",
      title: "하위 시스템",
      lede: "블로그에 꼭 필요한 수준보다 더 흥미로운, 사이트의 구성 요소들입니다.",
      sections: [
        {
          id: "live-chat",
          heading: "라이브 채팅과 통화",
          blocks: [
            {
              kind: "paragraph",
              text: "채팅은 간결한 바이너리 코덱, 모더레이션, 상한이 있는 기록을 갖춘 WebSocket 위에서 동작합니다. 음성 및 영상 통화는 webrtc-rs 기반으로 Axum 프로세스 내부에서 실행되는 SFU(Selective Forwarding Unit)를 거칩니다: 각 브라우저는 피어 연결 하나만 유지하고, SFU는 발행 트랙별로 상한을 둔 채널과 참가자 수 제한을 통해 인코딩된 미디어를 트랜스코딩 없이 전달합니다. [라이브 채팅 열기](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Minecraft 지형 탐색기",
          blocks: [
            {
              kind: "paragraph",
              text: "Paper 플러그인은 같은 사용자만 접근할 수 있는 UNIX 소켓으로 상한이 있는 월드 조회를 제공하며, 한 번에 하나의 작업만 처리하고 프레임 크기를 엄격히 제한합니다. [탐색기](/minecraft)는 생성된 지도의 JavaScript를 실행하는 대신 자체 Leaflet 클라이언트로 squaremap 타일을 그립니다.",
            },
            {
              kind: "paragraph",
              text: "바이옴 오버레이는 gzip으로 압축해 제공하는 간결한 바이너리 타일 형식 **CYBM**을 사용하거나, 비공개 `cyBM` 청크에 정확한 바이옴 정보를 담은 PNG를 사용하므로, 마우스를 올렸을 때 픽셀 색상으로 바이옴을 추측하지 않습니다. 탐색되지 않은 지형에 대한 시드 기반 예측은 Pumpkin으로 만든 별도의 월드 생성 워커가 담당합니다.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Geo-IP 조회",
          blocks: [
            {
              kind: "paragraph",
              text: "[방문자 보드](/visitor-board)와 [IP 조회](/geo-ip-db)는 외부 API를 호출하지 않고 주소의 위치를 찾습니다. 전처리 도구가 IP2Location LITE CSV 파일을 범위 시작값을 키로 하는 `BTreeMap`으로 변환하고, bitcode로 인코딩한 뒤 zstd 레벨 22로 압축합니다.",
            },
            {
              kind: "table",
              columns: ["데이터셋", "CSV", "번들"],
              numericColumns: [1, 2],
              rows: [
                ["IPv4", "312.5 MB", "24.9 MB"],
                ["IPv6", "794.2 MB", "42.4 MB"],
              ],
            },
            {
              kind: "paragraph",
              text: "시작 시 서버는 최대 메모리 사용량을 제한하기 위해 번들을 하나씩 디코딩하고, 반복되는 지명을 인터닝합니다. 조회할 때는 주소 이하에서 시작하는 마지막 범위를 찾아 그 끝 경계를 확인합니다.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "업로드된 [데모 번들](/projects)은 디스크에 임시 저장되고, 매직 바이트로 형식을 식별하며, 압축 해제 후 크기를 50 MiB로 제한하고, 샌드박스 프레임 안에서 상한이 있는 캐시를 통해 제공됩니다. 워크스페이스에서는 블록 깨기 게임과 레이 트레이서를 포함한 Rust 데모도 빌드합니다.",
            },
            {
              kind: "paragraph",
              text: "[EU5 Location DB](/eu5-locations-db)는 WebAssembly로 컴파일한 Slint 애플리케이션입니다. 페이지가 작은 메시지 프로토콜로 라이트·다크 모드를 전달하므로, 내장된 팔레트가 사이트와 함께 바뀝니다.",
            },
          ],
        },
        {
          id: "photography",
          heading: "사진 처리 파이프라인",
          blocks: [
            {
              kind: "paragraph",
              text: "업로드된 이미지는 고정된 픽셀 상한 안에서 디코딩되고, 처리 시간을 더 들이는 대신 파일 크기를 줄이도록 가장 느린 프리셋으로 AVIF 인코딩됩니다. EXIF 데이터의 위치는 게시 전에 지도에서 확인할 수 있으며, 처리 작업은 동시에 최대 두 개까지만 실행됩니다.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "기술 구성",
      title: "성능",
      lede: "백엔드에 프로필 기반 최적화와 BOLT를 적용하고, 병목이 되지 않는 부하 생성기로 측정했습니다.",
      sections: [
        {
          id: "approach",
          heading: "접근 방식",
          blocks: [
            {
              kind: "paragraph",
              text: "최적화 빌드는 대표적인 워크로드로 학습합니다. 계측된 바이너리가 PostgreSQL, SMTP, S3, OpenID Connect, Minecraft 엔드포인트를 합성한 일회용 픽스처를 대상으로 프로필을 기록합니다. LLVM이 이 프로필로 서버를 다시 빌드하고(PGO), 이어서 BOLT가 샘플링한 실행 정보를 바탕으로 최종 바이너리의 코드 배치를 재정렬합니다. 모든 변형은 LTO, 프레임 포인터, 다시 빌드한 표준 라이브러리를 공통으로 사용합니다.",
            },
          ],
        },
        {
          id: "correction",
          heading: "자기 자신을 측정한 벤치마크",
          blocks: [
            {
              kind: "paragraph",
              text: "첫 번째 측정에서는 Playwright로 부하를 걸었고, 성능이 저하된 것처럼 보였습니다. 실제 저하는 아니었습니다: 범위가 지정되지 않은 Playwright API 호출은 부모를 찾기 위해 이전 테스트 단계를 검색하므로, 실행이 진행될수록 클라이언트가 느려졌습니다. 단순한 HTTP 서버를 대상으로 했을 때 클라이언트만으로도 샘플 9개에 걸쳐 초당 2,684개에서 1,314개 요청으로 떨어졌습니다.",
            },
            {
              kind: "paragraph",
              text: "부하 생성은 oha로 옮겼습니다. Playwright는 여전히 각 경로의 응답을 검사하지만, 시간 측정 구간 밖에서 수행합니다.",
            },
          ],
        },
        {
          id: "setup",
          heading: "측정 환경",
          blocks: [
            {
              kind: "list",
              items: [
                "서버, 픽스처 제공자, PostgreSQL은 AMD Ryzen 7 9800X3D의 코어 4개에 고정하고, 부하 클라이언트는 나머지 코어 4개에 고정했습니다.",
                "연결 32개를 풀링한 HTTP/1.1 기반 HTTPS를 사용하고, 응답 전체를 읽으며, 기한은 2초로 설정했습니다.",
                "새 데이터베이스 스냅샷에서 시작해 순서를 바꿔 가며 3라운드를 진행했고, 라운드마다 변형별 샘플 5개를 각각 10,000개의 워밍업 요청 뒤에 측정했습니다.",
                "측정한 요청은 총 27,000,000개이며, 모두 HTTP 200을 반환했습니다.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "결과",
          blocks: [
            {
              kind: "bars",
              caption: "변형별 샘플 15개의 초당 요청 수 중앙값.",
              groups: [
                {
                  label: "10개 경로 공개 읽기",
                  items: [
                    { label: "기준선", value: 44874.12, display: "44,874" },
                    { label: "PGO", value: 52925.46, display: "52,925" },
                    { label: "PGO + BOLT", value: 53888.61, display: "53,889", emphasis: true },
                  ],
                },
                {
                  label: "작은 헬스 체크 JSON",
                  items: [
                    { label: "기준선", value: 272837.41, display: "272,837" },
                    { label: "PGO", value: 308685.26, display: "308,685", emphasis: true },
                    { label: "PGO + BOLT", value: 306276.64, display: "306,277" },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "10개 경로 혼합 부하의 꼬리 지연 시간.",
              columns: ["변형", "p95 (ms)", "p99 (ms)"],
              numericColumns: [1, 2],
              rows: [
                ["기준선", "2.380", "2.967"],
                ["PGO", "2.020", "2.524"],
                ["PGO + BOLT", "1.987", "2.483"],
              ],
            },
            {
              kind: "paragraph",
              text: "PGO는 혼합 처리량을 **17.9%**, PGO와 BOLT는 **20.1%** 높였으며, 두 최적화 빌드 모두 순서를 바꾼 모든 라운드에서 앞섰습니다. BOLT는 혼합 읽기에서 PGO보다 1.8% 더 빨랐고, 단순한 헬스 경로에서는 0.8% 느렸습니다. 이는 4코어 기준 결과이며, 실제 운영 용량은 네트워크, 데이터베이스 작업 집합, 실제 트래픽에도 좌우됩니다.",
            },
          ],
        },
        {
          id: "transport",
          heading: "작은 수정도 중요합니다",
          blocks: [
            {
              kind: "paragraph",
              text: "axum-server의 기본 Rustls acceptor는 `TCP_NODELAY`를 설정하지 않았기 때문에, 헤더 프레임과 데이터 프레임으로 나뉜 HTTP/2 응답이 지연 ACK를 기다릴 수 있었습니다. 수락한 모든 소켓에 이 옵션을 활성화하자, 재사용 연결에서 59,571바이트 타일 응답이 다음과 같이 바뀌었습니다.",
            },
            {
              kind: "table",
              columns: ["TCP_NODELAY", "프로토콜", "첫 바이트 중앙값", "완료 중앙값"],
              numericColumns: [2, 3],
              rows: [
                ["꺼짐", "HTTP/2", "0.290 ms", "40.981 ms"],
                ["켜짐", "HTTP/2", "0.113 ms", "0.152 ms"],
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
