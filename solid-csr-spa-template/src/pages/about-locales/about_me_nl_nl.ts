import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "november 2026",
  chrome: {
    overview: "Overzicht",
    updated: "Laatst bijgewerkt",
    onThisPage: "Op deze pagina",
    sectionNavigation: "Pagina’s over mij",
    previous: "Vorige",
    next: "Volgende",
    readingTime: "{minutes} min leestijd",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "Over mij",
      lede: "Backend- en infrastructuurontwikkelaar met Rust, PostgreSQL en cloudplatforms als werkterrein. Ik bouw diensten die de onderliggende hardware effectief benutten, plus het deployment-, monitoring- en beveiligingswerk dat ze draaiende houdt.",
      sections: [
        {
          id: "introduction",
          heading: "Introductie",
          blocks: [
            {
              kind: "paragraph",
              text: "Ik ben een Koreaanse softwareontwikkelaar en woon in Colorado. Bij **Soundpatrol** werk ik als Rust-ontwikkelaar aan cybersecurity, DevOps, webdiensten en ontwikkeltools, voornamelijk met Rust, Python, PostgreSQL, Kubernetes en Google Cloud.",
            },
            {
              kind: "paragraph",
              text: "Daarvoor leidde ik het backend- en infrastructuurwerk voor een digital twin van een gebouw van Samsung C&T, droeg ik bij aan de backends van de apps van Hyundai, Kia en Genesis en bouwde ik de gegevensverzameling voor een YouTube-analyseplatform. Sinds 2023 draait mijn werk om backenddiensten, datapijplijnen en de infrastructuur daaronder.",
            },
            {
              kind: "paragraph",
              text: "Ik geloof in het bouwen van diensten die moderne hardware en programmeertalen effectief benutten. In de praktijk betekent dat zorgvuldig schemaontwerp, begrensde wachtrijen en caches, expliciete foutafhandeling en deployments die je kunt inspecteren en herstellen. Ik help collega’s graag op weg met Rust en ik wil werken met mensen die trots zijn op hun vak.",
            },
          ],
        },
        {
          id: "results",
          heading: "Geselecteerde resultaten",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "Lagere maandelijkse cloudkosten voor de digital twin van Samsung C&T, van ongeveer $5.000 naar $150",
                },
                {
                  value: "80",
                  count: 80,
                  label: "REST-endpoints geïmplementeerd voor de digital twin, met domeinlogica uit de bouwtechniek",
                },
                {
                  value: "9 maanden",
                  count: 9,
                  suffix: " maanden",
                  label: "De digital twin draaide in productie zonder onderbreking van de runtime",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "Hogere doorvoer bij gemengde leesbelasting op de backend van deze site dankzij [PGO en BOLT](/about-blog/performance)",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Statisch gelinkte servicecontainers uit multi-stage builds",
                },
                {
                  value: "30.000",
                  label: "Regels geschreven voor de backend van de digital twin, ongeveer 75% van de code",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Gereedschapskist",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Talen",
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
                  label: "Infrastructuur",
                  items: ["Kubernetes", "Helm", "Docker", "OpenTofu / Terraform", "GitHub Actions", "Google Cloud", "AWS", "Linux"],
                },
                {
                  label: "Betrouwbaarheid en beveiliging",
                  items: ["OpenTelemetry", "IAM", "TLS", "Network policy", "Credential lifecycle", "Incident response"],
                },
                {
                  label: "Interfaces en apparaten",
                  items: ["SolidJS", "Leptos", "Slint", "WebAssembly", "WebRTC", "BACnet", "Modbus"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "Meer over mij",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Ervaring",
                  href: "/about/experience",
                  meta: "2016 - heden",
                  text: "Functies van Soundpatrol terug tot mijn eerste stage, met de projecten en resultaten erachter.",
                },
                {
                  title: "Projecten",
                  href: "/about/projects",
                  meta: "Rust · open source",
                  text: "Sillok, EU5 Location DB, deze website en ander zelfstandig en open-sourcewerk.",
                },
                {
                  title: "Achtergrond",
                  href: "/about/background",
                  meta: "Opleiding · onderzoek",
                  text: "Opleiding, gepubliceerd onderzoek, prijzen, talen en werk buiten de software.",
                },
                {
                  title: "Hoe deze site werkt",
                  href: "/about-blog",
                  meta: "Techstack",
                  text: "De host, de backend, de frontend, de subsystemen en de prestatiemetingen achter cyhdev.com.",
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
              text: "E-mail is de betrouwbaarste manier om mij te bereiken. Wil je meedoen op de Minecraft-server, laat dat dan ook in je e-mail weten.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "Over mij",
      title: "Ervaring",
      lede: "Backend-, infrastructuur- en beveiligingswerk bij een Amerikaanse startup, voor Koreaanse zakelijke klanten en in een eerste stage, gevolgd door het werk van voordat ik in de software terechtkwam.",
      sections: [
        {
          id: "software",
          heading: "Softwareontwikkeling",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Rust-ontwikkelaar",
                  period: "mei 2026 - heden",
                  location: "Verenigde Staten · Op afstand",
                  summary: "Cybersecurity, DevOps, ontwikkeling van webdiensten en ontwikkeltools voor een bedrijf dat muziek en audio detecteert.",
                  highlights: [
                    "Onderhoud Kubernetes-infrastructuur op Google Cloud, infrastructure as code en gereviewde deploymentworkflows die kortlevende inloggegevens per omgeving gebruiken.",
                    "Plan en voer productiemigraties van datastores uit, waarbij ik de gemigreerde staat controleer en het herstel oefen vóór elke omschakeling.",
                    "Breng metrics, logs en traces samen in OpenTelemetry en bouw Rust-tooling voor servicegezondheid, diagnostiek en inzicht in kosten.",
                    "Werk aan beveiligingsgereedheid: toegangsbeheer, rotatie en verloopbewaking van inloggegevens, beleid, incidentrespons en auditbewijs.",
                    "Bouw interne diensten en tools in Rust en Python voor operationele en supportworkflows.",
                  ],
                  tags: ["Rust", "Python", "PostgreSQL", "Kubernetes", "Google Cloud", "OpenTofu", "OpenTelemetry"],
                },
                {
                  organization: "GenesisNest",
                  title: "Softwareontwikkelaar",
                  period: "jan. 2025 - jul. 2025",
                  location: "Seongnam, Zuid-Korea",
                  summary: "Backenddiensten en infrastructuurtooling voor interne platforms en de officiële fanapp van een K-popgroep.",
                  highlights: [
                    "Leverde API’s, bugfixes en integraties voor de backends van de officiële apps van Hyundai, Kia en Genesis, in een grote Java-codebase en in afstemming met leveranciers, voor diensten die tientallen miljoenen mensen in Azië en Europa gebruiken.",
                    "Bouwde internationaliseringspijplijnen in Rust en Python die vertaalde inhoud omzetten en in bedrijfsdatabases laden, in afstemming met de databasebeheerders die verantwoordelijk zijn voor datasets die aan fabriekssystemen zijn gekoppeld.",
                    "Ontwikkelde beheertools voor de infrastructuur van interne diensten.",
                  ],
                  tags: ["Java", "Spring Boot", "Rust", "Python"],
                },
                {
                  organization: "pampam Inc.",
                  title: "Softwareontwikkelaar (contract)",
                  period: "nov. 2024 - dec. 2024",
                  location: "Seoel, Zuid-Korea",
                  highlights: [
                    "Ontwierp het PostgreSQL-schema en de Rust-backend voor een AI-ondersteund YouTube-analyseplatform en leidde de deployment over AWS en Google Cloud.",
                    "Bouwde workflows voor het verzamelen van gegevens via de YouTube-API, voor tientallen miljoenen reacties en gebruikers, waarin kanaalgegevens, transcripties en pagina-inhoud werden gecombineerd.",
                    "Integreerde gehoste taalmodellen en open-sourcemodellen op de eigen server om samenvattingen en marketinginzichten te genereren.",
                  ],
                  tags: ["Rust", "PostgreSQL", "AWS", "Google Cloud"],
                },
                {
                  organization: "EAN Technology Co. Ltd.",
                  title: "Backend-softwareontwikkelaar (lead)",
                  period: "aug. 2023 - aug. 2024",
                  location: "Seoel, Zuid-Korea",
                  summary: "Leidde de ontwikkeling van backend, database en cloudinfrastructuur voor twee platforms voor gebouwen en energie.",
                  highlights: [],
                  projects: [
                    {
                      name: "Digital twin van het Samsung C&T Global Engineering Center",
                      highlights: [
                        "Verving een Express-backend en een opzet met cloudmicroservices door een geoptimaliseerde Rust/Axum-monoliet en verlaagde de maandelijkse infrastructuurkosten van ongeveer **$5.000 naar $150**.",
                        "Schreef ongeveer **30.000 regels**, zo’n 75% van de backend, waaronder **80 REST-endpoints** met domeinlogica uit de bouwtechniek, gegevensintegratie en realtime-operaties.",
                        "Bracht veel paden voor verzoeken met een P99 van meerdere seconden terug naar enkele tot tientallen milliseconden op een deployment met twee cores en 4 GB, met asynchrone Rust, multithreading, caching en tuning van het PostgreSQL-schema en de queries.",
                        "Integreerde enkele duizenden sensoren, meters en regelaars van HVAC- en watersystemen via BACnet en Modbus, ter plaatse op het hoofdkantoor van Samsung C&T.",
                        "Draaide negen maanden in productie zonder onderbreking van de runtime, met ongeveer 90% dekking van de foutafhandeling.",
                        "Bouwde CI/CD met GitHub-runners, AWS CodeDeploy en Docker; statische, op de processor afgestemde builds leverden draagbare containers van ongeveer 20 MB op.",
                        "Koppelde inferentie van voorspellende modellen via ECS, EC2, SQS, SNS, S3 en SageMaker, implementeerde de beveiligingseisen van Samsung SDS en integreerde de backend met een Unreal Engine 4-interface van een partnerstudio.",
                      ],
                    },
                    {
                      name: "AI-dienst voor energiebeheer",
                      highlights: [
                        "Ontwierp het PostgreSQL-schema en de Rust/Axum-API van de grond af en schreef ongeveer **25.000 regels**, zo’n 70% van de backend.",
                        "Koos voor directe SQL via tokio-postgres in plaats van een ORM, voor meer controle over queries en compileertijden, en paste daarbij de lessen uit de digital twin toe.",
                        "Hielp twee ontwikkelaars die uit Node.js en Java kwamen binnen ongeveer een maand productief te worden in Rust, in een team van negen backend-, frontend- en datasciencespecialisten.",
                      ],
                    },
                  ],
                  tags: ["Rust", "Axum", "PostgreSQL", "AWS", "Docker", "BACnet", "Modbus"],
                },
                {
                  organization: "Artifyc Inc.",
                  title: "Stagiair softwareontwikkeling",
                  period: "aug. 2022 - mrt. 2023",
                  location: "Austin, Texas · Op afstand",
                  highlights: [
                    "Bouwde een Discord-bot in Python die via de Reddit-API verzoeken om kunstopdrachten opspoorde en in een communitykanaal plaatste, elke vijf minuten ingepland op AWS Lambda.",
                    "Implementeerde Lambda-logica voor valutaomrekening, betaling en berekening van toeslagen per rechtsgebied.",
                    "Droeg wijzigingen bij aan de React-frontend, voor lay-out, assets en teksten.",
                  ],
                  tags: ["Python", "AWS Lambda", "React"],
                },
              ],
            },
          ],
        },
        {
          id: "other",
          heading: "Voor de software",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Korea Institute of Maritime Science and Technology",
                  title: "Vertaler en tolk",
                  period: "jun. - jul. 2023, aug. 2024",
                  location: "Seoel, Zuid-Korea · Contract",
                  highlights: [
                    "Tolkte in het Engels, Koreaans en Noors op een maritieme onderzoeksconferentie, bijgewoond door de Noorse ambassadeur in Korea, een Noorse staatssecretaris, de leiding van het instituut en de voorzitter van Dongwon Group.",
                    "Vertaalde toespraken en begeleidde medewerkers van het instituut en de directeur bij een internationale conferentie en bij een evenement van het Amerikaanse ministerie van Energie over een gezamenlijk project.",
                  ],
                },
                {
                  organization: "Leger van de Republiek Korea · 2e Infanteriedivisie van het Amerikaanse leger",
                  title: "Sergeant, KATUSA-verbindingspersoon en instructeur",
                  period: "mrt. 2016 - dec. 2017",
                  location: "Dongducheon, Zuid-Korea",
                  highlights: [
                    "Diende in het Headquarters Battalion van de divisie via het [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army)-programma, in een gezamenlijke Koreaans-Amerikaanse omgeving.",
                    "Gaf instructie aan ongeveer 6.000 Amerikaanse en Koreaanse militairen en begeleidde hun aankomst, in cursussen over fitheid, cultuur, geschiedenis en preventie.",
                    "Leidde een team van 13 personen en hield de gereedheid op peil tijdens de [crisis van 2017](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) op het schiereiland.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Logistiek medewerker (laden)",
                  period: "2020 - 2022",
                  location: "Zuid-Korea · Incidentele contracten",
                  highlights: [
                    "Nachtdiensten in het magazijn naast mijn studie.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "Over mij",
      title: "Projecten",
      lede: "Zelfstandig werk en open-sourcewerk, vooral in Rust. Interactieve demo’s staan op de [projectenpagina](/projects).",
      sections: [
        {
          id: "featured",
          heading: "Uitgelicht",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "Een werkkroniek op de opdrachtregel. Doelstellingen, afgeronde taken, correcties en intrekkingen worden vastgelegd als gebeurtenissen waaraan alleen wordt toegevoegd, met geïndexeerde actuele projecties, en een archief op basis van Git voegt de geschiedenis van verschillende machines samen.",
                  tags: ["Event sourcing", "Synchronisatie"],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Een desktop- en browsertoepassing voor kaartgegevens van Europa Universalis V, met ingebedde gecomprimeerde datasets, vooraf berekende zoek- en sorteerindexen, categorische filters en tabellen waarvan je het formaat kunt aanpassen. [Probeer het hier](/eu5-locations-db).",
                  tags: ["Native + web", "Indexen"],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Deze site: publicaties, fotografie, een forum, livechat met gesprekken, een Minecraft-terreinverkenner en WebAssembly-demo’s, geleverd door één Rust-binary. [Hoe de site is gebouwd](/about-blog).",
                  tags: ["Monorepo", "Zelf gehost"],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "Een generator en formatter voor UUIDv4 met uitvoer naar terminal en bestand, optionele controle op duplicaten en opmaak die compatibel is met Python en JSON; in een vroege lokale benchmark ongeveer drie keer sneller dan uuidgen.",
                  tags: ["Doorvoer"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "Ander werk",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: een [upstream-fix](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) voor de Minecraft-server in Rust, die de overworld-instellingen van Paper behoudt bij importeren en opslaan; open voor review.",
                "**Minecraft-kaarttooling**: een Paper-plugin en Rust-diensten achter de [terreinverkenner](/minecraft) van deze site, met een compact binair biometegelformaat en voorspellingen van biomen op basis van de seed.",
                "**Bestandssysteemsynchronisatie**: een Rust-hulpprogramma voor tweerichtingssynchronisatie van mappen en het opruimen van buildartefacten tussen verschillende bestandssystemen, met dry runs, operatiejournalen, optionele checksums en beveiligingen tegen grote onbedoelde verwijderingen.",
                "**impulsr**: een tool die YouTube-video’s transcribeert en reacties verzamelt en daar met taalmodellen samenvattingen van maakt voor marketingwerk.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Hoe ik bouw",
          blocks: [
            {
              kind: "paragraph",
              text: "Ik geef de voorkeur aan kleine systemen die je kunt inspecteren: één binary waar één binary volstaat, een expliciete grens op elke wachtrij en cache, en dataformaten die zijn gekozen voor het toegangspatroon in plaats van uit gewoonte.",
            },
            {
              kind: "paragraph",
              text: "Metingen komen vóór beweringen; de [prestatienotities](/about-blog/performance) over deze site laten zien hoe ze worden uitgevoerd.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "Over mij",
      title: "Achtergrond",
      lede: "Opleiding, onderzoek, kwalificaties en interesses buiten de software.",
      sections: [
        {
          id: "education",
          heading: "Opleiding",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Sungkyunkwan University",
                  title: "B.Eng. in Software Engineering",
                  period: "mrt. 2015 - aug. 2023",
                  location: "Suwon, Zuid-Korea",
                  summary: "Afdeling Computer Science and Engineering. De looptijd van de opleiding omvat de verplichte militaire dienst.",
                  highlights: [
                    "Vakken over embedded software, logische schakelingen, besturingssystemen, netwerken, low-level programmeren en elektronica, naast de praktijk van software engineering.",
                    "Sungkyunkwan University gaat terug tot 1398, toen zij de koninklijke academie van de Joseon-dynastie was, en is een van de vooraanstaande onderzoeksuniversiteiten van Korea.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Onderzoek",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Korean Language NLP Model Based Emotional Analysis of Social Media Communities.** Paste KoBERT toe op sentimentanalyse en categorisering van Koreaanstalige sociale media, met onderzoek naar emotionele verandering in de beginperiode van COVID-19.",
            },
            {
              kind: "paragraph",
              text: "Het project ontving de Bronzen Prijs bij de prijzen voor afstudeerscripties en -projecten van de universiteit, uit ongeveer 60 inzendingen.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Prijzen en kwalificaties",
          blocks: [
            {
              kind: "table",
              columns: ["Jaar", "Erkenning", "Details"],
              rows: [
                ["2023", "**Bronzen Prijs**", "Prijzen voor afstudeerscripties en -projecten van Sungkyunkwan University"],
                ["2023", "**IELTS 8,5 / 9,0**", "CEFR C2"],
                ["2021", "**7e plaats**", "Capstone Design and Idea Hackathon, Korean Society for Internet Information"],
                ["2020", "**3e plaats**", "Fintech-hackathon, Seoul National University of Science and Technology"],
                ["2018", "**TOEFL 117 / 120**", "CEFR C2"],
              ],
            },
          ],
        },
        {
          id: "languages",
          heading: "Talen",
          blocks: [
            {
              kind: "list",
              items: [
                "**Koreaans**: moedertaal.",
                "**Engels**: professionele beheersing (C2).",
                "**Frans, Duits, Spaans en Mandarijn**: basiskennis.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "Buiten de software",
          blocks: [
            {
              kind: "list",
              items: [
                "**Fotografie**: landschaps- en portretfotografie sinds 2010, met ongeveer 30.000 gearchiveerde opnamen. Een selectie staat in de [galerij](/photographs).",
                "**Journalistiek**: studentverslaggever voor de Sungkyun Times, het Engelstalige tijdschrift van de universiteit, over technologie, beveiliging en internationale zaken.",
                "**Lesgeven en vrijwilligerswerk**: gaf Engelse les in een buurthuis en deed vrijwilligerswerk in een kringloopwinkel die wordt gerund door mensen met een beperking.",
                "**Vertalen**: tolk- en vertaalwerk voor overheidsinstellingen; zie [Ervaring](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
