import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "November 2026",
  chrome: {
    overview: "Overview",
    updated: "Last updated",
    onThisPage: "On this page",
    sectionNavigation: "About pages",
    previous: "Previous",
    next: "Next",
    readingTime: "{minutes} min read",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "About",
      lede: "Backend and infrastructure engineer working in Rust, PostgreSQL, and cloud platforms. I build services that make effective use of the hardware underneath them, along with the deployment, monitoring, and security work that keeps them running.",
      sections: [
        {
          id: "introduction",
          heading: "Introduction",
          blocks: [
            {
              kind: "paragraph",
              text: "I am a Korean software engineer based in Colorado. At **Soundpatrol** I work as a Rust Engineer on cybersecurity, DevOps, webservices, and developer tooling, primarily with Rust, Python, PostgreSQL, Kubernetes, and Google Cloud.",
            },
            {
              kind: "paragraph",
              text: "Before that, I led backend and infrastructure work for a Samsung C&T building digital twin, contributed to the Hyundai, Kia, and Genesis application backends, and built data collection for a YouTube analytics platform. Since 2023 my work has centered on backend services, data pipelines, and the infrastructure under them.",
            },
            {
              kind: "paragraph",
              text: "I believe in building services that make effective use of modern hardware and programming languages. In practice that means careful schema design, bounded queues and caches, explicit error handling, and deployments that can be inspected and recovered. I enjoy helping colleagues pick up Rust, and I want to work with people who take pride in their craft.",
            },
          ],
        },
        {
          id: "results",
          heading: "Selected results",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "Lower monthly cloud spend on the Samsung C&T digital twin, from about $5,000 to $150",
                },
                {
                  value: "80",
                  count: 80,
                  label: "REST endpoints implemented for the digital twin, with construction-engineering domain logic",
                },
                {
                  value: "9 months",
                  count: 9,
                  suffix: " months",
                  label: "Production operation of the digital twin without a runtime shutdown",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "Mixed-read throughput gained on this site's backend from [PGO and BOLT](/about-blog/performance)",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Statically linked service containers from multi-stage builds",
                },
                {
                  value: "30,000",
                  label: "Lines written for the digital twin backend, about 75% of its code",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Toolbox",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Languages",
                  items: ["Rust", "Python", "TypeScript", "SQL", "Java", "C", "C++"],
                },
                {
                  label: "Backend",
                  items: ["Axum", "Tokio", "Diesel", "tokio-postgres", "Spring Boot", "FastAPI", "WebSockets", "gRPC", "Protocol Buffers"],
                },
                {
                  label: "Data",
                  items: ["PostgreSQL", "Valkey / Redis", "SQLite / Turso", "MySQL", "FAISS", "tantivy"],
                },
                {
                  label: "Infrastructure",
                  items: ["Kubernetes", "Helm", "Docker", "OpenTofu / Terraform", "GitHub Actions", "Google Cloud", "AWS", "Linux"],
                },
                {
                  label: "Reliability and security",
                  items: ["OpenTelemetry", "IAM", "TLS", "Network policy", "Credential lifecycle", "Incident response"],
                },
                {
                  label: "Interfaces and devices",
                  items: ["SolidJS", "Leptos", "Slint", "WebAssembly", "WebRTC", "BACnet", "Modbus"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "More about me",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Experience",
                  href: "/about/experience",
                  meta: "2016 - present",
                  text: "Roles from Soundpatrol back to my first internship, with the projects and results behind each.",
                },
                {
                  title: "Projects",
                  href: "/about/projects",
                  meta: "Rust · open source",
                  text: "Sillok, EU5 Location DB, this website, and other independent and open-source work.",
                },
                {
                  title: "Background",
                  href: "/about/background",
                  meta: "Education · research",
                  text: "Education, published research, awards, languages, and work outside software.",
                },
                {
                  title: "How this site works",
                  href: "/about-blog",
                  meta: "Tech stack",
                  text: "The host, backend, frontend, subsystems, and performance measurements behind cyhdev.com.",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "Contact",
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
              text: "Email is the most reliable way to reach me. If you would like to join the Minecraft server, say so there as well.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "About",
      title: "Experience",
      lede: "Backend, infrastructure, and security work at a US startup, for Korean enterprise clients, and in an early internship, followed by the work that came before software.",
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
                  period: "May 2026 - present",
                  location: "United States · Remote",
                  summary: "Cybersecurity, DevOps, webservice development, and developer tooling for a music and audio detection company.",
                  highlights: [
                    "Maintain Kubernetes infrastructure on Google Cloud, infrastructure as code, and reviewed deployment workflows that use short-lived, environment-scoped credentials.",
                    "Plan and carry out production data-store migrations, verifying migrated state and rehearsing recovery before each cutover.",
                    "Consolidate metrics, logs, and traces on OpenTelemetry, and build Rust tooling for service health, diagnostics, and cost visibility.",
                    "Work on security readiness: access administration, credential rotation and expiry tracking, policies, incident response, and audit evidence.",
                    "Build internal Rust and Python services and tools for operations and support workflows.",
                  ],
                  tags: ["Rust", "Python", "PostgreSQL", "Kubernetes", "Google Cloud", "OpenTofu", "OpenTelemetry"],
                },
                {
                  organization: "GenesisNest",
                  title: "Software Engineer",
                  period: "Jan 2025 - Jul 2025",
                  location: "Seongnam, South Korea",
                  summary: "Backend services and infrastructure tooling for in-house platforms and a K-pop group's official fan application.",
                  highlights: [
                    "Contributed APIs, bug fixes, and integrations to the Hyundai, Kia, and Genesis official application backends, working in a large Java codebase and coordinating with vendors for services used by tens of millions of people across Asia and Europe.",
                    "Built internationalization pipelines in Rust and Python that transform and load translated content into corporate databases, coordinating with the database administrators responsible for datasets connected to factory systems.",
                    "Developed infrastructure management tools for in-house services.",
                  ],
                  tags: ["Java", "Spring Boot", "Rust", "Python"],
                },
                {
                  organization: "pampam Inc.",
                  title: "Contract Software Engineer",
                  period: "Nov 2024 - Dec 2024",
                  location: "Seoul, South Korea",
                  highlights: [
                    "Designed the PostgreSQL schema and Rust backend for an AI-assisted YouTube analytics platform and led its deployment across AWS and Google Cloud.",
                    "Built YouTube API collection workflows covering tens of millions of comments and users, combining channel data, transcripts, and page content.",
                    "Integrated hosted language models and on-server open-source models to generate summaries and marketing insights.",
                  ],
                  tags: ["Rust", "PostgreSQL", "AWS", "Google Cloud"],
                },
                {
                  organization: "EAN Technology Co. Ltd.",
                  title: "Backend Software Engineer (Lead)",
                  period: "Aug 2023 - Aug 2024",
                  location: "Seoul, South Korea",
                  summary: "Led backend, database, and cloud infrastructure development for two building and energy platforms.",
                  highlights: [],
                  projects: [
                    {
                      name: "Samsung C&T Global Engineering Center digital twin",
                      highlights: [
                        "Replaced an Express backend and cloud microservice arrangement with an optimized Rust/Axum monolith, cutting monthly infrastructure spend from about **$5,000 to $150**.",
                        "Wrote about **30,000 lines**, roughly 75% of the backend, including **80 REST endpoints** carrying construction-engineering domain logic, data integration, and real-time operations.",
                        "Brought many multi-second P99 request paths down to single- or double-digit milliseconds on a two-core, 4 GB deployment through asynchronous Rust, multithreading, caching, and PostgreSQL schema and query tuning.",
                        "Integrated several thousand sensors, meters, and HVAC and water-system controllers over BACnet and Modbus, on site at Samsung C&T headquarters.",
                        "Operated for nine months in production without a runtime shutdown, with about 90% error-handling coverage.",
                        "Built CI/CD with GitHub runners, AWS CodeDeploy, and Docker; static, processor-tuned builds produced portable containers of about 20 MB.",
                        "Connected predictive-model inference through ECS, EC2, SQS, SNS, S3, and SageMaker, implemented Samsung SDS security requirements, and integrated the backend with an Unreal Engine 4 interface built by a partner studio.",
                      ],
                    },
                    {
                      name: "AI energy management service",
                      highlights: [
                        "Designed the PostgreSQL schema and Rust/Axum API from the ground up and wrote about **25,000 lines**, roughly 70% of the backend.",
                        "Chose direct SQL through tokio-postgres over an ORM for control over queries and compile times, applying lessons from the digital twin.",
                        "Helped two engineers coming from Node.js and Java become productive in Rust within about a month, in a nine-person team of backend, frontend, and data science engineers.",
                      ],
                    },
                  ],
                  tags: ["Rust", "Axum", "PostgreSQL", "AWS", "Docker", "BACnet", "Modbus"],
                },
                {
                  organization: "Artifyc Inc.",
                  title: "Software Engineering Intern",
                  period: "Aug 2022 - Mar 2023",
                  location: "Austin, Texas · Remote",
                  highlights: [
                    "Built a Python Discord bot that used the Reddit API to find art-commission requests and post them to a community channel, scheduled on AWS Lambda every five minutes.",
                    "Implemented Lambda logic for currency conversion, charging, and surcharge calculation across jurisdictions.",
                    "Contributed React frontend revisions to layout, assets, and copy.",
                  ],
                  tags: ["Python", "AWS Lambda", "React"],
                },
              ],
            },
          ],
        },
        {
          id: "other",
          heading: "Before software",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Korea Institute of Maritime Science and Technology",
                  title: "Translator and Interpreter",
                  period: "Jun - Jul 2023, Aug 2024",
                  location: "Seoul, South Korea · Contract",
                  highlights: [
                    "Interpreted in English, Korean, and Norwegian at a maritime research conference attended by Norway's ambassador to Korea, a Norwegian state secretary, institute leadership, and the chairman of Dongwon Group.",
                    "Translated speeches and coached institute staff and its director for an international conference and for a U.S. Department of Energy event on a joint project.",
                  ],
                },
                {
                  organization: "Republic of Korea Army · U.S. Army 2nd Infantry Division",
                  title: "Sergeant, KATUSA liaison and instructor",
                  period: "Mar 2016 - Dec 2017",
                  location: "Dongducheon, South Korea",
                  highlights: [
                    "Served in the division's Headquarters Battalion through the [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army) program, in a joint Korean-American environment.",
                    "Instructed and in-processed about 6,000 American and Korean personnel in fitness, culture, history, and prevention courses.",
                    "Led a 13-person team and maintained readiness through the [2017 crisis](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) on the peninsula.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Logistics loader",
                  period: "2020 - 2022",
                  location: "South Korea · Occasional contracts",
                  highlights: [
                    "Night-shift warehouse work alongside university studies.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "About",
      title: "Projects",
      lede: "Independent and open-source work, mostly in Rust. Interactive demos live on the [projects page](/projects).",
      sections: [
        {
          id: "featured",
          heading: "Featured",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "A command-line work chronicle. Objectives, completed tasks, corrections, and retractions are recorded as append-only events with indexed current projections, and a Git-backed archive merges histories across machines.",
                  tags: ["Event sourcing", "Sync"],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "A desktop and browser application for Europa Universalis V map data, with embedded compressed datasets, precomputed search and sort indexes, categorical filters, and resizable tables. [Try it here](/eu5-locations-db).",
                  tags: ["Native + web", "Indexes"],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "This site: publishing, photography, a forum, live chat with calls, a Minecraft terrain explorer, and WebAssembly demos, served by one Rust binary. [How it is built](/about-blog).",
                  tags: ["Monorepo", "Self-hosted"],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "A UUIDv4 generator and formatter with terminal and file output, optional duplicate checking, and Python- and JSON-compatible formatting; about three times faster than uuidgen in an early local benchmark.",
                  tags: ["Throughput"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "More work",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: an [upstream fix](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) for the Rust Minecraft server that preserves Paper overworld settings on import and save; open for review.",
                "**Minecraft map tooling**: a Paper plugin and Rust services behind this site's [terrain explorer](/minecraft), including a compact binary biome tile format and seed-based biome predictions.",
                "**Filesystem synchronization**: a Rust utility for two-way folder synchronization and build-artifact cleanup across foreign filesystems, with dry runs, operation journals, optional checksums, and guards against large unintended deletions.",
                "**impulsr**: a YouTube transcription and comment-collection tool that produces language-model summaries for marketing work.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "How I build",
          blocks: [
            {
              kind: "paragraph",
              text: "I prefer small, inspectable systems: one binary where one binary will do, an explicit bound on every queue and cache, and data formats chosen for the access pattern rather than out of habit.",
            },
            {
              kind: "paragraph",
              text: "Measurements come before claims; the [performance notes](/about-blog/performance) for this site show how they are taken.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "About",
      title: "Background",
      lede: "Education, research, qualifications, and interests outside software.",
      sections: [
        {
          id: "education",
          heading: "Education",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Sungkyunkwan University",
                  title: "B.Eng. in Software Engineering",
                  period: "Mar 2015 - Aug 2023",
                  location: "Suwon, South Korea",
                  summary: "Department of Computer Science and Engineering. The degree timeline includes mandatory military service.",
                  highlights: [
                    "Coursework across embedded software, logic circuits, operating systems, networks, low-level programming, and electronics, alongside software engineering practice.",
                    "Sungkyunkwan University traces its origins to 1398 as the royal academy of the Joseon dynasty and is one of Korea's leading research universities.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Research",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Korean Language NLP Model Based Emotional Analysis of Social Media Communities.** Applied KoBERT to sentiment analysis and categorization of Korean-language social media, examining emotional change during the early COVID-19 period.",
            },
            {
              kind: "paragraph",
              text: "The project received the Bronze Prize in the university's graduation paper and project awards, among about 60 entries.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Awards and qualifications",
          blocks: [
            {
              kind: "table",
              columns: ["Year", "Recognition", "Detail"],
              rows: [
                ["2023", "**Bronze Prize**", "Sungkyunkwan University graduation paper and project awards"],
                ["2023", "**IELTS 8.5 / 9.0**", "CEFR C2"],
                ["2021", "**7th place**", "Capstone Design and Idea Hackathon, Korean Society for Internet Information"],
                ["2020", "**3rd place**", "Fintech Hackathon, Seoul National University of Science and Technology"],
                ["2018", "**TOEFL 117 / 120**", "CEFR C2"],
              ],
            },
          ],
        },
        {
          id: "languages",
          heading: "Languages",
          blocks: [
            {
              kind: "list",
              items: [
                "**Korean**: native.",
                "**English**: professional fluency (C2).",
                "**French, German, Spanish, and Mandarin**: basic familiarity.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "Outside software",
          blocks: [
            {
              kind: "list",
              items: [
                "**Photography**: landscape and portrait photography since 2010, with about 30,000 archived frames. A selection is in the [gallery](/photographs).",
                "**Journalism**: student reporter for the Sungkyun Times, the university's English-language magazine, covering technology, security, and international affairs.",
                "**Teaching and volunteering**: taught English at a community center and volunteered at a thrift store run by people with disabilities.",
                "**Translation**: interpreting and translation for public institutions; see [Experience](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
