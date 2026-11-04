import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "nov 2026",
  chrome: {
    overview: "Pregled",
    updated: "Posljednje ažuriranje",
    onThisPage: "Na ovoj stranici",
    sectionNavigation: "Stranice o meni",
    previous: "Prethodna",
    next: "Sljedeća",
    readingTime: "Čitanje: {minutes} min",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "O meni",
      lede: "Backend i infrastrukturni inženjer koji radi s Rustom, PostgreSQL-om i platformama u oblaku. Gradim servise koji djelotvorno koriste hardver ispod sebe, kao i isporuku, nadzor i sigurnosni rad koji ih drže u pogonu.",
      sections: [
        {
          id: "introduction",
          heading: "Uvod",
          blocks: [
            {
              kind: "paragraph",
              text: "Ja sam korejski softverski inženjer sa sjedištem u Koloradu. U kompaniji **Soundpatrol** radim kao Rust inženjer na kibernetičkoj sigurnosti, DevOps-u, web servisima i razvojnim alatima, uglavnom s Rustom, Pythonom, PostgreSQL-om, Kubernetesom i Google Cloudom.",
            },
            {
              kind: "paragraph",
              text: "Prije toga sam vodio backend i infrastrukturni rad na digitalnom blizancu zgrade za Samsung C&T, doprinosio backendovima aplikacija Hyundai, Kia i Genesis te izgradio prikupljanje podataka za YouTube analitičku platformu. Od 2023. moj rad je usmjeren na backend servise, cjevovode podataka i infrastrukturu ispod njih.",
            },
            {
              kind: "paragraph",
              text: "Vjerujem u izgradnju servisa koji djelotvorno koriste moderni hardver i programske jezike. U praksi to znači pažljivo oblikovanje šeme baze podataka, ograničene redove čekanja i keševe, eksplicitno rukovanje greškama i isporuke koje se mogu pregledati i oporaviti. Rado pomažem kolegama da savladaju Rust i želim raditi s ljudima koji su ponosni na svoj zanat.",
            },
          ],
        },
        {
          id: "results",
          heading: "Odabrani rezultati",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "Manji mjesečni troškovi oblaka za digitalni blizanac Samsung C&T, s oko 5.000 na 150 dolara",
                },
                {
                  value: "80",
                  count: 80,
                  label: "REST endpointa implementirano za digitalni blizanac, s domenskom logikom građevinskog inženjerstva",
                },
                {
                  value: "9 mjeseci",
                  count: 9,
                  suffix: " mjeseci",
                  label: "Produkcijski rad digitalnog blizanca bez gašenja tokom izvođenja",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "Veća propusnost mješovitog čitanja na backendu ove web stranice zahvaljujući [PGO-u i BOLT-u](/about-blog/performance)",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Statički povezani kontejneri servisa iz višefaznih izgradnji",
                },
                {
                  value: "30.000",
                  label: "Napisanih linija za backend digitalnog blizanca, oko 75% njegovog koda",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Alati",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Jezici",
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
                  label: "Podaci",
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
                  label: "Infrastruktura",
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
                  label: "Pouzdanost i sigurnost",
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
                  label: "Sučelja i uređaji",
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
          heading: "Još o meni",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Iskustvo",
                  href: "/about/experience",
                  meta: "2016. - danas",
                  text: "Uloge od kompanije Soundpatrol unazad do prve prakse, s projektima i rezultatima iza svake.",
                },
                {
                  title: "Projekti",
                  href: "/about/projects",
                  meta: "Rust · otvoreni kod",
                  text: "Sillok, EU5 Location DB, ova web stranica i drugi samostalni projekti i projekti otvorenog koda.",
                },
                {
                  title: "Životopis",
                  href: "/about/background",
                  meta: "Obrazovanje · istraživanje",
                  text: "Obrazovanje, objavljena istraživanja, nagrade, jezici i rad izvan softvera.",
                },
                {
                  title: "Kako radi ova web stranica",
                  href: "/about-blog",
                  meta: "Tehnološki stek",
                  text: "Hosting, backend, frontend, podsistemi i mjerenja performansi iza cyhdev.com.",
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
                "**E-pošta**: [younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**: [github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**: [linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "E-pošta je najpouzdaniji način da me kontaktirate. Ako se želite pridružiti Minecraft serveru, navedite to i tamo.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "O meni",
      title: "Iskustvo",
      lede: "Backend, infrastrukturni i sigurnosni rad u američkom startapu, za korejske korporativne klijente i na ranoj praksi, a zatim rad koji je prethodio softveru.",
      sections: [
        {
          id: "software",
          heading: "Softversko inženjerstvo",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Rust inženjer",
                  period: "maj 2026. - danas",
                  location: "Sjedinjene Američke Države · Na daljinu",
                  summary: "Kibernetička sigurnost, DevOps, razvoj web servisa i razvojni alati za kompaniju za prepoznavanje muzike i zvuka.",
                  highlights: [
                    "Održavam Kubernetes infrastrukturu na Google Cloudu, infrastrukturu kao kod i pregledane tokove isporuke koji koriste kratkotrajne pristupne podatke ograničene na okruženje.",
                    "Planiram i izvodim migracije produkcijskih skladišta podataka, uz provjeru migriranog stanja i uvježbavanje oporavka prije svakog prelaska.",
                    "Objedinjujem metrike, logove i tragove na OpenTelemetryju te pravim Rust alate za zdravlje servisa, dijagnostiku i pregled troškova.",
                    "Radim na sigurnosnoj spremnosti: administracija pristupa, rotacija pristupnih podataka i praćenje isteka, politike, odgovor na incidente i dokazi za reviziju.",
                    "Pravim interne Rust i Python servise i alate za operativne tokove rada i podršku.",
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
                  title: "Softverski inženjer",
                  period: "jan 2025. - jul 2025.",
                  location: "Seongnam, Južna Koreja",
                  summary: "Backend servisi i infrastrukturni alati za interne platforme i službenu aplikaciju za obožavatelje jedne K-pop grupe.",
                  highlights: [
                    "Doprinosio sam API-jima, ispravkama grešaka i integracijama u službenim backendovima aplikacija Hyundai, Kia i Genesis, radeći u velikoj Java bazi koda i usklađujući se s dobavljačima za servise koje koriste desetine miliona ljudi u Aziji i Evropi.",
                    "Izgradio sam internacionalizacijske cjevovode u Rustu i Pythonu koji transformiraju i učitavaju prevedeni sadržaj u korporativne baze podataka, uz usklađivanje s administratorima baza odgovornim za skupove podataka povezane s tvorničkim sistemima.",
                    "Razvio sam alate za upravljanje infrastrukturom za interne servise.",
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
                  title: "Softverski inženjer na ugovor",
                  period: "nov 2024. - dec 2024.",
                  location: "Seul, Južna Koreja",
                  highlights: [
                    "Oblikovao sam PostgreSQL šemu i Rust backend za YouTube analitičku platformu uz podršku AI-ja te vodio njenu isporuku na AWS-u i Google Cloudu.",
                    "Izgradio sam tokove prikupljanja podataka preko YouTube API-ja koji obuhvataju desetine miliona komentara i korisnika, uz kombiniranje podataka o kanalima, transkripata i sadržaja stranica.",
                    "Integrirao sam hostirane jezičke modele i modele otvorenog koda na vlastitom serveru za generiranje sažetaka i marketinških uvida.",
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
                  title: "Backend softverski inženjer (voditelj)",
                  period: "aug 2023. - aug 2024.",
                  location: "Seul, Južna Koreja",
                  summary: "Vodio sam razvoj backenda, baze podataka i infrastrukture u oblaku za dvije platforme za zgrade i energiju.",
                  highlights: [],
                  projects: [
                    {
                      name: "Digitalni blizanac Samsung C&T Global Engineering Center",
                      highlights: [
                        "Zamijenio sam Express backend i raspored mikroservisa u oblaku optimiziranim Rust/Axum monolitom, čime su mjesečni troškovi infrastrukture smanjeni s oko **5.000 na 150 dolara**.",
                        "Napisao sam oko **30.000 linija**, otprilike 75% backenda, uključujući **80 REST endpointa** s domenskom logikom građevinskog inženjerstva, integracijom podataka i operacijama u realnom vremenu.",
                        "Mnoge putanje zahtjeva s P99 od nekoliko sekundi spustio sam na jednocifrene ili dvocifrene milisekunde na isporuci s dva jezgra i 4 GB, uz asinhroni Rust, višenitnost, keširanje te podešavanje PostgreSQL šeme i upita.",
                        "Integrirao sam nekoliko hiljada senzora, brojila te kontrolera grijanja, ventilacije, klimatizacije i vodovodnih sistema preko BACnet-a i Modbusa, na licu mjesta u sjedištu Samsung C&T.",
                        "U produkciji je radio devet mjeseci bez gašenja tokom izvođenja, s oko 90% pokrivenosti rukovanja greškama.",
                        "Izgradio sam CI/CD s GitHub runnerima, AWS CodeDeployem i Dockerom; statičke izgradnje podešene za procesor davale su prenosive kontejnere od oko 20 MB.",
                        "Povezao sam inferenciju prediktivnih modela preko ECS-a, EC2-a, SQS-a, SNS-a, S3-a i SageMakera, implementirao sigurnosne zahtjeve Samsung SDS-a i integrirao backend sa sučeljem za Unreal Engine 4 koje je izradio partnerski studio.",
                      ],
                    },
                    {
                      name: "Servis za upravljanje energijom s AI-jem",
                      highlights: [
                        "Od temelja sam oblikovao PostgreSQL šemu i Rust/Axum API te napisao oko **25.000 linija**, otprilike 70% backenda.",
                        "Odabrao sam direktni SQL preko tokio-postgresa umjesto ORM-a radi kontrole nad upitima i vremenom kompajliranja, primjenjujući iskustva s digitalnog blizanca.",
                        "Pomogao sam dvojici inženjera iz svijeta Node.js-a i Jave da za otprilike mjesec dana postanu produktivni u Rustu, u timu od devet backend, frontend i data science inženjera.",
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
                  title: "Pripravnik softverskog inženjerstva",
                  period: "aug 2022. - mar 2023.",
                  location: "Austin, Teksas · Na daljinu",
                  highlights: [
                    "Izgradio sam Python Discord bota koji je preko Reddit API-ja pronalazio zahtjeve za umjetničke narudžbe i objavljivao ih u kanalu zajednice, zakazan na AWS Lambdi svakih pet minuta.",
                    "Implementirao sam Lambda logiku za konverziju valuta, naplatu i obračun dodatnih naknada u različitim jurisdikcijama.",
                    "Doprinio sam izmjenama React frontenda u rasporedu, resursima i tekstovima.",
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
          heading: "Prije softvera",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Korejski institut za pomorske nauke i tehnologiju",
                  title: "Prevodilac i tumač",
                  period: "jun - jul 2023., aug 2024.",
                  location: "Seul, Južna Koreja · Ugovor",
                  highlights: [
                    "Tumačio sam na engleski, korejski i norveški na pomorskoj istraživačkoj konferenciji kojoj su prisustvovali norveški ambasador u Koreji, norveški državni sekretar, rukovodstvo instituta i predsjednik grupe Dongwon.",
                    "Prevodio sam govore i pripremao osoblje instituta i njegovog direktora za međunarodnu konferenciju i za događaj američkog Ministarstva energetike o zajedničkom projektu.",
                  ],
                },
                {
                  organization: "Vojska Republike Koreje · 2. pješadijska divizija vojske SAD-a",
                  title: "Narednik, KATUSA veza i instruktor",
                  period: "mar 2016. - dec 2017.",
                  location: "Dongducheon, Južna Koreja",
                  highlights: [
                    "Služio sam u štabnom bataljonu divizije kroz program [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army), u zajedničkom korejsko-američkom okruženju.",
                    "Podučavao sam i uvodio u službu oko 6.000 američkih i korejskih pripadnika kroz kurseve o kondiciji, kulturi, historiji i prevenciji.",
                    "Vodio sam tim od 13 ljudi i održavao spremnost tokom [krize 2017.](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) na poluotoku.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Utovarivač u logistici",
                  period: "2020. - 2022.",
                  location: "Južna Koreja · Povremeni ugovori",
                  highlights: [
                    "Noćni rad u skladištu uporedo sa studijem.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "O meni",
      title: "Projekti",
      lede: "Samostalni radovi i radovi otvorenog koda, uglavnom u Rustu. Interaktivni demo primjeri nalaze se na [stranici projekata](/projects).",
      sections: [
        {
          id: "featured",
          heading: "Izdvojeno",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "Kronika rada u komandnoj liniji. Ciljevi, završeni zadaci, ispravke i povlačenja bilježe se kao događaji koji se samo dopisuju, uz indeksirane trenutne projekcije, a arhiva zasnovana na Gitu spaja historije s više računara.",
                  tags: [
                    "Event sourcing",
                    "Sync",
                  ],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Desktop i web aplikacija za podatke s mape igre Europa Universalis V, s ugrađenim komprimiranim skupovima podataka, unaprijed izračunatim indeksima pretrage i sortiranja, kategorijskim filterima i tabelama promjenljive veličine. [Isprobajte je ovdje](/eu5-locations-db).",
                  tags: [
                    "Native + web",
                    "Indexes",
                  ],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Ova web stranica: objavljivanje, fotografija, forum, chat uživo s pozivima, istraživač Minecraft terena i WebAssembly demo primjeri, a sve poslužuje jedan Rust binarni program. [Kako je izgrađen](/about-blog).",
                  tags: [
                    "Monorepo",
                    "Self-hosted",
                  ],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "Generator i formater UUIDv4 identifikatora s izlazom na terminal i u datoteku, opcionalnom provjerom duplikata i formatiranjem kompatibilnim s Pythonom i JSON-om; u ranom lokalnom mjerenju otprilike tri puta brži od alata uuidgen.",
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
          heading: "Ostali radovi",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: [ispravka u izvornom projektu](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) za Minecraft server pisan u Rustu, koja pri uvozu i čuvanju zadržava postavke Paper overworlda; čeka pregled.",
                "**Alati za Minecraft mape**: Paper dodatak i Rust servisi iza [istraživača terena](/minecraft) na ovoj web stranici, uključujući kompaktan binarni format pločica bioma i predviđanja bioma na osnovu seeda.",
                "**Sinkronizacija datotečnih sistema**: Rust alat za dvosmjernu sinkronizaciju direktorija i čišćenje artefakata izgradnje na različitim datotečnim sistemima, s probnim pokretanjem, dnevnicima operacija, opcionalnim kontrolnim sumama i zaštitom od velikih nenamjernih brisanja.",
                "**impulsr**: alat za transkripciju YouTube videa i prikupljanje komentara koji za potrebe marketinga izrađuje sažetke pomoću jezičkih modela.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Kako gradim",
          blocks: [
            {
              kind: "paragraph",
              text: "Volim male sisteme koje je moguće provjeriti: jedan binarni program gdje je jedan dovoljan, izričito ograničenje za svaki red čekanja i keš, te formate podataka odabrane prema obrascu pristupa, a ne iz navike.",
            },
            {
              kind: "paragraph",
              text: "Mjerenja dolaze prije tvrdnji; [bilješke o performansama](/about-blog/performance) ove web stranice pokazuju kako se mjeri.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "O meni",
      title: "Životopis",
      lede: "Obrazovanje, istraživanje, kvalifikacije i interesi izvan softvera.",
      sections: [
        {
          id: "education",
          heading: "Obrazovanje",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Univerzitet Sungkyunkwan",
                  title: "Diplomirani inženjer softverskog inženjerstva (B.Eng.)",
                  period: "mar 2015 - aug 2023",
                  location: "Suwon, Južna Koreja",
                  summary: "Odsjek za računarstvo i inženjerstvo. U trajanje studija ulazi i obavezna vojna služba.",
                  highlights: [
                    "Kursevi iz ugrađenog softvera, logičkih kola, operativnih sistema, mreža, programiranja niskog nivoa i elektronike, uz praksu softverskog inženjerstva.",
                    "Univerzitet Sungkyunkwan vodi porijeklo iz 1398. godine kao kraljevska akademija dinastije Joseon i jedan je od vodećih istraživačkih univerziteta u Koreji.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Istraživanje",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Emocionalna analiza zajednica na društvenim mrežama zasnovana na NLP modelu za korejski jezik.** KoBERT je primijenjen na analizu sentimenta i kategorizaciju sadržaja na korejskom jeziku s društvenih mreža, uz ispitivanje emocionalnih promjena u ranom razdoblju pandemije bolesti COVID-19.",
            },
            {
              kind: "paragraph",
              text: "Projekat je dobio bronzanu nagradu na univerzitetskim nagradama za diplomske radove i projekte, među oko 60 prijava.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Nagrade i kvalifikacije",
          blocks: [
            {
              kind: "table",
              columns: [
                "Godina",
                "Priznanje",
                "Pojedinosti",
              ],
              rows: [
                [
                  "2023",
                  "**Bronzana nagrada**",
                  "Nagrade za diplomske radove i projekte, Univerzitet Sungkyunkwan",
                ],
                [
                  "2023",
                  "**IELTS 8.5 / 9.0**",
                  "CEFR C2",
                ],
                [
                  "2021",
                  "**7. mjesto**",
                  "Hakaton za završne projekte i ideje, Korejsko društvo za internetske informacije",
                ],
                [
                  "2020",
                  "**3. mjesto**",
                  "Fintech hakaton, Seulski nacionalni univerzitet za nauku i tehnologiju",
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
          heading: "Jezici",
          blocks: [
            {
              kind: "list",
              items: [
                "**Korejski**: maternji.",
                "**Engleski**: profesionalno znanje (C2).",
                "**Francuski, njemački, španski i mandarinski**: osnovno poznavanje.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "Izvan softvera",
          blocks: [
            {
              kind: "list",
              items: [
                "**Fotografija**: pejzažna i portretna fotografija od 2010. godine, s oko 30.000 arhiviranih kadrova. Izbor je u [galeriji](/photographs).",
                "**Novinarstvo**: studentski reporter u Sungkyun Timesu, univerzitetskom časopisu na engleskom jeziku, s temama iz tehnologije, sigurnosti i međunarodnih odnosa.",
                "**Podučavanje i volontiranje**: predavao sam engleski u društvenom centru i volontirao u trgovini polovne robe koju vode osobe s invaliditetom.",
                "**Prevođenje**: usmeno i pismeno prevođenje za javne ustanove; pogledajte [Iskustvo](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
