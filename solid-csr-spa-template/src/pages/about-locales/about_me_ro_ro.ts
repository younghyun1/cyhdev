import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "noiembrie 2026",
  chrome: {
    overview: "Prezentare generală",
    updated: "Ultima actualizare",
    onThisPage: "Pe această pagină",
    sectionNavigation: "Pagini Despre",
    previous: "Anterior",
    next: "Următor",
    readingTime: "{minutes} min de citit",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "Despre",
      lede: "Inginer backend și de infrastructură, lucrez cu Rust, PostgreSQL și platforme cloud. Construiesc servicii care folosesc eficient hardware-ul de dedesubt, împreună cu munca de implementare, monitorizare și securitate care le menține în funcțiune.",
      sections: [
        {
          id: "introduction",
          heading: "Introducere",
          blocks: [
            {
              kind: "paragraph",
              text: "Sunt un inginer software coreean, stabilit în Colorado. La **Soundpatrol** lucrez ca inginer Rust la securitate cibernetică, DevOps, servicii web și unelte pentru dezvoltatori, în principal cu Rust, Python, PostgreSQL, Kubernetes și Google Cloud.",
            },
            {
              kind: "paragraph",
              text: "Înainte de aceasta, am condus lucrările de backend și infrastructură pentru un geamăn digital al unei clădiri Samsung C&T, am contribuit la backend-urile aplicațiilor Hyundai, Kia și Genesis și am construit colectarea datelor pentru o platformă de analiză YouTube. Din 2023, munca mea se concentrează pe servicii backend, fluxuri de date și infrastructura de sub acestea.",
            },
            {
              kind: "paragraph",
              text: "Cred în construirea de servicii care folosesc eficient hardware-ul și limbajele de programare moderne. În practică, asta înseamnă scheme de bază de date atent proiectate, cozi și cache-uri limitate, tratarea explicită a erorilor și implementări care pot fi inspectate și recuperate. Îmi place să-mi ajut colegii să învețe Rust și vreau să lucrez cu oameni care sunt mândri de meseria lor.",
            },
          ],
        },
        {
          id: "results",
          heading: "Rezultate alese",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "Reducere a costului lunar de cloud pentru geamănul digital Samsung C&T, de la aproximativ $5.000 la $150",
                },
                {
                  value: "80",
                  count: 80,
                  label: "Endpoint-uri REST implementate pentru geamănul digital, cu logică de domeniu din ingineria construcțiilor",
                },
                {
                  value: "9 luni",
                  count: 9,
                  suffix: " luni",
                  label: "Funcționare în producție a geamănului digital fără nicio oprire a execuției",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "Creștere a debitului la citiri mixte pe backend-ul acestui site datorită [PGO și BOLT](/about-blog/performance)",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Containere de servicii cu legare statică, obținute din compilări în mai multe etape",
                },
                {
                  value: "30.000",
                  label: "Linii scrise pentru backend-ul geamănului digital, aproximativ 75% din cod",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Unelte",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Limbaje",
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
                  label: "Date",
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
                  label: "Infrastructură",
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
                  label: "Fiabilitate și securitate",
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
                  label: "Interfețe și dispozitive",
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
          heading: "Mai multe despre mine",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Experiență",
                  href: "/about/experience",
                  meta: "2016 - prezent",
                  text: "Rolurile de la Soundpatrol până la primul meu stagiu, cu proiectele și rezultatele din spatele fiecăruia.",
                },
                {
                  title: "Proiecte",
                  href: "/about/projects",
                  meta: "Rust · cod deschis",
                  text: "Sillok, EU5 Location DB, acest site și alte lucrări independente și open source.",
                },
                {
                  title: "Parcurs",
                  href: "/about/background",
                  meta: "Studii · cercetare",
                  text: "Studii, cercetări publicate, premii, limbi străine și activități din afara software-ului.",
                },
                {
                  title: "Cum funcționează acest site",
                  href: "/about-blog",
                  meta: "Tehnologii folosite",
                  text: "Găzduirea, backend-ul, frontend-ul, subsistemele și măsurătorile de performanță din spatele cyhdev.com.",
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
                "**E-mail**: [younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**: [github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**: [linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "E-mailul este cea mai sigură cale de a mă contacta. Dacă doriți să vă alăturați serverului de Minecraft, spuneți-mi tot acolo.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "Despre",
      title: "Experiență",
      lede: "Lucrări de backend, infrastructură și securitate la un startup din SUA, pentru clienți corporativi din Coreea și într-un stagiu timpuriu, urmate de munca de dinainte de software.",
      sections: [
        {
          id: "software",
          heading: "Inginerie software",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Inginer Rust",
                  period: "mai 2026 - prezent",
                  location: "Statele Unite · La distanță",
                  summary: "Securitate cibernetică, DevOps, dezvoltare de servicii web și unelte pentru dezvoltatori la o companie de detecție a muzicii și a sunetului.",
                  highlights: [
                    "Mențin infrastructura Kubernetes din Google Cloud, infrastructura ca cod și fluxurile de implementare revizuite, care folosesc credențiale de scurtă durată, limitate la mediu.",
                    "Planific și execut migrări de depozite de date din producție, verificând starea migrată și exersând recuperarea înainte de fiecare comutare.",
                    "Consolidez metricile, jurnalele și traseele pe OpenTelemetry și construiesc unelte în Rust pentru starea serviciilor, diagnosticare și vizibilitatea costurilor.",
                    "Lucrez la pregătirea pentru securitate: administrarea accesului, rotirea și urmărirea expirării credențialelor, politici, răspuns la incidente și dovezi pentru audit.",
                    "Construiesc servicii și unelte interne în Rust și Python pentru fluxurile de operațiuni și suport.",
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
                  title: "Inginer software",
                  period: "ian. 2025 - iul. 2025",
                  location: "Seongnam, Coreea de Sud",
                  summary: "Servicii backend și unelte de infrastructură pentru platforme interne și pentru aplicația oficială a fanilor unui grup K-pop.",
                  highlights: [
                    "Am contribuit cu API-uri, remedieri de erori și integrări la backend-urile aplicațiilor oficiale Hyundai, Kia și Genesis, lucrând într-o bază de cod Java mare și coordonându-mă cu furnizorii pentru servicii folosite de zeci de milioane de oameni din Asia și Europa.",
                    "Am construit fluxuri de internaționalizare în Rust și Python care transformă și încarcă conținut tradus în bazele de date ale companiei, în coordonare cu administratorii de baze de date responsabili de seturile de date conectate la sistemele din fabrici.",
                    "Am dezvoltat unelte de administrare a infrastructurii pentru serviciile interne.",
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
                  title: "Inginer software contractual",
                  period: "nov. 2024 - dec. 2024",
                  location: "Seul, Coreea de Sud",
                  highlights: [
                    "Am proiectat schema PostgreSQL și backend-ul Rust pentru o platformă de analiză YouTube asistată de IA și am condus implementarea ei în AWS și Google Cloud.",
                    "Am construit fluxuri de colectare prin API-ul YouTube pentru zeci de milioane de comentarii și utilizatori, combinând date despre canale, transcrieri și conținutul paginilor.",
                    "Am integrat modele de limbaj găzduite și modele open source rulate pe server pentru a genera rezumate și informații de marketing.",
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
                  title: "Inginer software backend (lider)",
                  period: "aug. 2023 - aug. 2024",
                  location: "Seul, Coreea de Sud",
                  summary: "Am condus dezvoltarea backend-ului, a bazei de date și a infrastructurii cloud pentru două platforme de clădiri și energie.",
                  highlights: [],
                  projects: [
                    {
                      name: "Geamănul digital al Samsung C&T Global Engineering Center",
                      highlights: [
                        "Am înlocuit un backend Express și un aranjament de microservicii în cloud cu un monolit Rust/Axum optimizat, reducând costul lunar de infrastructură de la aproximativ **$5.000 la $150**.",
                        "Am scris aproximativ **30.000 de linii**, circa 75% din backend, inclusiv **80 de endpoint-uri REST** cu logică de domeniu din ingineria construcțiilor, integrare de date și operațiuni în timp real.",
                        "Am redus multe căi de cerere cu P99 de mai multe secunde la milisecunde cu una sau două cifre, pe o implementare cu două nuclee și 4 GB, prin Rust asincron, multithreading, caching și optimizarea schemei și a interogărilor PostgreSQL.",
                        "Am integrat câteva mii de senzori, contoare și controlere de HVAC și de sisteme de apă prin BACnet și Modbus, la fața locului, la sediul Samsung C&T.",
                        "A funcționat nouă luni în producție fără nicio oprire a execuției, cu o acoperire de aproximativ 90% a tratării erorilor.",
                        "Am construit CI/CD cu runner-e GitHub, AWS CodeDeploy și Docker; compilările statice, adaptate procesorului, au produs containere portabile de aproximativ 20 MB.",
                        "Am conectat inferența modelelor predictive prin ECS, EC2, SQS, SNS, S3 și SageMaker, am implementat cerințele de securitate Samsung SDS și am integrat backend-ul cu o interfață Unreal Engine 4 realizată de un studio partener.",
                      ],
                    },
                    {
                      name: "Serviciu de management energetic cu IA",
                      highlights: [
                        "Am proiectat de la zero schema PostgreSQL și API-ul Rust/Axum și am scris aproximativ **25.000 de linii**, circa 70% din backend.",
                        "Am ales SQL direct prin tokio-postgres în locul unui ORM, pentru control asupra interogărilor și a timpilor de compilare, aplicând lecțiile din geamănul digital.",
                        "Am ajutat doi ingineri veniți din Node.js și Java să devină productivi în Rust în aproximativ o lună, într-o echipă de nouă persoane formată din ingineri backend, frontend și specialiști în știința datelor.",
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
                  title: "Stagiar în inginerie software",
                  period: "aug. 2022 - mar. 2023",
                  location: "Austin, Texas · La distanță",
                  highlights: [
                    "Am construit un bot Discord în Python care folosea API-ul Reddit pentru a găsi cereri de comenzi de artă și le posta pe un canal al comunității, programat pe AWS Lambda la fiecare cinci minute.",
                    "Am implementat logica Lambda pentru conversie valutară, taxare și calculul suprataxelor în mai multe jurisdicții.",
                    "Am contribuit cu modificări ale frontend-ului React privind aspectul, resursele și textele.",
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
          heading: "Înainte de software",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Institutul Coreean de Știință și Tehnologie Maritimă",
                  title: "Traducător și interpret",
                  period: "iun. - iul. 2023, aug. 2024",
                  location: "Seul, Coreea de Sud · Contract",
                  highlights: [
                    "Am interpretat în engleză, coreeană și norvegiană la o conferință de cercetare maritimă la care au participat ambasadorul Norvegiei în Coreea, un secretar de stat norvegian, conducerea institutului și președintele Dongwon Group.",
                    "Am tradus discursuri și am îndrumat personalul institutului și directorul acestuia pentru o conferință internațională și pentru un eveniment al Departamentului Energiei al SUA dedicat unui proiect comun.",
                  ],
                },
                {
                  organization: "Armata Republicii Coreea · Divizia 2 Infanterie a Armatei SUA",
                  title: "Sergent, ofițer de legătură și instructor KATUSA",
                  period: "mar. 2016 - dec. 2017",
                  location: "Dongducheon, Coreea de Sud",
                  highlights: [
                    "Am servit în Batalionul de Comandă al diviziei prin programul [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army), într-un mediu comun coreeano-american.",
                    "Am instruit și am primit la înrolare aproximativ 6.000 de militari americani și coreeni la cursuri de pregătire fizică, cultură, istorie și prevenție.",
                    "Am condus o echipă de 13 persoane și am menținut starea de pregătire pe durata [crizei din 2017](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) de pe peninsulă.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Încărcător logistic",
                  period: "2020 - 2022",
                  location: "Coreea de Sud · Contracte ocazionale",
                  highlights: [
                    "Muncă în depozit în tura de noapte, alături de studiile universitare.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "Despre",
      title: "Proiecte",
      lede: "Lucrări independente și open source, în mare parte în Rust. Demonstrațiile interactive se află pe [pagina de proiecte](/projects).",
      sections: [
        {
          id: "featured",
          heading: "Proiecte principale",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "O cronică de lucru în linia de comandă. Obiectivele, sarcinile finalizate, corecțiile și retragerile sunt înregistrate ca evenimente doar de adăugare, cu proiecții curente indexate, iar o arhivă bazată pe Git îmbină istoricele de pe mai multe mașini.",
                  tags: [
                    "Event sourcing",
                    "Sync",
                  ],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "O aplicație desktop și de browser pentru datele hărții din Europa Universalis V, cu seturi de date comprimate încorporate, indexuri de căutare și sortare precalculate, filtre pe categorii și tabele redimensionabile. [Încercați-o aici](/eu5-locations-db).",
                  tags: [
                    "Native + web",
                    "Indexes",
                  ],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Acest site: publicare, fotografie, un forum, chat în timp real cu apeluri, un explorator de teren Minecraft și demonstrații WebAssembly, servite de un singur binar Rust. [Cum este construit](/about-blog).",
                  tags: [
                    "Monorepo",
                    "Self-hosted",
                  ],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "Un generator și formatator de UUIDv4 cu ieșire în terminal și în fișier, verificare opțională a duplicatelor și formatare compatibilă cu Python și JSON; de aproximativ trei ori mai rapid decât uuidgen într-un benchmark local inițial.",
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
          heading: "Alte lucrări",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: o [corecție în amonte](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) pentru serverul Minecraft scris în Rust, care păstrează setările lumii Overworld din Paper la import și la salvare; deschisă pentru revizuire.",
                "**Instrumente pentru hărți Minecraft**: un plugin Paper și servicii Rust din spatele [exploratorului de teren](/minecraft) al acestui site, inclusiv un format binar compact pentru plăcile de biomuri și predicții de biomuri pe baza seed-ului.",
                "**Sincronizarea sistemelor de fișiere**: un utilitar Rust pentru sincronizarea bidirecțională a folderelor și curățarea artefactelor de compilare între sisteme de fișiere diferite, cu rulări de probă, jurnale de operații, sume de control opționale și protecții împotriva ștergerilor mari neintenționate.",
                "**impulsr**: un instrument de transcriere YouTube și de colectare a comentariilor, care produce rezumate generate de modele de limbaj pentru activități de marketing.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Cum construiesc",
          blocks: [
            {
              kind: "paragraph",
              text: "Prefer sisteme mici, ușor de inspectat: un singur binar acolo unde ajunge un singur binar, o limită explicită pentru fiecare coadă și cache, și formate de date alese pentru modul de acces, nu din obișnuință.",
            },
            {
              kind: "paragraph",
              text: "Măsurătorile vin înaintea afirmațiilor; [notele despre performanță](/about-blog/performance) ale acestui site arată cum sunt făcute.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "Despre",
      title: "Parcurs",
      lede: "Studii, cercetare, calificări și interese din afara software-ului.",
      sections: [
        {
          id: "education",
          heading: "Studii",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Universitatea Sungkyunkwan",
                  title: "Licență în inginerie software",
                  period: "mar. 2015 - aug. 2023",
                  location: "Suwon, Coreea de Sud",
                  summary: "Departamentul de Informatică și Inginerie. Durata studiilor include serviciul militar obligatoriu.",
                  highlights: [
                    "Cursuri de software încorporat, circuite logice, sisteme de operare, rețele, programare de nivel scăzut și electronică, alături de practica ingineriei software.",
                    "Universitatea Sungkyunkwan își are originea în 1398, ca academie regală a dinastiei Joseon, și este una dintre principalele universități de cercetare din Coreea.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Cercetare",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Analiza emoțională a comunităților din rețelele sociale pe baza unui model NLP pentru limba coreeană.** Am aplicat KoBERT la analiza sentimentelor și la clasificarea conținutului în limba coreeană din rețelele sociale, urmărind schimbarea emoțională din primele luni ale pandemiei de COVID-19.",
            },
            {
              kind: "paragraph",
              text: "Proiectul a primit Premiul de Bronz la concursul universității pentru lucrări de licență și proiecte, dintre aproximativ 60 de participări.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Premii și calificări",
          blocks: [
            {
              kind: "table",
              columns: [
                "An",
                "Recunoaștere",
                "Detalii",
              ],
              rows: [
                [
                  "2023",
                  "**Premiul de Bronz**",
                  "Premiile pentru lucrări de licență și proiecte ale Universității Sungkyunkwan",
                ],
                [
                  "2023",
                  "**IELTS 8.5 / 9.0**",
                  "CEFR C2",
                ],
                [
                  "2021",
                  "**Locul 7**",
                  "Hackathon de proiecte de diplomă și idei, Societatea Coreeană pentru Informații pe Internet",
                ],
                [
                  "2020",
                  "**Locul 3**",
                  "Hackathon Fintech, Universitatea Națională de Știință și Tehnologie din Seul",
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
          heading: "Limbi",
          blocks: [
            {
              kind: "list",
              items: [
                "**Coreeană**: limbă maternă.",
                "**Engleză**: fluență profesională (C2).",
                "**Franceză, germană, spaniolă și mandarină**: cunoștințe de bază.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "În afara software-ului",
          blocks: [
            {
              kind: "list",
              items: [
                "**Fotografie**: fotografie de peisaj și de portret începând din 2010, cu aproximativ 30.000 de cadre arhivate. O selecție se află în [galerie](/photographs).",
                "**Jurnalism**: reporter student la Sungkyun Times, revista în limba engleză a universității, pe teme de tehnologie, securitate și afaceri internaționale.",
                "**Predare și voluntariat**: am predat engleza la un centru comunitar și am fost voluntar la un magazin second-hand administrat de persoane cu dizabilități.",
                "**Traducere**: interpretare și traducere pentru instituții publice; consultați [Experiență](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
