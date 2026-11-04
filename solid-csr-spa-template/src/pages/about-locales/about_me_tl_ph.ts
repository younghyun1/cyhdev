import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "Nobyembre 2026",
  chrome: {
    overview: "Pangkalahatang-ideya",
    updated: "Huling na-update",
    onThisPage: "Sa pahinang ito",
    sectionNavigation: "Mga pahina ng Tungkol",
    previous: "Nakaraan",
    next: "Susunod",
    readingTime: "{minutes} min na pagbasa",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "Tungkol",
      lede: "Backend at infrastructure engineer na gumagamit ng Rust, PostgreSQL, at mga cloud platform. Gumagawa ako ng mga serbisyong mahusay na gumagamit ng hardware sa ilalim ng mga ito, kasama ang gawaing deployment, monitoring, at seguridad na nagpapanatili sa kanilang tumatakbo.",
      sections: [
        {
          id: "introduction",
          heading: "Pagpapakilala",
          blocks: [
            {
              kind: "paragraph",
              text: "Isa akong Koreanong software engineer na nakabase sa Colorado. Sa **Soundpatrol**, nagtatrabaho ako bilang Rust Engineer sa cybersecurity, DevOps, mga webservice, at developer tooling, pangunahin gamit ang Rust, Python, PostgreSQL, Kubernetes, at Google Cloud.",
            },
            {
              kind: "paragraph",
              text: "Bago nito, pinamunuan ko ang backend at infrastructure para sa isang digital twin ng gusali ng Samsung C&T, nag-ambag sa mga backend ng application ng Hyundai, Kia, at Genesis, at gumawa ng data collection para sa isang YouTube analytics platform. Mula 2023, nakatuon ang trabaho ko sa mga backend service, data pipeline, at sa infrastructure sa ilalim ng mga ito.",
            },
            {
              kind: "paragraph",
              text: "Naniniwala akong dapat gumawa ng mga serbisyong mahusay na gumagamit ng makabagong hardware at mga programming language. Sa praktika, ibig sabihin nito ay maingat na disenyo ng schema, may hangganang queue at cache, tahasang paghawak ng error, at mga deployment na maaaring siyasatin at ibalik sa dating ayos. Masaya akong tumulong sa mga kasamahan na matuto ng Rust, at gusto kong makatrabaho ang mga taong ipinagmamalaki ang kanilang gawa.",
            },
          ],
        },
        {
          id: "results",
          heading: "Mga piling resulta",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "Mas mababa ang buwanang gastos sa cloud ng digital twin ng Samsung C&T, mula humigit-kumulang $5,000 hanggang $150",
                },
                {
                  value: "80",
                  count: 80,
                  label: "Mga REST endpoint na ipinatupad para sa digital twin, kasama ang domain logic ng construction engineering",
                },
                {
                  value: "9 buwan",
                  count: 9,
                  suffix: " buwan",
                  label: "Operasyon ng digital twin sa production nang walang runtime shutdown",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "Dagdag sa mixed-read throughput ng backend ng site na ito mula sa [PGO at BOLT](/about-blog/performance)",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Mga statically linked na service container mula sa multi-stage build",
                },
                {
                  value: "30,000",
                  label: "Mga linyang isinulat para sa backend ng digital twin, humigit-kumulang 75% ng code nito",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Mga gamit",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Mga wika",
                  items: [
                    "Rust",
                    "Python",
                    "TypeScript",
                    "SQL",
                    "Java",
                    "C",
                    "C++",
                  ],
                },
                {
                  label: "Backend",
                  items: [
                    "Axum",
                    "Tokio",
                    "Diesel",
                    "tokio-postgres",
                    "Spring Boot",
                    "FastAPI",
                    "WebSockets",
                    "gRPC",
                    "Protocol Buffers",
                  ],
                },
                {
                  label: "Data",
                  items: [
                    "PostgreSQL",
                    "Valkey / Redis",
                    "SQLite / Turso",
                    "MySQL",
                    "FAISS",
                    "tantivy",
                  ],
                },
                {
                  label: "Infrastructure",
                  items: [
                    "Kubernetes",
                    "Helm",
                    "Docker",
                    "OpenTofu / Terraform",
                    "GitHub Actions",
                    "Google Cloud",
                    "AWS",
                    "Linux",
                  ],
                },
                {
                  label: "Pagiging maaasahan at seguridad",
                  items: [
                    "OpenTelemetry",
                    "IAM",
                    "TLS",
                    "Network policy",
                    "Credential lifecycle",
                    "Incident response",
                  ],
                },
                {
                  label: "Mga interface at device",
                  items: [
                    "SolidJS",
                    "Leptos",
                    "Slint",
                    "WebAssembly",
                    "WebRTC",
                    "BACnet",
                    "Modbus",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "Higit pa tungkol sa akin",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Karanasan",
                  href: "/about/experience",
                  meta: "2016 - kasalukuyan",
                  text: "Mga tungkulin mula sa Soundpatrol hanggang sa una kong internship, kasama ang mga proyekto at resulta sa bawat isa.",
                },
                {
                  title: "Mga proyekto",
                  href: "/about/projects",
                  meta: "Rust · open source",
                  text: "Sillok, EU5 Location DB, ang website na ito, at iba pang independiyente at open-source na gawain.",
                },
                {
                  title: "Kaligiran",
                  href: "/about/background",
                  meta: "Edukasyon · pananaliksik",
                  text: "Edukasyon, nailathalang pananaliksik, mga gantimpala, mga wika, at trabahong labas sa software.",
                },
                {
                  title: "Paano gumagana ang site na ito",
                  href: "/about-blog",
                  meta: "Tech stack",
                  text: "Ang host, backend, frontend, mga subsystem, at mga sukat ng pagganap sa likod ng cyhdev.com.",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "Kontak",
          blocks: [
            {
              kind: "list",
              items: [
                "**Email**: [younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**: [github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**: [linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "Ang email ang pinakamaaasahang paraan para makontak ako. Kung gusto mong sumali sa Minecraft server, sabihin din doon.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "Tungkol",
      title: "Karanasan",
      lede: "Gawaing backend, infrastructure, at seguridad sa isang startup sa US, para sa mga Koreanong enterprise na kliyente, at sa isang maagang internship, kasunod ang trabahong nauna sa software.",
      sections: [
        {
          id: "software",
          heading: "Software engineering",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Rust Engineer",
                  period: "Mayo 2026 - kasalukuyan",
                  location: "Estados Unidos · Remote",
                  summary: "Cybersecurity, DevOps, pagbuo ng webservice, at developer tooling para sa isang kumpanya ng music at audio detection.",
                  highlights: [
                    "Pinapanatili ang Kubernetes infrastructure sa Google Cloud, infrastructure as code, at mga nirerepasong deployment workflow na gumagamit ng panandaliang credential na nakasakop sa isang environment.",
                    "Nagpaplano at nagsasagawa ng mga migration ng production data store, na bineberipika ang nailipat na estado at sinasanay ang recovery bago ang bawat cutover.",
                    "Pinagsasama-sama ang mga metric, log, at trace sa OpenTelemetry, at gumagawa ng Rust tooling para sa kalusugan ng serbisyo, diagnostics, at visibility ng gastos.",
                    "Nagtatrabaho sa security readiness: pangangasiwa ng access, rotation at pagsubaybay sa pag-expire ng credential, mga patakaran, incident response, at ebidensya para sa audit.",
                    "Gumagawa ng mga internal na serbisyo at tool sa Rust at Python para sa mga workflow ng operasyon at suporta.",
                  ],
                  tags: [
                    "Rust",
                    "Python",
                    "PostgreSQL",
                    "Kubernetes",
                    "Google Cloud",
                    "OpenTofu",
                    "OpenTelemetry",
                  ],
                },
                {
                  organization: "GenesisNest",
                  title: "Software Engineer",
                  period: "Ene 2025 - Hul 2025",
                  location: "Seongnam, Timog Korea",
                  summary: "Mga backend service at infrastructure tooling para sa mga in-house na platform at sa opisyal na fan application ng isang K-pop group.",
                  highlights: [
                    "Nag-ambag ng mga API, bug fix, at integration sa mga opisyal na backend ng application ng Hyundai, Kia, at Genesis, na nagtatrabaho sa malaking Java codebase at nakikipag-ugnayan sa mga vendor para sa mga serbisyong ginagamit ng sampu-sampung milyong tao sa buong Asya at Europa.",
                    "Gumawa ng mga internationalization pipeline sa Rust at Python na nagta-transform at naglo-load ng isinaling nilalaman sa mga database ng kumpanya, na nakikipag-ugnayan sa mga database administrator na responsable sa mga dataset na nakakonekta sa mga sistema ng pabrika.",
                    "Bumuo ng mga tool sa pamamahala ng infrastructure para sa mga in-house na serbisyo.",
                  ],
                  tags: [
                    "Java",
                    "Spring Boot",
                    "Rust",
                    "Python",
                  ],
                },
                {
                  organization: "pampam Inc.",
                  title: "Contract Software Engineer",
                  period: "Nob 2024 - Dis 2024",
                  location: "Seoul, Timog Korea",
                  highlights: [
                    "Dinisenyo ang PostgreSQL schema at Rust backend para sa isang YouTube analytics platform na may tulong ng AI, at pinamunuan ang deployment nito sa AWS at Google Cloud.",
                    "Gumawa ng mga workflow sa pangongolekta gamit ang YouTube API na sumasaklaw sa sampu-sampung milyong komento at user, na pinagsasama ang data ng channel, mga transcript, at nilalaman ng pahina.",
                    "Isinama ang mga hosted na language model at open-source na model na tumatakbo sa sariling server para bumuo ng mga buod at marketing insight.",
                  ],
                  tags: [
                    "Rust",
                    "PostgreSQL",
                    "AWS",
                    "Google Cloud",
                  ],
                },
                {
                  organization: "EAN Technology Co. Ltd.",
                  title: "Backend Software Engineer (Lead)",
                  period: "Ago 2023 - Ago 2024",
                  location: "Seoul, Timog Korea",
                  summary: "Pinamunuan ang pagbuo ng backend, database, at cloud infrastructure para sa dalawang platform ng gusali at enerhiya.",
                  highlights: [],
                  projects: [
                    {
                      name: "Digital twin ng Samsung C&T Global Engineering Center",
                      highlights: [
                        "Pinalitan ang Express backend at ang kaayusan ng cloud microservice ng isang naka-optimize na Rust/Axum monolith, na nagbaba ng buwanang gastos sa infrastructure mula humigit-kumulang **$5,000 hanggang $150**.",
                        "Sumulat ng humigit-kumulang **30,000 linya**, mga 75% ng backend, kasama ang **80 REST endpoint** na may domain logic ng construction engineering, integrasyon ng data, at mga real-time na operasyon.",
                        "Ibinaba ang maraming request path na may P99 na ilang segundo tungo sa isa o dalawang digit na millisecond sa deployment na may dalawang core at 4 GB, sa pamamagitan ng asynchronous na Rust, multithreading, caching, at pag-tune ng PostgreSQL schema at query.",
                        "Isinama ang ilang libong sensor, meter, at controller ng HVAC at sistema ng tubig sa pamamagitan ng BACnet at Modbus, nang personal sa punong-tanggapan ng Samsung C&T.",
                        "Tumakbo nang siyam na buwan sa production nang walang runtime shutdown, na may humigit-kumulang 90% na saklaw ng paghawak ng error.",
                        "Gumawa ng CI/CD gamit ang mga GitHub runner, AWS CodeDeploy, at Docker; ang mga static na build na naka-tune sa processor ay nagbunga ng portable na container na humigit-kumulang 20 MB.",
                        "Ikinonekta ang inference ng predictive model sa pamamagitan ng ECS, EC2, SQS, SNS, S3, at SageMaker, ipinatupad ang mga kinakailangan sa seguridad ng Samsung SDS, at isinama ang backend sa isang Unreal Engine 4 interface na ginawa ng isang partner na studio.",
                      ],
                    },
                    {
                      name: "Serbisyo sa pamamahala ng enerhiya gamit ang AI",
                      highlights: [
                        "Dinisenyo mula sa simula ang PostgreSQL schema at Rust/Axum API at sumulat ng humigit-kumulang **25,000 linya**, mga 70% ng backend.",
                        "Pinili ang direktang SQL gamit ang tokio-postgres kaysa sa ORM para sa kontrol sa mga query at oras ng compile, na inilapat ang mga natutunan mula sa digital twin.",
                        "Tumulong sa dalawang engineer na galing sa Node.js at Java na maging produktibo sa Rust sa loob ng humigit-kumulang isang buwan, sa isang pangkat na siyam na katao ng mga backend, frontend, at data science engineer.",
                      ],
                    },
                  ],
                  tags: [
                    "Rust",
                    "Axum",
                    "PostgreSQL",
                    "AWS",
                    "Docker",
                    "BACnet",
                    "Modbus",
                  ],
                },
                {
                  organization: "Artifyc Inc.",
                  title: "Software Engineering Intern",
                  period: "Ago 2022 - Mar 2023",
                  location: "Austin, Texas · Remote",
                  highlights: [
                    "Gumawa ng Python Discord bot na gumamit ng Reddit API para maghanap ng mga kahilingan sa art commission at i-post ang mga ito sa channel ng komunidad, na naka-iskedyul sa AWS Lambda tuwing limang minuto.",
                    "Ipinatupad ang Lambda logic para sa pagpapalit ng pera, pagsingil, at pagkalkula ng surcharge sa iba't ibang hurisdiksyon.",
                    "Nag-ambag ng mga rebisyon sa React frontend para sa layout, mga asset, at teksto.",
                  ],
                  tags: [
                    "Python",
                    "AWS Lambda",
                    "React",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "other",
          heading: "Bago ang software",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Korea Institute of Maritime Science and Technology",
                  title: "Tagasalin at interpreter",
                  period: "Hun - Hul 2023, Ago 2024",
                  location: "Seoul, Timog Korea · Kontrata",
                  highlights: [
                    "Nag-interpret sa Ingles, Koreano, at Norwegian sa isang kumperensya sa pananaliksik pandagat na dinaluhan ng ambassador ng Norway sa Korea, isang state secretary ng Norway, pamunuan ng institute, at chairman ng Dongwon Group.",
                    "Isinalin ang mga talumpati at sinanay ang mga kawani ng institute at ang direktor nito para sa isang pandaigdigang kumperensya at para sa isang kaganapan ng U.S. Department of Energy tungkol sa magkasanib na proyekto.",
                  ],
                },
                {
                  organization: "Republic of Korea Army · U.S. Army 2nd Infantry Division",
                  title: "Sarhento, KATUSA liaison at instruktor",
                  period: "Mar 2016 - Dis 2017",
                  location: "Dongducheon, Timog Korea",
                  highlights: [
                    "Naglingkod sa Headquarters Battalion ng dibisyon sa pamamagitan ng programang [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army), sa magkasanib na kapaligirang Koreano at Amerikano.",
                    "Nagturo at nagproseso sa pagdating ng humigit-kumulang 6,000 tauhang Amerikano at Koreano sa mga kurso sa physical fitness, kultura, kasaysayan, at pag-iwas.",
                    "Pinamunuan ang pangkat na 13 katao at pinanatili ang kahandaan sa gitna ng [krisis noong 2017](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) sa tangway.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Tagapagkarga sa logistics",
                  period: "2020 - 2022",
                  location: "Timog Korea · Paminsan-minsang kontrata",
                  highlights: [
                    "Trabaho sa bodega tuwing night shift habang nag-aaral sa unibersidad.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "Tungkol",
      title: "Mga Proyekto",
      lede: "Mga independiyente at open-source na gawa, karamihan ay sa Rust. Nasa [pahina ng mga proyekto](/projects) ang mga interactive na demo.",
      sections: [
        {
          id: "featured",
          heading: "Tampok",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "Isang command-line na talaan ng trabaho. Itinatala bilang append-only na mga event ang mga layunin, natapos na gawain, pagwawasto, at pagbawi, may mga naka-index na kasalukuyang projection, at pinagsasama ng archive na nakabatay sa Git ang mga kasaysayan mula sa iba't ibang makina.",
                  tags: [
                    "Event sourcing",
                    "Sync",
                  ],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Isang desktop at browser na application para sa datos ng mapa ng Europa Universalis V, na may naka-embed na mga compressed na dataset, precomputed na search at sort index, categorical na mga filter, at mga talahanayang maaaring baguhin ang laki. [Subukan dito](/eu5-locations-db).",
                  tags: [
                    "Native + web",
                    "Indexes",
                  ],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Ang site na ito: publikasyon, litrato, forum, live chat na may mga tawag, Minecraft terrain explorer, at mga demo ng WebAssembly, na inihahain ng iisang Rust binary. [Paano ito binuo](/about-blog).",
                  tags: [
                    "Monorepo",
                    "Self-hosted",
                  ],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "Isang generator at formatter ng UUIDv4 na may output sa terminal at file, opsyonal na pagsusuri ng duplicate, at format na tugma sa Python at JSON; humigit-kumulang tatlong beses na mas mabilis kaysa sa uuidgen sa isang maagang lokal na benchmark.",
                  tags: [
                    "Throughput",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "Iba pang gawa",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: isang [upstream na pag-aayos](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) para sa Minecraft server na gawa sa Rust na nagpapanatili ng mga setting ng overworld ng Paper sa pag-import at pag-save; bukas pa para sa review.",
                "**Mga tool para sa mapa ng Minecraft**: isang Paper plugin at mga Rust service sa likod ng [terrain explorer](/minecraft) ng site na ito, kasama ang compact na binary na format ng biome tile at mga hula ng biome batay sa seed.",
                "**Pag-synchronize ng filesystem**: isang Rust utility para sa two-way na pag-synchronize ng mga folder at paglilinis ng build artifact sa pagitan ng magkaibang filesystem, na may mga dry run, operation journal, opsyonal na checksum, at mga pananggalang laban sa malalaking hindi sinasadyang pagbura.",
                "**impulsr**: isang tool sa transkripsyon ng YouTube at pangongolekta ng komento na gumagawa ng mga buod mula sa language model para sa gawaing marketing.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Paano ako bumubuo",
          blocks: [
            {
              kind: "paragraph",
              text: "Mas gusto ko ang maliliit at nasusuri na mga sistema: iisang binary kung sapat na ang iisa, tahasang hangganan sa bawat queue at cache, at mga format ng datos na pinili ayon sa pattern ng pag-access, hindi dahil sa nakagawian.",
            },
            {
              kind: "paragraph",
              text: "Nauuna ang mga sukat bago ang mga pahayag; ipinapakita ng [mga tala sa pagganap](/about-blog/performance) ng site na ito kung paano ito sinusukat.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "Tungkol",
      title: "Kaligiran",
      lede: "Edukasyon, pananaliksik, mga kwalipikasyon, at mga interes sa labas ng software.",
      sections: [
        {
          id: "education",
          heading: "Edukasyon",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Sungkyunkwan University",
                  title: "B.Eng. sa Software Engineering",
                  period: "Mar 2015 - Ago 2023",
                  location: "Suwon, Timog Korea",
                  summary: "Departamento ng Computer Science and Engineering. Kasama sa takdang panahon ng degree ang sapilitang serbisyo militar.",
                  highlights: [
                    "Mga kurso sa embedded software, logic circuit, operating system, network, low-level programming, at electronics, kasabay ng praktika sa software engineering.",
                    "Nagmula ang Sungkyunkwan University noong 1398 bilang royal academy ng dinastiyang Joseon at isa ito sa mga nangungunang unibersidad sa pananaliksik sa Korea.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Pananaliksik",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Korean Language NLP Model Based Emotional Analysis of Social Media Communities.** Ginamit ang KoBERT sa sentiment analysis at pagkakategorya ng social media sa wikang Koreano, upang suriin ang pagbabago ng damdamin sa unang bahagi ng panahon ng COVID-19.",
            },
            {
              kind: "paragraph",
              text: "Nakatanggap ang proyekto ng Bronze Prize sa mga gantimpala ng unibersidad para sa graduation paper at proyekto, sa halos 60 lahok.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Mga gantimpala at kwalipikasyon",
          blocks: [
            {
              kind: "table",
              columns: [
                "Taon",
                "Pagkilala",
                "Detalye",
              ],
              rows: [
                [
                  "2023",
                  "**Bronze Prize**",
                  "Mga gantimpala ng Sungkyunkwan University para sa graduation paper at proyekto",
                ],
                [
                  "2023",
                  "**IELTS 8.5 / 9.0**",
                  "CEFR C2",
                ],
                [
                  "2021",
                  "**Ika-7 puwesto**",
                  "Capstone Design and Idea Hackathon, Korean Society for Internet Information",
                ],
                [
                  "2020",
                  "**Ika-3 puwesto**",
                  "Fintech Hackathon, Seoul National University of Science and Technology",
                ],
                [
                  "2018",
                  "**TOEFL 117 / 120**",
                  "CEFR C2",
                ],
              ],
            },
          ],
        },
        {
          id: "languages",
          heading: "Mga wika",
          blocks: [
            {
              kind: "list",
              items: [
                "**Koreano**: katutubong wika.",
                "**Ingles**: propesyonal na kasanayan (C2).",
                "**Pranses, Aleman, Espanyol, at Mandarin**: batayang kaalaman.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "Sa labas ng software",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pagkuha ng litrato**: litrato ng tanawin at portrait mula pa noong 2010, na may humigit-kumulang 30,000 naka-archive na kuha. May piling koleksyon sa [gallery](/photographs).",
                "**Peryodismo**: estudyanteng reporter ng Sungkyun Times, ang magasing Ingles ng unibersidad, na sumasaklaw sa teknolohiya, seguridad, at usaping pandaigdig.",
                "**Pagtuturo at pagboboluntaryo**: nagturo ng Ingles sa isang community center at nagboluntaryo sa isang thrift store na pinapatakbo ng mga taong may kapansanan.",
                "**Pagsasalin**: interpretasyon at pagsasalin para sa mga pampublikong institusyon; tingnan ang [Karanasan](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
