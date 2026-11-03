import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "novembre 2026",
  chrome: {
    overview: "Vue d’ensemble",
    updated: "Dernière mise à jour",
    onThisPage: "Sur cette page",
    sectionNavigation: "Pages À propos",
    previous: "Précédent",
    next: "Suivant",
    readingTime: "{minutes} min de lecture",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "À propos",
      lede: "Ingénieur backend et infrastructure, je travaille avec Rust, PostgreSQL et les plateformes cloud. Je conçois des services qui exploitent efficacement le matériel sur lequel ils tournent, et je prends en charge le déploiement, la supervision et la sécurité qui les maintiennent en service.",
      sections: [
        {
          id: "introduction",
          heading: "Présentation",
          blocks: [
            {
              kind: "paragraph",
              text: "Je suis un ingénieur logiciel coréen installé dans le Colorado. Chez **Soundpatrol**, je suis ingénieur Rust et je travaille sur la cybersécurité, le DevOps, les services web et les outils de développement, principalement avec Rust, Python, PostgreSQL, Kubernetes et Google Cloud.",
            },
            {
              kind: "paragraph",
              text: "Auparavant, j’ai dirigé le backend et l’infrastructure d’un jumeau numérique de bâtiment pour Samsung C&T, contribué aux backends des applications Hyundai, Kia et Genesis, et développé la collecte de données d’une plateforme d’analyse YouTube. Depuis 2023, mon travail se concentre sur les services backend, les pipelines de données et l’infrastructure qui les porte.",
            },
            {
              kind: "paragraph",
              text: "Je crois qu’il faut concevoir des services qui tirent pleinement parti du matériel et des langages de programmation modernes. Concrètement, cela signifie une conception soignée des schémas, des files d’attente et des caches bornés, une gestion explicite des erreurs et des déploiements que l’on peut inspecter et restaurer. J’aime aider mes collègues à se mettre à Rust, et je souhaite travailler avec des personnes fières de leur métier.",
            },
          ],
        },
        {
          id: "results",
          heading: "Quelques résultats",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97 %",
                  count: 97,
                  suffix: " %",
                  label: "Baisse des dépenses cloud mensuelles du jumeau numérique Samsung C&T, d’environ 5 000 $ à 150 $",
                },
                {
                  value: "80",
                  count: 80,
                  label: "Points de terminaison REST implémentés pour le jumeau numérique, avec une logique métier d’ingénierie de la construction",
                },
                {
                  value: "9 mois",
                  count: 9,
                  suffix: " mois",
                  label: "Exploitation en production du jumeau numérique sans arrêt en cours d’exécution",
                },
                {
                  value: "+20 %",
                  count: 20,
                  prefix: "+",
                  suffix: " %",
                  label: "Gain de débit en lectures mixtes sur le backend de ce site grâce à [PGO et BOLT](/about-blog/performance)",
                },
                {
                  value: "~20 Mo",
                  count: 20,
                  prefix: "~",
                  suffix: " Mo",
                  label: "Conteneurs de services liés statiquement, issus de builds multi-étapes",
                },
                {
                  value: "30 000",
                  label: "Lignes écrites pour le backend du jumeau numérique, soit environ 75 % de son code",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Boîte à outils",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Langages",
                  items: ["Rust", "Python", "TypeScript", "SQL", "Java", "C", "C++"],
                },
                {
                  label: "Backend",
                  items: ["Axum", "Tokio", "Diesel", "tokio-postgres", "Spring Boot", "FastAPI", "WebSockets", "gRPC", "Protocol Buffers"],
                },
                {
                  label: "Données",
                  items: ["PostgreSQL", "Valkey / Redis", "SQLite / Turso", "MySQL", "FAISS", "tantivy"],
                },
                {
                  label: "Infrastructure",
                  items: ["Kubernetes", "Helm", "Docker", "OpenTofu / Terraform", "GitHub Actions", "Google Cloud", "AWS", "Linux"],
                },
                {
                  label: "Fiabilité et sécurité",
                  items: ["OpenTelemetry", "IAM", "TLS", "Network policy", "Credential lifecycle", "Incident response"],
                },
                {
                  label: "Interfaces et appareils",
                  items: ["SolidJS", "Leptos", "Slint", "WebAssembly", "WebRTC", "BACnet", "Modbus"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "En savoir plus",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Expérience",
                  href: "/about/experience",
                  meta: "2016 - aujourd’hui",
                  text: "Mes postes, de Soundpatrol jusqu’à mon premier stage, avec les projets et les résultats de chacun.",
                },
                {
                  title: "Projets",
                  href: "/about/projects",
                  meta: "Rust · open source",
                  text: "Sillok, EU5 Location DB, ce site et d’autres travaux indépendants et open source.",
                },
                {
                  title: "Parcours",
                  href: "/about/background",
                  meta: "Formation · recherche",
                  text: "Formation, recherche publiée, distinctions, langues et activités en dehors du logiciel.",
                },
                {
                  title: "Fonctionnement du site",
                  href: "/about-blog",
                  meta: "Technologies",
                  text: "L’hôte, le backend, le frontend, les sous-systèmes et les mesures de performance derrière cyhdev.com.",
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
                "**E-mail** : [younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub** : [github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn** : [linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "L’e-mail est le moyen le plus fiable de me joindre. Si vous souhaitez rejoindre le serveur Minecraft, précisez-le également dans votre message.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "À propos",
      title: "Expérience",
      lede: "Travaux de backend, d’infrastructure et de sécurité dans une startup américaine, pour de grandes entreprises coréennes et lors d’un premier stage, puis les expériences antérieures au logiciel.",
      sections: [
        {
          id: "software",
          heading: "Ingénierie logicielle",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Ingénieur Rust",
                  period: "mai 2026 - aujourd’hui",
                  location: "États-Unis · Télétravail",
                  summary: "Cybersécurité, DevOps, développement de services web et outils de développement pour une entreprise de détection de musique et d’audio.",
                  highlights: [
                    "Maintenance de l’infrastructure Kubernetes sur Google Cloud, de l’infrastructure en tant que code et de workflows de déploiement relus, qui utilisent des identifiants éphémères propres à chaque environnement.",
                    "Planification et réalisation de migrations de stockages de données en production, avec vérification de l’état migré et répétition de la restauration avant chaque bascule.",
                    "Regroupement des métriques, journaux et traces sur OpenTelemetry, et développement d’outils Rust pour l’état des services, le diagnostic et la visibilité des coûts.",
                    "Travail sur la préparation en matière de sécurité : administration des accès, rotation des identifiants et suivi de leur expiration, politiques, réponse aux incidents et preuves d’audit.",
                    "Développement de services et d’outils internes en Rust et Python pour les processus d’exploitation et de support.",
                  ],
                  tags: ["Rust", "Python", "PostgreSQL", "Kubernetes", "Google Cloud", "OpenTofu", "OpenTelemetry"],
                },
                {
                  organization: "GenesisNest",
                  title: "Ingénieur logiciel",
                  period: "janv. 2025 - juil. 2025",
                  location: "Seongnam, Corée du Sud",
                  summary: "Services backend et outils d’infrastructure pour des plateformes internes et l’application officielle des fans d’un groupe de K-pop.",
                  highlights: [
                    "Contribution d’API, de corrections de bogues et d’intégrations aux backends des applications officielles Hyundai, Kia et Genesis, au sein d’une vaste base de code Java et en coordination avec les fournisseurs de services utilisés par des dizaines de millions de personnes en Asie et en Europe.",
                    "Création de pipelines d’internationalisation en Rust et Python qui transforment et chargent les contenus traduits dans les bases de données de l’entreprise, en coordination avec les administrateurs de bases de données responsables de jeux de données reliés aux systèmes des usines.",
                    "Développement d’outils de gestion d’infrastructure pour les services internes.",
                  ],
                  tags: ["Java", "Spring Boot", "Rust", "Python"],
                },
                {
                  organization: "pampam Inc.",
                  title: "Ingénieur logiciel contractuel",
                  period: "nov. 2024 - déc. 2024",
                  location: "Séoul, Corée du Sud",
                  highlights: [
                    "Conception du schéma PostgreSQL et du backend Rust d’une plateforme d’analyse YouTube assistée par IA, et pilotage de son déploiement sur AWS et Google Cloud.",
                    "Création de processus de collecte via l’API YouTube couvrant des dizaines de millions de commentaires et d’utilisateurs, combinant données de chaînes, transcriptions et contenu des pages.",
                    "Intégration de modèles de langage hébergés et de modèles open source exécutés sur le serveur pour produire des synthèses et des analyses marketing.",
                  ],
                  tags: ["Rust", "PostgreSQL", "AWS", "Google Cloud"],
                },
                {
                  organization: "EAN Technology Co. Ltd.",
                  title: "Ingénieur logiciel backend principal",
                  period: "août 2023 - août 2024",
                  location: "Séoul, Corée du Sud",
                  summary: "Direction du développement backend, base de données et infrastructure cloud de deux plateformes dédiées au bâtiment et à l’énergie.",
                  highlights: [],
                  projects: [
                    {
                      name: "Jumeau numérique du Global Engineering Center de Samsung C&T",
                      highlights: [
                        "Remplacement d’un backend Express et d’un ensemble de microservices cloud par un monolithe Rust/Axum optimisé, ce qui a réduit les dépenses d’infrastructure mensuelles d’environ **5 000 $ à 150 $**.",
                        "Rédaction d’environ **30 000 lignes**, soit près de 75 % du backend, dont **80 points de terminaison REST** portant la logique métier d’ingénierie de la construction, l’intégration des données et les opérations en temps réel.",
                        "Réduction de nombreux chemins de requête dont la latence P99 atteignait plusieurs secondes à quelques millisecondes ou quelques dizaines de millisecondes, sur un déploiement à deux cœurs et 4 Go, grâce au Rust asynchrone, au multithreading, à la mise en cache et à l’optimisation du schéma et des requêtes PostgreSQL.",
                        "Intégration sur site, au siège de Samsung C&T, de plusieurs milliers de capteurs, de compteurs et de contrôleurs CVC et de réseaux d’eau via BACnet et Modbus.",
                        "Neuf mois d’exploitation en production sans arrêt en cours d’exécution, avec une couverture de gestion des erreurs d’environ 90 %.",
                        "Mise en place de la CI/CD avec des runners GitHub, AWS CodeDeploy et Docker ; des builds statiques optimisés pour le processeur produisaient des conteneurs portables d’environ 20 Mo.",
                        "Raccordement de l’inférence de modèles prédictifs via ECS, EC2, SQS, SNS, S3 et SageMaker, mise en œuvre des exigences de sécurité de Samsung SDS et intégration du backend à une interface Unreal Engine 4 réalisée par un studio partenaire.",
                      ],
                    },
                    {
                      name: "Service de gestion de l’énergie par IA",
                      highlights: [
                        "Conception de A à Z du schéma PostgreSQL et de l’API Rust/Axum, et rédaction d’environ **25 000 lignes**, soit près de 70 % du backend.",
                        "Choix du SQL direct via tokio-postgres plutôt que d’un ORM, pour maîtriser les requêtes et les temps de compilation, en appliquant les leçons du jumeau numérique.",
                        "Accompagnement de deux ingénieurs venant de Node.js et de Java, devenus productifs en Rust en un mois environ, au sein d’une équipe de neuf personnes réunissant des ingénieurs backend, frontend et data science.",
                      ],
                    },
                  ],
                  tags: ["Rust", "Axum", "PostgreSQL", "AWS", "Docker", "BACnet", "Modbus"],
                },
                {
                  organization: "Artifyc Inc.",
                  title: "Stagiaire en ingénierie logicielle",
                  period: "août 2022 - mars 2023",
                  location: "Austin, Texas · Télétravail",
                  highlights: [
                    "Création d’un bot Discord en Python qui utilisait l’API Reddit pour repérer les demandes de commandes artistiques et les publier dans un canal communautaire, exécuté toutes les cinq minutes sur AWS Lambda.",
                    "Mise en œuvre d’une logique Lambda pour la conversion de devises, la facturation et le calcul des suppléments selon les juridictions.",
                    "Contribution à des révisions du frontend React portant sur la mise en page, les ressources et les textes.",
                  ],
                  tags: ["Python", "AWS Lambda", "React"],
                },
              ],
            },
          ],
        },
        {
          id: "other",
          heading: "Avant le logiciel",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Korea Institute of Maritime Science and Technology",
                  title: "Traducteur et interprète",
                  period: "juin - juil. 2023, août 2024",
                  location: "Séoul, Corée du Sud · Contrat",
                  highlights: [
                    "Interprétation en anglais, coréen et norvégien lors d’une conférence de recherche maritime en présence de l’ambassadeur de Norvège en Corée, d’un secrétaire d’État norvégien, de la direction de l’institut et du président du groupe Dongwon.",
                    "Traduction de discours et préparation du personnel de l’institut et de son directeur pour une conférence internationale et pour un événement du département de l’Énergie des États-Unis consacré à un projet commun.",
                  ],
                },
                {
                  organization: "Armée de la République de Corée · 2e division d’infanterie de l’armée américaine",
                  title: "Sergent, agent de liaison et instructeur KATUSA",
                  period: "mars 2016 - déc. 2017",
                  location: "Dongducheon, Corée du Sud",
                  highlights: [
                    "Service au sein du bataillon de quartier général de la division dans le cadre du programme [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army), dans un environnement mixte coréano-américain.",
                    "Formation et accueil administratif d’environ 6 000 militaires américains et coréens, avec des cours de condition physique, de culture, d’histoire et de prévention.",
                    "Direction d’une équipe de 13 personnes et maintien de la disponibilité opérationnelle pendant la [crise de 2017](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) dans la péninsule.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Agent de chargement logistique",
                  period: "2020 - 2022",
                  location: "Corée du Sud · Contrats ponctuels",
                  highlights: [
                    "Travail de nuit en entrepôt parallèlement à mes études universitaires.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "À propos",
      title: "Projets",
      lede: "Travaux indépendants et open source, principalement en Rust. Les démonstrations interactives se trouvent sur la [page des projets](/projects).",
      sections: [
        {
          id: "featured",
          heading: "Projets phares",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "Une chronique de travail en ligne de commande. Les objectifs, tâches terminées, corrections et rétractations sont enregistrés sous forme d’événements en ajout seul avec des projections courantes indexées, et une archive adossée à Git fusionne les historiques entre machines.",
                  tags: ["Event sourcing", "Synchronisation"],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Une application de bureau et de navigateur pour les données cartographiques d’Europa Universalis V, avec jeux de données compressés intégrés, index de recherche et de tri précalculés, filtres par catégorie et tableaux redimensionnables. [L’essayer ici](/eu5-locations-db).",
                  tags: ["Natif + web", "Index"],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Ce site : publication, photographie, forum, discussion en direct avec appels, explorateur de terrain Minecraft et démonstrations WebAssembly, servis par un seul binaire Rust. [Comment il est construit](/about-blog).",
                  tags: ["Monorepo", "Auto-hébergé"],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "Un générateur et formateur d’UUIDv4 avec sortie dans le terminal ou dans un fichier, vérification facultative des doublons et formatage compatible Python et JSON ; environ trois fois plus rapide qu’uuidgen lors d’un premier benchmark local.",
                  tags: ["Débit"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "Autres travaux",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin** : un [correctif en amont](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) pour le serveur Minecraft en Rust, qui préserve les paramètres de l’Overworld de Paper à l’import et à l’enregistrement ; en attente de relecture.",
                "**Outils cartographiques Minecraft** : un plugin Paper et des services Rust derrière l’[explorateur de terrain](/minecraft) de ce site, dont un format binaire compact de tuiles de biomes et des prédictions de biomes fondées sur la graine.",
                "**Synchronisation de systèmes de fichiers** : un utilitaire Rust de synchronisation bidirectionnelle de dossiers et de nettoyage des artefacts de build entre systèmes de fichiers hétérogènes, avec simulations, journaux d’opérations, sommes de contrôle facultatives et garde-fous contre les suppressions massives involontaires.",
                "**impulsr** : un outil de transcription YouTube et de collecte de commentaires qui produit des synthèses par modèle de langage pour le marketing.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Ma façon de concevoir",
          blocks: [
            {
              kind: "paragraph",
              text: "Je préfère les systèmes petits et inspectables : un seul binaire quand un seul binaire suffit, une limite explicite pour chaque file d’attente et chaque cache, et des formats de données choisis pour le mode d’accès plutôt que par habitude.",
            },
            {
              kind: "paragraph",
              text: "Les mesures précèdent les affirmations. Quand un benchmark s’avère mesurer son propre client plutôt que le serveur, la correction est elle aussi consignée ; les [notes sur les performances](/about-blog/performance) de ce site en sont un exemple.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "À propos",
      title: "Parcours",
      lede: "Formation, recherche, qualifications et centres d’intérêt en dehors du logiciel.",
      sections: [
        {
          id: "education",
          heading: "Formation",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Sungkyunkwan University",
                  title: "Licence en génie logiciel (B.Eng.)",
                  period: "mars 2015 - août 2023",
                  location: "Suwon, Corée du Sud",
                  summary: "Département d’informatique et d’ingénierie. La durée des études inclut le service militaire obligatoire.",
                  highlights: [
                    "Cours couvrant les logiciels embarqués, les circuits logiques, les systèmes d’exploitation, les réseaux, la programmation bas niveau et l’électronique, en plus des pratiques du génie logiciel.",
                    "L’université Sungkyunkwan fait remonter ses origines à 1398, comme académie royale de la dynastie Joseon, et compte parmi les principales universités de recherche de Corée.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Recherche",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Analyse des émotions dans les communautés de réseaux sociaux à l’aide d’un modèle de TAL en coréen.** Application de KoBERT à l’analyse des sentiments et à la catégorisation de publications en coréen sur les réseaux sociaux, afin d’étudier l’évolution des émotions au début de la pandémie de COVID-19.",
            },
            {
              kind: "paragraph",
              text: "Le projet a reçu le prix de bronze du concours de mémoires et projets de fin d’études de l’université, parmi environ 60 participations.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Distinctions et qualifications",
          blocks: [
            {
              kind: "table",
              columns: ["Année", "Distinction", "Détail"],
              rows: [
                ["2023", "**Prix de bronze**", "Concours de mémoires et projets de fin d’études de l’université Sungkyunkwan"],
                ["2023", "**IELTS 8,5 / 9,0**", "CECR C2"],
                ["2021", "**7e place**", "Hackathon de conception et d’idées (Capstone Design), Korean Society for Internet Information"],
                ["2020", "**3e place**", "Hackathon Fintech, Seoul National University of Science and Technology"],
                ["2018", "**TOEFL 117 / 120**", "CECR C2"],
              ],
            },
          ],
        },
        {
          id: "languages",
          heading: "Langues",
          blocks: [
            {
              kind: "list",
              items: [
                "**Coréen** : langue maternelle.",
                "**Anglais** : maîtrise professionnelle (C2).",
                "**Français, allemand, espagnol et mandarin** : notions de base.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "En dehors du logiciel",
          blocks: [
            {
              kind: "list",
              items: [
                "**Photographie** : photographie de paysage et de portrait depuis 2010, avec environ 30 000 images archivées. Une sélection est visible dans la [galerie](/photographs).",
                "**Journalisme** : reporter étudiant pour le Sungkyun Times, le magazine anglophone de l’université, couvrant la technologie, la sécurité et les affaires internationales.",
                "**Enseignement et bénévolat** : enseignement de l’anglais dans un centre communautaire et bénévolat dans une friperie gérée par des personnes en situation de handicap.",
                "**Traduction** : interprétation et traduction pour des institutions publiques ; voir [Expérience](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
