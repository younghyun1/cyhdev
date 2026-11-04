import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "november 2026",
  chrome: {
    overview: "Översikt",
    updated: "Senast uppdaterad",
    onThisPage: "På den här sidan",
    sectionNavigation: "Om-sidor",
    previous: "Föregående",
    next: "Nästa",
    readingTime: "{minutes} min läsning",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "Om",
      lede: "Backend- och infrastrukturingenjör som arbetar med Rust, PostgreSQL och molnplattformar. Jag bygger tjänster som utnyttjar hårdvaran under dem effektivt, tillsammans med det arbete med driftsättning, övervakning och säkerhet som håller dem igång.",
      sections: [
        {
          id: "introduction",
          heading: "Introduktion",
          blocks: [
            {
              kind: "paragraph",
              text: "Jag är en koreansk mjukvaruingenjör bosatt i Colorado. På **Soundpatrol** arbetar jag som Rust-ingenjör med cybersäkerhet, DevOps, webbtjänster och utvecklarverktyg, främst med Rust, Python, PostgreSQL, Kubernetes och Google Cloud.",
            },
            {
              kind: "paragraph",
              text: "Dessförinnan ledde jag backend- och infrastrukturarbetet för en digital tvilling av en byggnad åt Samsung C&T, bidrog till backend-systemen för Hyundais, Kias och Genesis applikationer och byggde datainsamling för en analysplattform för YouTube. Sedan 2023 har mitt arbete kretsat kring backend-tjänster, datapipelines och infrastrukturen under dem.",
            },
            {
              kind: "paragraph",
              text: "Jag tror på att bygga tjänster som utnyttjar modern hårdvara och moderna programmeringsspråk effektivt. I praktiken betyder det noggrann schemadesign, begränsade köer och cacher, explicit felhantering och driftsättningar som går att inspektera och återställa. Jag gillar att hjälpa kollegor att komma in i Rust, och jag vill arbeta med människor som är stolta över sitt hantverk.",
            },
          ],
        },
        {
          id: "results",
          heading: "Utvalda resultat",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "Lägre månatlig molnkostnad för den digitala tvillingen åt Samsung C&T, från cirka 5 000 USD till 150 USD",
                },
                {
                  value: "80",
                  count: 80,
                  label: "REST-endpoints implementerade för den digitala tvillingen, med domänlogik för byggnadsteknik",
                },
                {
                  value: "9 månader",
                  count: 9,
                  suffix: " månader",
                  label: "Produktionsdrift av den digitala tvillingen utan någon nedstängning under körning",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "Ökad genomströmning för blandade läsningar i den här webbplatsens backend tack vare [PGO och BOLT](/about-blog/performance)",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Statiskt länkade tjänstecontainrar från flerstegsbyggen",
                },
                {
                  value: "30 000",
                  label: "Rader skrivna för backend till den digitala tvillingen, cirka 75 % av koden",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Verktygslåda",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Språk",
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
                  label: "Infrastruktur",
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
                  label: "Tillförlitlighet och säkerhet",
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
                  label: "Gränssnitt och enheter",
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
          heading: "Mer om mig",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Erfarenhet",
                  href: "/about/experience",
                  meta: "2016 - nu",
                  text: "Roller från Soundpatrol tillbaka till min första praktik, med projekten och resultaten bakom var och en.",
                },
                {
                  title: "Projekt",
                  href: "/about/projects",
                  meta: "Rust · öppen källkod",
                  text: "Sillok, EU5 Location DB, den här webbplatsen och annat eget arbete och arbete med öppen källkod.",
                },
                {
                  title: "Bakgrund",
                  href: "/about/background",
                  meta: "Utbildning · forskning",
                  text: "Utbildning, publicerad forskning, utmärkelser, språk och arbete utanför mjukvara.",
                },
                {
                  title: "Så fungerar den här webbplatsen",
                  href: "/about-blog",
                  meta: "Teknikstack",
                  text: "Värden, backend, frontend, delsystemen och prestandamätningarna bakom cyhdev.com.",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "Kontakt",
          blocks: [
            {
              kind: "list",
              items: [
                "**E-post**: [younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**: [github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**: [linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "E-post är det säkraste sättet att nå mig. Om du vill gå med i Minecraft-servern, skriv det där också.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "Om",
      title: "Erfarenhet",
      lede: "Backend-, infrastruktur- och säkerhetsarbete på ett amerikanskt startup-bolag, åt koreanska företagskunder och under en tidig praktik, följt av arbetet som kom före mjukvaran.",
      sections: [
        {
          id: "software",
          heading: "Mjukvaruutveckling",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Rust-ingenjör",
                  period: "maj 2026 - nu",
                  location: "USA · På distans",
                  summary: "Cybersäkerhet, DevOps, utveckling av webbtjänster och utvecklarverktyg för ett företag inom musik- och ljuddetektering.",
                  highlights: [
                    "Underhåller Kubernetes-infrastruktur på Google Cloud, infrastruktur som kod och granskade driftsättningsflöden som använder kortlivade, miljöavgränsade autentiseringsuppgifter.",
                    "Planerar och genomför migreringar av datalager i produktion, med verifiering av det migrerade tillståndet och övning av återställning före varje övergång.",
                    "Samlar mätvärden, loggar och spår på OpenTelemetry och bygger Rust-verktyg för tjänsters hälsa, diagnostik och kostnadsinsyn.",
                    "Arbetar med säkerhetsberedskap: åtkomstadministration, rotation och utgångsbevakning av autentiseringsuppgifter, policyer, incidenthantering och revisionsunderlag.",
                    "Bygger interna tjänster och verktyg i Rust och Python för drift- och supportflöden.",
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
                  title: "Mjukvaruingenjör",
                  period: "jan 2025 - jul 2025",
                  location: "Seongnam, Sydkorea",
                  summary: "Backend-tjänster och infrastrukturverktyg för interna plattformar och den officiella fanapplikationen för en K-pop-grupp.",
                  highlights: [
                    "Bidrog med API:er, buggfixar och integrationer till de officiella applikationsbackenderna för Hyundai, Kia och Genesis, i en stor Java-kodbas och i samordning med leverantörer, för tjänster som används av tiotals miljoner människor i Asien och Europa.",
                    "Byggde internationaliseringspipelines i Rust och Python som omvandlar och läser in översatt innehåll i företagsdatabaser, i samordning med de databasadministratörer som ansvarar för dataset kopplade till fabrikssystem.",
                    "Utvecklade verktyg för infrastrukturhantering av interna tjänster.",
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
                  title: "Mjukvaruingenjör (konsult)",
                  period: "nov 2024 - dec 2024",
                  location: "Seoul, Sydkorea",
                  highlights: [
                    "Utformade PostgreSQL-schemat och Rust-backenden för en AI-assisterad analysplattform för YouTube och ledde driftsättningen över AWS och Google Cloud.",
                    "Byggde insamlingsflöden mot YouTube API för tiotals miljoner kommentarer och användare, som kombinerar kanaldata, transkript och sidinnehåll.",
                    "Integrerade externt tillhandahållna språkmodeller och modeller med öppen källkod på egen server för att generera sammanfattningar och marknadsföringsinsikter.",
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
                  title: "Backend-ingenjör (ledare)",
                  period: "aug 2023 - aug 2024",
                  location: "Seoul, Sydkorea",
                  summary: "Ledde utvecklingen av backend, databas och molninfrastruktur för två plattformar för byggnader och energi.",
                  highlights: [],
                  projects: [
                    {
                      name: "Digital tvilling för Samsung C&T Global Engineering Center",
                      highlights: [
                        "Ersatte en Express-backend och en uppsättning molnmikrotjänster med en optimerad Rust/Axum-monolit och sänkte de månatliga infrastrukturkostnaderna från cirka **5 000 till 150 USD**.",
                        "Skrev cirka **30 000 rader**, ungefär 75 % av backenden, inklusive **80 REST-endpoints** med domänlogik för byggnadsteknik, dataintegration och realtidsdrift.",
                        "Förde ner många förfrågningsvägar med P99 på flera sekunder till ensiffriga eller tvåsiffriga millisekunder i en driftsättning med två kärnor och 4 GB, med asynkron Rust, multitrådning, cachning samt justering av PostgreSQL-schema och frågor.",
                        "Integrerade flera tusen sensorer, mätare och styrenheter för ventilation, värme och vattensystem över BACnet och Modbus, på plats vid Samsung C&T:s huvudkontor.",
                        "Drevs i nio månader i produktion utan nedstängning under körning, med cirka 90 % täckning av felhantering.",
                        "Byggde CI/CD med GitHub-runners, AWS CodeDeploy och Docker; statiska, processoranpassade bygg gav portabla containrar på cirka 20 MB.",
                        "Kopplade inferens för prediktiva modeller via ECS, EC2, SQS, SNS, S3 och SageMaker, implementerade säkerhetskrav från Samsung SDS och integrerade backenden med ett Unreal Engine 4-gränssnitt byggt av en partnerstudio.",
                      ],
                    },
                    {
                      name: "AI-baserad energihanteringstjänst",
                      highlights: [
                        "Utformade PostgreSQL-schemat och Rust/Axum-API:et från grunden och skrev cirka **25 000 rader**, ungefär 70 % av backenden.",
                        "Valde direkt SQL via tokio-postgres framför en ORM för kontroll över frågor och kompileringstider, med lärdomar från den digitala tvillingen.",
                        "Hjälpte två ingenjörer med bakgrund i Node.js och Java att bli produktiva i Rust på ungefär en månad, i ett team på nio personer med backend-, frontend- och datavetenskapsingenjörer.",
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
                  title: "Mjukvaruingenjör (praktikant)",
                  period: "aug 2022 - mar 2023",
                  location: "Austin, Texas · På distans",
                  highlights: [
                    "Byggde en Discord-bot i Python som använde Reddit API för att hitta förfrågningar om beställningsverk inom konst och posta dem i en communitykanal, schemalagd på AWS Lambda var femte minut.",
                    "Implementerade Lambda-logik för valutaomräkning, debitering och beräkning av tillägg mellan jurisdiktioner.",
                    "Bidrog med ändringar i React-frontenden av layout, resurser och texter.",
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
          heading: "Före mjukvaran",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Korea Institute of Maritime Science and Technology",
                  title: "Översättare och tolk",
                  period: "jun - jul 2023, aug 2024",
                  location: "Seoul, Sydkorea · Uppdrag",
                  highlights: [
                    "Tolkade mellan engelska, koreanska och norska vid en maritim forskningskonferens med Norges ambassadör i Korea, en norsk statssekreterare, institutets ledning och ordföranden för Dongwon Group som deltagare.",
                    "Översatte tal och coachade institutets personal och dess direktör inför en internationell konferens och ett evenemang med det amerikanska energidepartementet om ett gemensamt projekt.",
                  ],
                },
                {
                  organization: "Sydkoreas armé · USA:s armé, 2nd Infantry Division",
                  title: "Sergeant, KATUSA-förbindelseman och instruktör",
                  period: "mar 2016 - dec 2017",
                  location: "Dongducheon, Sydkorea",
                  highlights: [
                    "Tjänstgjorde i divisionens stabsbataljon genom programmet [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army), i en gemensam koreansk-amerikansk miljö.",
                    "Utbildade och introducerade cirka 6 000 amerikansk och koreansk personal i kurser om fysisk träning, kultur, historia och prevention.",
                    "Ledde ett team på 13 personer och upprätthöll beredskapen under [krisen 2017](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) på halvön.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Logistiklastare",
                  period: "2020 - 2022",
                  location: "Sydkorea · Enstaka uppdrag",
                  highlights: [
                    "Nattskiftsarbete på lager vid sidan av universitetsstudierna.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "Om",
      title: "Projekt",
      lede: "Fristående arbete och öppen källkod, mest i Rust. Interaktiva demos finns på [projektsidan](/projects).",
      sections: [
        {
          id: "featured",
          heading: "I urval",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "En kommandoradsbaserad arbetskrönika. Mål, slutförda uppgifter, rättelser och återkallelser sparas som händelser som bara läggs till, med indexerade aktuella vyer, och ett Git-baserat arkiv slår ihop historik mellan datorer.",
                  tags: [
                    "Event sourcing",
                    "Sync",
                  ],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Ett skrivbords- och webbläsarprogram för kartdata från Europa Universalis V, med inbäddade komprimerade datamängder, förberäknade sök- och sorteringsindex, kategorifilter och tabeller med justerbar storlek. [Prova här](/eu5-locations-db).",
                  tags: [
                    "Native + web",
                    "Indexes",
                  ],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Den här webbplatsen: publicering, fotografi, ett forum, livechatt med samtal, en terrängutforskare för Minecraft och WebAssembly-demos, serverade av en enda Rust-binär. [Så är den byggd](/about-blog).",
                  tags: [
                    "Monorepo",
                    "Self-hosted",
                  ],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "En generator och formaterare för UUIDv4 med utdata till terminal och fil, valfri kontroll av dubbletter och formatering som är kompatibel med Python och JSON; ungefär tre gånger snabbare än uuidgen i ett tidigt lokalt benchmark.",
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
          heading: "Mer arbete",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: en [rättning i uppströmsprojektet](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) för Minecraft-servern skriven i Rust, som bevarar Papers inställningar för övervärlden vid import och sparning; öppen för granskning.",
                "**Verktyg för Minecraft-kartor**: ett Paper-insticksprogram och Rust-tjänster bakom webbplatsens [terrängutforskare](/minecraft), med ett kompakt binärt format för biomrutor och biomprognoser baserade på seed.",
                "**Filsystemssynkronisering**: ett Rust-verktyg för tvåvägssynkronisering av mappar och rensning av byggartefakter mellan olika filsystem, med torrkörningar, operationsjournaler, valfria kontrollsummor och skydd mot stora oavsiktliga raderingar.",
                "**impulsr**: ett verktyg för att transkribera YouTube och samla in kommentarer, som tar fram sammanfattningar med språkmodeller för marknadsföringsarbete.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Så bygger jag",
          blocks: [
            {
              kind: "paragraph",
              text: "Jag föredrar små system som går att granska: en binär där en binär räcker, en uttrycklig gräns för varje kö och cache, och dataformat som väljs efter åtkomstmönstret och inte av gammal vana.",
            },
            {
              kind: "paragraph",
              text: "Mätningar kommer före påståenden; [prestandaanteckningarna](/about-blog/performance) för den här webbplatsen visar hur de görs.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "Om",
      title: "Bakgrund",
      lede: "Utbildning, forskning, meriter och intressen utanför mjukvara.",
      sections: [
        {
          id: "education",
          heading: "Utbildning",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Sungkyunkwan University",
                  title: "Högskoleingenjörsexamen i programvaruteknik",
                  period: "mar 2015 - aug 2023",
                  location: "Suwon, Sydkorea",
                  summary: "Institutionen för datavetenskap och datateknik. Studietiden omfattar obligatorisk militärtjänstgöring.",
                  highlights: [
                    "Kurser i inbyggd programvara, logiska kretsar, operativsystem, nätverk, lågnivåprogrammering och elektronik, vid sidan av programvaruutveckling i praktiken.",
                    "Sungkyunkwan University har sitt ursprung i år 1398 som Joseondynastins kungliga akademi och är ett av Koreas ledande forskningsuniversitet.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Forskning",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Korean Language NLP Model Based Emotional Analysis of Social Media Communities.** Tillämpade KoBERT på sentimentanalys och kategorisering av koreanskspråkiga sociala medier och undersökte känslomässiga förändringar under den tidiga covid-19-perioden.",
            },
            {
              kind: "paragraph",
              text: "Projektet fick bronspriset i universitetets pris för examensarbeten och projekt, bland ungefär 60 bidrag.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Utmärkelser och meriter",
          blocks: [
            {
              kind: "table",
              columns: [
                "År",
                "Utmärkelse",
                "Detaljer",
              ],
              rows: [
                [
                  "2023",
                  "**Bronspris**",
                  "Sungkyunkwan Universitys pris för examensarbeten och projekt",
                ],
                [
                  "2023",
                  "**IELTS 8.5 / 9.0**",
                  "CEFR C2",
                ],
                [
                  "2021",
                  "**7:e plats**",
                  "Capstone Design and Idea Hackathon, Korean Society for Internet Information",
                ],
                [
                  "2020",
                  "**3:e plats**",
                  "Fintech-hackathon, Seoul National University of Science and Technology",
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
          heading: "Språk",
          blocks: [
            {
              kind: "list",
              items: [
                "**Koreanska**: modersmål.",
                "**Engelska**: flytande på professionell nivå (C2).",
                "**Franska, tyska, spanska och mandarin**: grundläggande kännedom.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "Utanför mjukvaran",
          blocks: [
            {
              kind: "list",
              items: [
                "**Fotografi**: landskaps- och porträttfotografi sedan 2010, med omkring 30 000 arkiverade bilder. Ett urval finns i [galleriet](/photographs).",
                "**Journalistik**: studentreporter på Sungkyun Times, universitetets engelskspråkiga tidskrift, med teknik, säkerhet och internationella frågor som ämnen.",
                "**Undervisning och volontärarbete**: undervisade i engelska på ett medborgarhus och var volontär i en secondhandbutik som drivs av personer med funktionsnedsättning.",
                "**Översättning**: tolkning och översättning för offentliga institutioner; se [Erfarenhet](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
