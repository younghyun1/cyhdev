// English proper names, product names, technologies, and publication titles remain in their official form.
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
    <main lang="ko-KR" class={`${pageStyles.page} about-page`}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">소개</h1>
          <p class="text-xs text-ink-muted">
            마지막 업데이트: 2026년 9월
          </p>
        </div>

        <section class="mb-12">
          <h2 class="text-xl font-bold mb-4 border-b border-line pb-2">
            1. 소개
          </h2>

          <div class="mb-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div>
              <h3 class="text-lg font-semibold">
                Young Hyun Chi / 지영현 / 池營賢 / 池营贤
              </h3>

              <p class="text-ink-muted">
                백엔드 및 인프라 엔지니어 · Rust / 플랫폼 안정성 / 보안
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
                  alt="인물 사진"
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
            <h3 class="font-bold mb-2">연락처</h3>
            <ul class="list-none space-y-2 text-sm">
              <li>
                <span class="w-20 inline-block font-medium text-ink-muted">
                  이메일:
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
            저는 콜로라도에 거주하는 백엔드 엔지니어로, Rust 서비스와 PostgreSQL 데이터베이스, 클라우드 인프라를 전문으로 합니다. Soundpatrol에서 Rust, Python, PostgreSQL, Kubernetes, GCP를 사용해 사이버 보안, DevOps, 웹 서비스, 개발자 도구를 다룹니다.
          </p>
          <p class="mb-4 leading-relaxed">
            이전 프로젝트로는 AWS에 배포한 삼성물산의 수천 개 환경 센서를 통합하는 디지털 트윈, 자막·댓글·언어 모델을 활용한 YouTube 채널 분석 플랫폼, 현대·기아·제네시스 앱 및 K-pop 그룹 팬 앱의 백엔드와 데이터 파이프라인이 있습니다.
          </p>
          <p class="mb-4 leading-relaxed">
            저는 최신 하드웨어와 프로그래밍 언어를 효과적으로 활용하는 서비스를 만드는 것이 중요하다고 생각합니다. 클라우드 자원을 엔지니어링의 무한한 대체재로 여기기보다 동시성, 데이터베이스 쿼리, 메모리 사용, 배포의 세부 사항을 다루는 일을 즐깁니다. 직접 서비스를 호스팅하고 Rust 도구를 만들며 동료의 Rust 학습을 돕는 일도 좋아합니다.
          </p>
          <p class="mb-4 leading-relaxed">
            저는 여러 분야에서 다양한 문화와 배경을 가진 동료들과 일한 경험을 자랑스럽게 생각합니다. 2016년부터 2017년까지 21개월 동안 미 육군 제2보병사단 소속으로 북한 접경 지역에서 미국 및 한국 군인들과 함께 복무했습니다. 대학 시절에는 언론, 사진, 번역 업무와 야간 창고·물류 업무를 했습니다. 2023년부터는 백엔드 엔지니어링, 데이터 파이프라인, 이를 뒷받침하는 인프라에 집중하고 있습니다.
          </p>
        </section>

        {/* 2) Professional Career */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            2. 개발자 경력
          </h2>

          <div class="space-y-8">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Soundpatrol</h3>
                <span class="text-sm font-mono text-ink-muted">미국 · 원격 | 2026년 5월 - 현재</span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">Rust 엔지니어</p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>Rust와 Python으로 웹 서비스와 내부 도구를 개발합니다.</li>
                <li>클라우드 인프라, Kubernetes 배포, CI/CD 파이프라인을 운영합니다.</li>
                <li>모니터링, 접근 제어, 자격 증명 관리 등 사이버 보안 대책을 다룹니다.</li>
              </ul>
            </div>

            {/* GenesisNest */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">GenesisNest</h3>
                <span class="text-sm font-mono text-ink-muted">
                  대한민국 성남 | 2025년 1월 - 2025년 7월
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                소프트웨어 엔지니어
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  사내 서비스와 K-pop 그룹 공식 앱을 위한 인프라 관리 도구를 개발했습니다. Spring Boot 생태계에 익숙해졌습니다.
                </li>
                <li>
                  현대자동차의 현대·기아·제네시스 공식 앱 백엔드 개발 계약 업무를 맡아 대규모 Java 코드베이스에 참여했습니다.
                </li>
                <li>
                  아시아와 유럽의 수천만 사용자가 이용하는 서비스의 API와 버그 수정 작업을 수행하고 협력사와 조율했습니다.
                </li>
                <li>
                  다국어화 작업을 위한 데이터 파이프라인을 구축하고, 번역된 콘텐츠를 현대 데이터베이스에 통합하는 과정을 자동화하는 Rust 및 Python 스크립트를 작성했습니다. 회사 공장 시스템과 연결된 대규모 데이터셋을 담당하는 사내 DBA와 협업했습니다.
                </li>
              </ul>
            </div>

            {/* pampam */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">pampam Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  대한민국 서울 | 2024년 11월 - 2024년 12월
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                계약직 소프트웨어 엔지니어
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  AI 기반 YouTube 분석 플랫폼의 PostgreSQL 스키마와 Rust 백엔드를 설계하고 AWS 및 GCP 배포를 주도했습니다.
                </li>
                <li>
                  YouTube API를 통한 대규모 데이터 수집(수천만 건의 댓글 및 사용자)을 이끌었습니다.
                </li>
                <li>
                  LLM과 서버 내 오픈소스 모델을 활용해 인사이트와 요약을 생성했습니다.
                </li>
              </ul>
            </div>

            {/* EAN Technology */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">EAN Technology Co. Ltd</h3>
                <span class="text-sm font-mono text-ink-muted">
                  대한민국 서울 | 2023년 8월 - 2024년 8월
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                백엔드 소프트웨어 엔지니어 (리드)
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>삼성물산 디지털 트윈 프로젝트:</b> 제대로 작동하지 않던 마이크로서비스 아키텍처를 종료하고 고도로 최적화된 Axum (Rust) 모놀리스로 교체했습니다. 월 클라우드 비용을 약 5,000달러에서 약 150달러로 줄였습니다.
                </li>
                <li>
                  약 3만 줄(전체 코드의 약 75%)을 작성하고 복잡한 도메인 로직을 포함한 엔드포인트 80개를 구현했습니다.
                </li>
                <li>
                  9개월간 운영하면서 런타임 종료 0건을 달성하고 약 90%의 오류 처리 범위를 확보했습니다.
                </li>
                <li>
                  삼성물산 본사의 수천 개 실시간 물리 센서(BACnet, Modbus)와 사내 서버를 연동했습니다.
                </li>
                <li>
                  PostgreSQL 쿼리와 스키마를 최적화해 제한된 하드웨어(2코어, RAM 4GB)에서 일부 비최적화 쿼리의 수 초에 달하던 P99 지연 시간을 두 자릿수 밀리초로 낮췄습니다.
                </li>
                <li>
                  GitHub Runners, AWS CodeDeploy, Docker를 사용해 CI/CD를 구현했습니다. MUSL, 정적 링크, scratch 이미지를 이용한 멀티스테이지 빌드로 보안과 콜드 스타트 효율성을 높인 약 20MB Docker 이미지를 만들었습니다.
                </li>
                <li>
                  동료들의 Rust 학습을 도와 두 달 안에 Node.js/Java에서 전환할 수 있도록 했습니다.
                </li>
              </ul>
            </div>

            {/* Artifyc */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Artifyc Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  미국 텍사스주 오스틴 (원격) | 2022년 8월 - 2023년 3월
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                소프트웨어 엔지니어 인턴
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Reddit API를 활용해 그림 의뢰를 수집하고 게시하는 Python Discord 봇을 구현했습니다.
                </li>
                <li>
                  통화 변환과 추가 요금 계산을 위한 서버리스 백엔드 로직을 AWS Lambda에 개발했습니다.
                </li>
                <li>
                  React 프런트엔드 수정과 에셋 통합을 지원했습니다.
                </li>
              </ul>
            </div>
          </div>
        </section>


        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">주요 프로젝트</h2>
          <ul class="list-disc list-inside text-sm space-y-3 text-ink">
            <li><a href="https://github.com/younghyun1/sillok" class={pageStyles.link}>Sillok</a>: 구조화된 작업 기록을 위한 Rust CLI로, 추가 전용 이력, 인덱싱된 Turso/SQLite 프로젝션, 수정 기능, Git 기반 기기 간 동기화를 제공합니다.</li>
            <li><a href="https://github.com/younghyun1/eu5-location-filter" class={pageStyles.link}>EU5 Location DB</a>: 압축 지도 데이터 내장, 사전 계산 인덱스, 검색 가능한 필터, 크기 조절 표를 갖춘 Rust 및 Slint 데스크톱·WebAssembly 애플리케이션입니다.</li>
            <li><a href="https://github.com/younghyun1/cyhdev" class={pageStyles.link}>cyhdev.com</a>: 이 Rust/Axum, PostgreSQL, SolidJS 웹사이트로 글 게시, 사진, 토론, 채팅, 브라우저 데모를 제공합니다.</li>
            <li><a href="https://github.com/younghyun1/oohid" class={pageStyles.link}>oohid</a>: 형식 설정, 파일 출력, 선택적 중복 검사를 지원하는 Rust UUID 생성기입니다.</li>
          </ul>
        </section>

        {/* 3) Non-IT Career */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            3. 비 IT 경력
          </h2>

          <div class="space-y-6">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">
                  한국해양과학기술원
                </h3>
                <span class="text-sm font-mono text-ink-muted">
                  대한민국 서울 | 2023년 6-7월, 2024년 8월
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                번역가 / 통역사
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  노르웨이 대사와 동원그룹 회장이 참석한 고위급 회의에서 영어, 한국어, 노르웨이어 통역을 맡았습니다.
                </li>
                <li>
                  해양 연구 개발과 관련된 국가 및 기업의 민감한 거래와 논의 내용을 신뢰받고 담당했습니다.
                </li>
                <li>
                  미국 에너지부 행사에 참여하는 KIMST 직원과 원장을 지도했습니다.
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
                    US Army
                  </a>{" "}
                  (
                  <a
                    href="https://en.wikipedia.org/wiki/2nd_Infantry_Division_(United_States)"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    2nd Infantry Division
                  </a>
                  ) /{" "}
                  <a
                    href="https://en.wikipedia.org/wiki/Republic_of_Korea_Army"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    ROK Army
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
                  , South Korea | Mar 2016 - Dec 2017
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
                | 교관 | 병장
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  연락병 겸 교관으로 복무했습니다. 약 6,000명의 미국 및 한국 인원을 교육하고 행정 절차에 따라 처리했습니다.
                </li>
                <li>
                  체력, 문화, 역사, 안전 및 예방 교육을 진행했습니다.
                </li>
                <li>
                  다음 기간 동안 작전 태세를 유지했습니다:{" "}
                  <a
                    href="https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    2017 crisis
                  </a>
                  ; 13명으로 구성된 팀을 이끌었습니다.
                </li>
              </ul>
            </div>

            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Coupang Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  대한민국 | 2020년 - 2022년
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                물류 상하차 직원
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  야간 근무로 물류 창고에서 고강도 육체 노동을 했습니다.
                </li>
              </ul>
            </div>
          </div>
        </section>

        {/* 4) Academics */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            4. 학력
          </h2>

          <div class="mb-6">
            <div class="flex flex-wrap justify-between items-baseline mb-2">
              <h3 class="text-lg font-bold">성균관대학교</h3>
              <span class="text-sm font-mono text-ink-muted">
                대한민국 수원 | 2015년 3월 - 2023년 8월
              </span>
            </div>
            <p class="text-sm font-medium mb-2 text-ink-muted">
              B. Eng in 소프트웨어 엔지니어ing
            </p>
            <div class="space-y-2 text-sm text-ink">
              <p>
                컴퓨터 과학, 웹 및 임베디드 소프트웨어공학, 전자공학을 아우르는 종합 과정을 이수했습니다. 논리학, 저수준 언어, 운영체제, 네트워크 공학과 소프트웨어 공학 실무에 중점을 두었습니다.
              </p>
              <p>
                성균관대학교는 한국에서 가장 오래된 대학으로, 유교 전통에 따라 조선의 국가 관료 양성 기관으로 1398년에 설립되었습니다. 광복 이후 한국의 주요 학술·역사 기관으로 자리 잡았으며, 제가 재학한 이래 세계 100대 대학에 꾸준히 선정되고 있습니다.
              </p>
            </div>
          </div>

          <div>
            <h4 class="font-bold mb-2 text-md">논문</h4>
            <ul class="list-disc list-inside text-sm space-y-2 text-ink">
              <li>
                <b>
                  한국어 자연어 처리 모델 기반 소셜 미디어 커뮤니티 감성 분석
                </b>{" "}
                (IMCOM 2023, IEEE Xplore).
                <br />
                <span class="ml-5 block text-xs text-ink-muted mt-1">
                  COVID 팬데믹 초기의 소셜 미디어 데이터를 KoBERT로 감성 분석했습니다. 졸업 작품에서 동상을 수상했습니다.
                </span>
              </li>
            </ul>
          </div>
        </section>

        {/* 5) Volunteer work and interests */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            5. 봉사 활동 및 관심사
          </h2>

          <div class="grid md:grid-cols-2 gap-8">
            <div>
              <h3 class="font-bold mb-3 text-lg">개인 프로젝트</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>
                  <b>oohid:</b> Rust로 만든 CLI UUIDv4 생성기입니다. libuuid보다 약 3배 빠르며, 중복 검사와 Python/JSON 형식 출력을 지원합니다.
                </li>
                <li>
                  <b>impulsr:</b> 마케팅 요약을 위한 YouTube 자막 및 댓글 수집 도구입니다.
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-3 text-lg">봉사 활동</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>지역 커뮤니티 센터에서 영어를 가르쳤습니다.</li>
                <li>장애인이 운영하는 중고품 매장에서 자원봉사했습니다.</li>
              </ul>
            </div>
          </div>
        </section>

        {/* 6) Hobbies */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            6. 취미
          </h2>

          <div class="space-y-4">
            <div>
              <h3 class="font-bold inline mr-2">사진</h3>
              <span class="text-sm text-ink-muted">
                2010년부터 풍경 및 인물 사진을 찍어 온 아마추어 사진가입니다. 약 3만 장의 사진을 보관하고 있습니다.
              </span>
              <div class="mt-2">
                <a
                  href="/photographs"
                  class={`inline-flex items-center ${pageStyles.link} font-medium`}
                >
                  사진 포트폴리오 보기 →
                </a>
              </div>
            </div>

            <div>
              <h3 class="font-bold inline mr-2">언론 활동</h3>
              <p class="text-sm text-ink mt-1">
                성균관대학교 영자 신문 Sungkyun Times의 학생 기자로 활동했습니다.
              </p>
            </div>
          </div>
        </section>

        {/* 7) Qualifications and Other Skills */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            7. 자격 및 기타 기술
          </h2>

          <div class="grid md:grid-cols-2 gap-8 mb-8">
            <div>
              <h3 class="font-bold mb-2">자격 및 수상</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>TOEFL:</b> 117/120 (CEFR C2)
                </li>
                <li>
                  <b>IELTS:</b> 8.5/9.0 (CEFR C2)
                </li>
                <li>
                  <b>7위:</b> 2021 캡스톤 디자인 및 아이디어 해커톤
                </li>
                <li>
                  <b>3위:</b> 2020 핀테크 해커톤 (SNUST)
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-2">언어</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>한국어 (모국어)</li>
                <li>영어 (C2)</li>
                <li>기초 프랑스어, 독일어, 스페인어, 중국어</li>
              </ul>
            </div>
          </div>
        </section>
      </section>
    </main>
  );
}
