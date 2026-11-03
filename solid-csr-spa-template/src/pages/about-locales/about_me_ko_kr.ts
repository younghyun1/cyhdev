import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "2026년 11월",
  chrome: {
    overview: "개요",
    updated: "마지막 업데이트",
    onThisPage: "이 페이지의 내용",
    sectionNavigation: "소개 페이지",
    previous: "이전",
    next: "다음",
    readingTime: "읽는 데 {minutes}분",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "소개",
      lede: "Rust, PostgreSQL, 클라우드 플랫폼을 다루는 백엔드 및 인프라 엔지니어입니다. 하드웨어 성능을 제대로 끌어내는 서비스를 만들고, 그 서비스가 계속 운영되도록 배포, 모니터링, 보안 작업도 함께 맡습니다.",
      sections: [
        {
          id: "introduction",
          heading: "자기소개",
          blocks: [
            {
              kind: "paragraph",
              text: "저는 콜로라도에 거주하는 한국인 소프트웨어 엔지니어입니다. **Soundpatrol**에서 Rust 엔지니어로 일하며, 주로 Rust, Python, PostgreSQL, Kubernetes, Google Cloud를 사용해 사이버 보안, DevOps, 웹 서비스, 개발자 도구를 다룹니다.",
            },
            {
              kind: "paragraph",
              text: "그 전에는 삼성물산 건물 디지털 트윈의 백엔드와 인프라 작업을 이끌었고, 현대·기아·제네시스 앱 백엔드 개발에 참여했으며, YouTube 분석 플랫폼의 데이터 수집을 구축했습니다. 2023년부터는 백엔드 서비스, 데이터 파이프라인, 이를 뒷받침하는 인프라에 집중해 왔습니다.",
            },
            {
              kind: "paragraph",
              text: "저는 최신 하드웨어와 프로그래밍 언어를 효과적으로 활용하는 서비스를 만드는 것이 중요하다고 생각합니다. 실무에서는 신중한 스키마 설계, 상한을 둔 큐와 캐시, 명시적인 오류 처리, 점검하고 복구할 수 있는 배포로 이어집니다. 동료가 Rust를 익히도록 돕는 일을 즐기며, 자기 일에 자부심을 가진 사람들과 함께 일하고 싶습니다.",
            },
          ],
        },
        {
          id: "results",
          heading: "주요 성과",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "삼성물산 디지털 트윈의 월 클라우드 비용을 약 5,000달러에서 150달러로 절감",
                },
                {
                  value: "80",
                  count: 80,
                  label: "건설 엔지니어링 도메인 로직을 담아 디지털 트윈에 구현한 REST 엔드포인트",
                },
                {
                  value: "9개월",
                  count: 9,
                  suffix: "개월",
                  label: "런타임 중단 없이 이어진 디지털 트윈의 운영 기간",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "[PGO와 BOLT](/about-blog/performance)로 얻은 이 사이트 백엔드의 혼합 읽기 처리량 향상",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "멀티스테이지 빌드로 만든 정적 링크 서비스 컨테이너",
                },
                {
                  value: "30,000",
                  label: "디지털 트윈 백엔드에서 작성한 코드 줄 수, 전체 코드의 약 75%",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "사용 기술",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "언어",
                  items: ["Rust", "Python", "TypeScript", "SQL", "Java", "C", "C++"],
                },
                {
                  label: "백엔드",
                  items: ["Axum", "Tokio", "Diesel", "tokio-postgres", "Spring Boot", "FastAPI", "WebSockets", "gRPC", "Protocol Buffers"],
                },
                {
                  label: "데이터",
                  items: ["PostgreSQL", "Valkey / Redis", "SQLite / Turso", "MySQL", "FAISS", "tantivy"],
                },
                {
                  label: "인프라",
                  items: ["Kubernetes", "Helm", "Docker", "OpenTofu / Terraform", "GitHub Actions", "Google Cloud", "AWS", "Linux"],
                },
                {
                  label: "안정성 및 보안",
                  items: ["OpenTelemetry", "IAM", "TLS", "Network policy", "Credential lifecycle", "Incident response"],
                },
                {
                  label: "인터페이스 및 장치",
                  items: ["SolidJS", "Leptos", "Slint", "WebAssembly", "WebRTC", "BACnet", "Modbus"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "더 알아보기",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "경력",
                  href: "/about/experience",
                  meta: "2016년 - 현재",
                  text: "Soundpatrol부터 첫 인턴십까지의 경력과, 각 역할에서 진행한 프로젝트와 성과를 정리했습니다.",
                },
                {
                  title: "프로젝트",
                  href: "/about/projects",
                  meta: "Rust · 오픈소스",
                  text: "Sillok, EU5 Location DB, 이 웹사이트를 비롯한 개인 프로젝트와 오픈소스 작업입니다.",
                },
                {
                  title: "배경",
                  href: "/about/background",
                  meta: "학력 · 연구",
                  text: "학력, 발표 논문, 수상, 언어, 소프트웨어 밖의 활동입니다.",
                },
                {
                  title: "이 사이트의 작동 방식",
                  href: "/about-blog",
                  meta: "기술 구성",
                  text: "cyhdev.com을 이루는 호스트, 백엔드, 프런트엔드, 하위 시스템, 성능 측정을 다룹니다.",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "연락처",
          blocks: [
            {
              kind: "list",
              items: [
                "**이메일**: [younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**: [github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**: [linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "가장 확실한 연락 방법은 이메일입니다. Minecraft 서버에 참여하고 싶을 때도 이메일로 알려 주세요.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "소개",
      title: "경력",
      lede: "미국 스타트업, 한국 기업 고객, 초기 인턴십에서 맡은 백엔드, 인프라, 보안 업무와, 소프트웨어 이전에 했던 일을 정리했습니다.",
      sections: [
        {
          id: "software",
          heading: "소프트웨어 엔지니어링",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Rust 엔지니어",
                  period: "2026년 5월 - 현재",
                  location: "미국 · 원격",
                  summary: "음악 및 오디오 탐지 기업에서 사이버 보안, DevOps, 웹 서비스 개발, 개발자 도구를 담당합니다.",
                  highlights: [
                    "Google Cloud의 Kubernetes 인프라와 코드형 인프라, 그리고 수명이 짧고 환경별로 범위가 제한된 자격 증명을 사용하는 검토된 배포 워크플로를 유지 관리합니다.",
                    "운영 데이터 저장소 마이그레이션을 계획하고 수행하며, 각 전환 전에 마이그레이션된 상태를 검증하고 복구 절차를 리허설합니다.",
                    "메트릭, 로그, 트레이스를 OpenTelemetry로 통합하고, 서비스 상태, 진단, 비용 가시성을 위한 Rust 도구를 만듭니다.",
                    "보안 준비 태세를 갖추는 작업을 합니다: 접근 권한 관리, 자격 증명 교체와 만료 추적, 정책, 사고 대응, 감사 증빙.",
                    "운영 및 지원 워크플로를 위한 내부 Rust·Python 서비스와 도구를 만듭니다.",
                  ],
                  tags: ["Rust", "Python", "PostgreSQL", "Kubernetes", "Google Cloud", "OpenTofu", "OpenTelemetry"],
                },
                {
                  organization: "GenesisNest",
                  title: "소프트웨어 엔지니어",
                  period: "2025년 1월 - 2025년 7월",
                  location: "대한민국 성남",
                  summary: "사내 플랫폼과 K-pop 그룹 공식 팬 앱을 위한 백엔드 서비스와 인프라 도구를 개발했습니다.",
                  highlights: [
                    "현대·기아·제네시스 공식 앱 백엔드에 API, 버그 수정, 연동 작업으로 기여했습니다. 대규모 Java 코드베이스에서 일하며, 아시아와 유럽의 수천만 명이 사용하는 서비스를 위해 협력사와 조율했습니다.",
                    "번역된 콘텐츠를 변환해 기업 데이터베이스에 적재하는 다국어화 파이프라인을 Rust와 Python으로 구축했으며, 공장 시스템과 연결된 데이터셋을 담당하는 데이터베이스 관리자와 협업했습니다.",
                    "사내 서비스를 위한 인프라 관리 도구를 개발했습니다.",
                  ],
                  tags: ["Java", "Spring Boot", "Rust", "Python"],
                },
                {
                  organization: "pampam Inc.",
                  title: "계약직 소프트웨어 엔지니어",
                  period: "2024년 11월 - 2024년 12월",
                  location: "대한민국 서울",
                  highlights: [
                    "AI 기반 YouTube 분석 플랫폼의 PostgreSQL 스키마와 Rust 백엔드를 설계하고, AWS와 Google Cloud에 걸친 배포를 주도했습니다.",
                    "채널 데이터, 자막, 페이지 콘텐츠를 결합해 수천만 건의 댓글과 사용자를 아우르는 YouTube API 수집 워크플로를 구축했습니다.",
                    "호스팅형 언어 모델과 서버 내 오픈소스 모델을 연동해 요약과 마케팅 인사이트를 생성했습니다.",
                  ],
                  tags: ["Rust", "PostgreSQL", "AWS", "Google Cloud"],
                },
                {
                  organization: "EAN Technology Co. Ltd.",
                  title: "백엔드 소프트웨어 엔지니어 (리드)",
                  period: "2023년 8월 - 2024년 8월",
                  location: "대한민국 서울",
                  summary: "두 건물·에너지 플랫폼의 백엔드, 데이터베이스, 클라우드 인프라 개발을 이끌었습니다.",
                  highlights: [],
                  projects: [
                    {
                      name: "삼성물산 글로벌 엔지니어링 센터 디지털 트윈",
                      highlights: [
                        "Express 백엔드와 클라우드 마이크로서비스 구성을 최적화된 Rust/Axum 모놀리스로 교체해, 월 인프라 비용을 약 **5,000달러에서 150달러로** 줄였습니다.",
                        "백엔드의 약 75%에 해당하는 약 **30,000줄**을 작성했으며, 여기에는 건설 엔지니어링 도메인 로직, 데이터 통합, 실시간 운영을 담은 **REST 엔드포인트 80개**가 포함됩니다.",
                        "비동기 Rust, 멀티스레딩, 캐싱, PostgreSQL 스키마 및 쿼리 튜닝으로, 2코어, 4 GB 환경에서 수 초에 달하던 여러 P99 요청 경로를 한두 자릿수 밀리초로 낮췄습니다.",
                        "삼성물산 본사 현장에서 BACnet과 Modbus로 수천 개의 센서, 계량기, HVAC 및 수도 설비 제어기를 연동했습니다.",
                        "약 90%의 오류 처리 커버리지를 갖추고, 운영 환경에서 9개월간 런타임 중단 없이 가동했습니다.",
                        "GitHub 러너, AWS CodeDeploy, Docker로 CI/CD를 구축했으며, 프로세서에 맞춰 튜닝한 정적 빌드로 약 20 MB의 이식 가능한 컨테이너를 만들었습니다.",
                        "ECS, EC2, SQS, SNS, S3, SageMaker를 통해 예측 모델 추론을 연결하고, 삼성SDS 보안 요구 사항을 구현했으며, 협력 스튜디오가 만든 Unreal Engine 4 인터페이스와 백엔드를 연동했습니다.",
                      ],
                    },
                    {
                      name: "AI 에너지 관리 서비스",
                      highlights: [
                        "PostgreSQL 스키마와 Rust/Axum API를 처음부터 설계하고, 백엔드의 약 70%에 해당하는 약 **25,000줄**을 작성했습니다.",
                        "디지털 트윈에서 얻은 교훈을 바탕으로, 쿼리와 컴파일 시간을 직접 통제하기 위해 ORM 대신 tokio-postgres로 SQL을 직접 사용했습니다.",
                        "백엔드, 프런트엔드, 데이터 과학 엔지니어로 구성된 9인 팀에서, Node.js와 Java 출신 엔지니어 두 명이 약 한 달 만에 Rust로 생산적으로 일할 수 있도록 도왔습니다.",
                      ],
                    },
                  ],
                  tags: ["Rust", "Axum", "PostgreSQL", "AWS", "Docker", "BACnet", "Modbus"],
                },
                {
                  organization: "Artifyc Inc.",
                  title: "소프트웨어 엔지니어 인턴",
                  period: "2022년 8월 - 2023년 3월",
                  location: "미국 텍사스주 오스틴 · 원격",
                  highlights: [
                    "Reddit API로 그림 의뢰 요청을 찾아 커뮤니티 채널에 게시하는 Python Discord 봇을 만들고, AWS Lambda에서 5분마다 실행되도록 예약했습니다.",
                    "관할 지역별 통화 변환, 요금 청구, 추가 요금 계산을 위한 Lambda 로직을 구현했습니다.",
                    "React 프런트엔드의 레이아웃, 에셋, 문구 수정에 기여했습니다.",
                  ],
                  tags: ["Python", "AWS Lambda", "React"],
                },
              ],
            },
          ],
        },
        {
          id: "other",
          heading: "소프트웨어 이전의 경력",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "해양수산과학기술진흥원",
                  title: "번역가 겸 통역사",
                  period: "2023년 6-7월, 2024년 8월",
                  location: "대한민국 서울 · 계약직",
                  highlights: [
                    "주한 노르웨이 대사, 노르웨이 차관, 기관 경영진, 동원그룹 회장이 참석한 해양 연구 회의에서 영어, 한국어, 노르웨이어 통역을 맡았습니다.",
                    "국제회의와 공동 프로젝트 관련 미국 에너지부 행사를 앞두고 연설문을 번역하고, 기관 직원과 원장을 지도했습니다.",
                  ],
                },
                {
                  organization: "대한민국 육군 · 미 육군 제2보병사단",
                  title: "병장, KATUSA 연락병 겸 교관",
                  period: "2016년 3월 - 2017년 12월",
                  location: "대한민국 동두천",
                  highlights: [
                    "[KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army) 제도를 통해 한미 연합 환경에서 사단 본부대대에 복무했습니다.",
                    "약 6,000명의 미국 및 한국 인원을 대상으로 체력, 문화, 역사, 예방 교육을 진행하고 전입 절차를 처리했습니다.",
                    "13명 규모의 팀을 이끌며 한반도의 [2017년 위기](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) 동안 작전 태세를 유지했습니다.",
                  ],
                },
                {
                  organization: "쿠팡",
                  title: "물류 상하차 직원",
                  period: "2020년 - 2022년",
                  location: "대한민국 · 비정기 계약",
                  highlights: [
                    "대학 재학과 병행해 물류 창고에서 야간 근무를 했습니다.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "소개",
      title: "프로젝트",
      lede: "주로 Rust로 진행한 개인 및 오픈소스 작업입니다. 인터랙티브 데모는 [프로젝트 페이지](/projects)에 있습니다.",
      sections: [
        {
          id: "featured",
          heading: "대표 프로젝트",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "명령줄 작업 기록 도구입니다. 목표, 완료한 작업, 정정, 철회를 추가 전용 이벤트로 기록하고 인덱싱된 현재 상태 프로젝션을 유지하며, Git 기반 아카이브로 여러 기기의 이력을 병합합니다.",
                  tags: ["이벤트 소싱", "동기화"],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Europa Universalis V 지도 데이터를 위한 데스크톱 및 브라우저 애플리케이션으로, 압축 데이터셋 내장, 사전 계산된 검색 및 정렬 인덱스, 범주형 필터, 크기 조절 가능한 표를 갖췄습니다. [여기서 사용해 보세요](/eu5-locations-db).",
                  tags: ["네이티브 + 웹", "인덱스"],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "이 사이트입니다. 글 게시, 사진, 포럼, 통화 기능이 있는 라이브 채팅, Minecraft 지형 탐색기, WebAssembly 데모를 하나의 Rust 바이너리로 제공합니다. [구축 방식 보기](/about-blog).",
                  tags: ["모노레포", "셀프 호스팅"],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "터미널 및 파일 출력, 선택적 중복 검사, Python 및 JSON 호환 형식을 지원하는 UUIDv4 생성기 겸 포매터입니다. 초기 로컬 벤치마크에서 uuidgen보다 약 세 배 빨랐습니다.",
                  tags: ["처리량"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "기타 작업",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: Rust Minecraft 서버에서 월드를 가져오고 저장할 때 Paper 오버월드 설정을 보존하는 [업스트림 수정](https://github.com/Pumpkin-MC/Pumpkin/pull/3447)으로, 현재 리뷰 대기 중입니다.",
                "**Minecraft 지도 도구**: 이 사이트의 [지형 탐색기](/minecraft)를 뒷받침하는 Paper 플러그인과 Rust 서비스로, 간결한 바이너리 바이옴 타일 형식과 시드 기반 바이옴 예측을 포함합니다.",
                "**파일 시스템 동기화**: 이기종 파일 시스템 간 양방향 폴더 동기화와 빌드 산출물 정리를 위한 Rust 유틸리티로, 드라이 런, 작업 저널, 선택적 체크섬, 의도치 않은 대량 삭제 방지 장치를 갖췄습니다.",
                "**impulsr**: 마케팅 업무를 위한 언어 모델 요약을 생성하는 YouTube 자막 및 댓글 수집 도구입니다.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "작업 방식",
          blocks: [
            {
              kind: "paragraph",
              text: "작고 점검하기 쉬운 시스템을 선호합니다. 바이너리 하나로 충분하면 바이너리 하나만 쓰고, 모든 큐와 캐시에 명시적인 상한을 두며, 데이터 형식은 습관이 아니라 접근 패턴에 맞춰 고릅니다.",
            },
            {
              kind: "paragraph",
              text: "주장보다 측정이 먼저입니다. 벤치마크가 서버가 아니라 자기 클라이언트를 측정하고 있었다고 밝혀지면 그 정정도 기록합니다. 이 사이트의 [성능 기록](/about-blog/performance)이 그 예입니다.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "소개",
      title: "배경",
      lede: "학력, 연구, 자격, 그리고 소프트웨어 밖의 관심사입니다.",
      sections: [
        {
          id: "education",
          heading: "학력",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "성균관대학교",
                  title: "소프트웨어공학 공학사",
                  period: "2015년 3월 - 2023년 8월",
                  location: "대한민국 수원",
                  summary: "컴퓨터공학과 소속이며, 학위 기간에는 의무 군 복무 기간이 포함되어 있습니다.",
                  highlights: [
                    "소프트웨어 공학 실무와 함께 임베디드 소프트웨어, 논리 회로, 운영체제, 네트워크, 저수준 프로그래밍, 전자공학에 걸친 과목을 이수했습니다.",
                    "성균관대학교는 1398년 조선 왕조의 최고 교육 기관에 기원을 두며, 한국을 대표하는 연구 중심 대학 중 하나입니다.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "연구",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Korean Language NLP Model Based Emotional Analysis of Social Media Communities.** KoBERT를 적용해 한국어 소셜 미디어의 감성 분석과 분류를 수행하고, COVID-19 초기의 감정 변화를 살펴보았습니다.",
            },
            {
              kind: "paragraph",
              text: "이 프로젝트는 약 60개 작품이 출품된 학교 졸업 논문 및 작품 시상에서 동상을 받았습니다.",
            },
          ],
        },
        {
          id: "awards",
          heading: "수상 및 자격",
          blocks: [
            {
              kind: "table",
              columns: ["연도", "항목", "내용"],
              rows: [
                ["2023", "**동상**", "성균관대학교 졸업 논문 및 작품 시상"],
                ["2023", "**IELTS 8.5 / 9.0**", "CEFR C2"],
                ["2021", "**7위**", "한국인터넷정보학회 캡스톤 디자인 및 아이디어 해커톤"],
                ["2020", "**3위**", "서울과학기술대학교 핀테크 해커톤"],
                ["2018", "**TOEFL 117 / 120**", "CEFR C2"],
              ],
            },
          ],
        },
        {
          id: "languages",
          heading: "언어",
          blocks: [
            {
              kind: "list",
              items: [
                "**한국어**: 모국어.",
                "**영어**: 업무 수준의 유창성 (C2).",
                "**프랑스어, 독일어, 스페인어, 중국어**: 기초 수준.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "소프트웨어 밖의 활동",
          blocks: [
            {
              kind: "list",
              items: [
                "**사진**: 2010년부터 풍경과 인물 사진을 찍어 왔으며, 약 30,000장의 사진을 보관하고 있습니다. 일부는 [갤러리](/photographs)에서 볼 수 있습니다.",
                "**언론 활동**: 성균관대학교 영자 잡지 Sungkyun Times의 학생 기자로 기술, 보안, 국제 문제를 취재했습니다.",
                "**교육 및 봉사**: 지역 커뮤니티 센터에서 영어를 가르쳤고, 장애인이 운영하는 중고품 매장에서 자원봉사를 했습니다.",
                "**번역**: 공공 기관을 위해 통역과 번역을 했습니다. [경력](/about/experience#other)을 참고하세요.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
