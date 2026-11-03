import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "novembre 2026",
  chrome: {
    overview: "Vue d’ensemble",
    updated: "Dernière mise à jour",
    onThisPage: "Sur cette page",
    sectionNavigation: "Pages techniques",
    previous: "Précédent",
    next: "Suivant",
    readingTime: "{minutes} min de lecture",
  },
  pages: {
    overview: {
      eyebrow: "À propos de ce site",
      title: "Technologies du blog",
      lede: "Un binaire Rust, une base de données PostgreSQL, une petite machine. Ces pages décrivent comment le site est construit, hébergé, sécurisé et mesuré, et pourquoi il a cette forme.",
      sections: [
        {
          id: "at-a-glance",
          heading: "En bref",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53 889 req/s",
                  label: "Débit médian en lectures publiques mixtes avec PGO et BOLT sur quatre cœurs serveur",
                },
                {
                  value: "~150 µs",
                  label: "Aller-retour observé avec la base de données via un socket UNIX, contre ~600 µs en TCP sur localhost",
                },
                {
                  value: "130 Kio",
                  count: 130,
                  suffix: " Kio",
                  label: "Budget gzip des ressources frontend initiales, imposé lors de la construction du bundle",
                },
                {
                  value: "8",
                  count: 8,
                  label: "Langues d’interface, chacune téléchargée uniquement lorsqu’elle est sélectionnée",
                },
                {
                  value: "0,15 ms",
                  label: "Temps médian de réception complète d’une tuile en HTTP/2 après activation de `TCP_NODELAY`, contre 41 ms auparavant",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Processus qui gère TLS, l’API, les WebSockets, les médias WebRTC et le frontend intégré",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Parcours d’une requête",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Navigateur",
                  detail: "HTTP/2 ou HTTP/1.1 sur TLS. Le frontend est mis en cache sous des URL immuables contenant une empreinte.",
                },
                {
                  label: "Admission",
                  detail: "Les connexions TCP sont admises avant TLS : 4 096 au total et 64 par adresse ou par /64 IPv6.",
                },
                {
                  label: "rustls",
                  detail: "TLS se termine dans le processus Rust, avec `TCP_NODELAY` sur chaque socket.",
                },
                {
                  label: "Axum",
                  detail: "En-têtes de sécurité, sessions, limitation de débit, compression et limites de taille du corps, puis le gestionnaire de la fonctionnalité.",
                },
                {
                  label: "Cache · Diesel",
                  detail: "Des caches mémoire bornés répondent aux lectures fréquentes ; tout le reste passe par Diesel asynchrone.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "Sur le même hôte via un socket UNIX, avec les E/S asynchrones io_uring.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Chapitres",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Infrastructure",
                  href: "/about-blog/infrastructure",
                  meta: "Hôte · Gentoo · réseau",
                  text: "La machine, le système d’exploitation et le système de fichiers, le réseau, et pourquoi il n’y a pas de proxy inverse.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "La pile du service, le modèle de données, les limites d’exécution, les en-têtes de sécurité et les contrats d’API.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "Le squelette de l’application, le budget de chargement, la localisation, le système de design et les animations.",
                },
                {
                  title: "Sous-systèmes",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "La discussion en direct avec une SFU intégrée au processus, l’explorateur Minecraft, les recherches Geo-IP et WebAssembly.",
                },
                {
                  title: "Performances",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Les builds optimisés, le benchmark qui mesurait la mauvaise chose et les chiffres corrigés.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Vérification des modifications",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy sur les cibles natives et WebAssembly comme contrôle d’implémentation.",
                "Un workflow de validation couvrant le formatage, les tests unitaires natifs, la dérive du contrat OpenAPI, ainsi que la vérification de types, le lint, les tests unitaires et le build du frontend.",
                "Playwright sur Chromium et WebKit, avec des contrôles d’accessibilité axe et des vérifications de reflow mobile jusqu’à 320 pixels CSS.",
                "Des tests d’intégration et de retour arrière des migrations sur une instance PostgreSQL jetable, ainsi qu’une recherche locale de secrets avec gitleaks.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Technologies",
      title: "Infrastructure",
      lede: "Une petite machine à domicile, une distribution Linux compilée depuis les sources, et aucune couche dont le trafic n’a pas besoin.",
      sections: [
        {
          id: "host",
          heading: "L’hôte",
          blocks: [
            {
              kind: "paragraph",
              text: "Le site tourne sur un [mini-serveur](https://store.minisforum.com/products/minisforum-um690l-slim) à mon domicile : un processeur mobile Ryzen 9 à huit cœurs montant jusqu’à 4,9 GHz, 32 Go de RAM et un SSD NVMe de 1 To. À environ 400 $, il coûte moins qu’un an de matériel cloud nettement moins puissant, et il reflète mon goût pour l’auto-hébergement.",
            },
            {
              kind: "paragraph",
              text: "Il exécute le serveur backend et frontend combiné, PostgreSQL et un serveur Minecraft. Si vous souhaitez jouer, écrivez-moi.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo et btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) n’est pas le choix évident pour un serveur ; dans un cadre professionnel, je choisirais Debian stable sur ext4. Ici, j’aime compiler moi-même les paquets avec des optimisations pour le processeur exact.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) apporte instantanés, compression et copie sur écriture, mais la copie sur écriture fragmente fortement les fichiers de base de données. Les répertoires de données de PostgreSQL et de Minecraft en sont donc exclus. Un [benchmark de systèmes de fichiers](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) montre btrfs en configuration par défaut derrière ext4 et XFS ; je soupçonne que la désactivation de la copie sur écriture comble l’essentiel de cet écart, ce qui ferait une mesure intéressante.",
            },
          ],
        },
        {
          id: "network",
          heading: "Réseau",
          blocks: [
            {
              kind: "paragraph",
              text: "Le trafic arrive par une connexion Xfinity filaire à 1 Gbit/s ; le modem du fournisseur ne prend pas en charge l’Ethernet 2,5 Gbit/s, ce que ce site ne remarquera pas. Route 53 héberge le DNS.",
            },
            {
              kind: "paragraph",
              text: "Sur la machine, il n’y a ni proxy inverse, ni environnement d’exécution de conteneurs, ni maillage de services. PostgreSQL tourne sur le système d’exploitation, et un seul binaire Rust sert le frontend et l’API. Les requêtes HTTP simples sont redirigées vers HTTPS par le même processus.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "Sockets UNIX",
          blocks: [
            {
              kind: "paragraph",
              text: "Le serveur accède à PostgreSQL via un socket UNIX plutôt qu’en TCP sur localhost, ce qui contourne entièrement la pile réseau. Des [benchmarks](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) montrent des débits de transactions presque doublés ; ici, j’ai observé des allers-retours de seulement **150 µs**, contre environ **600 µs** via localhost. C’est démodé et sans intérêt pour la plupart des déploiements cloud, et c’est plus rapide.",
            },
            {
              kind: "paragraph",
              text: "PostgreSQL 18, publié en septembre 2025, a ajouté les [E/S asynchrones](https://pganalyze.com/blog/postgres-18-async-io), et io_uring est activé ici, ce qui accélère nettement les lectures.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Build et déploiement",
          blocks: [
            {
              kind: "list",
              items: [
                "Le build du frontend est intégré au binaire backend avec des copies précompressées en zstd et gzip, si bien qu’aucun processus Node.js ne tourne en production.",
                "Les builds optimisés peuvent être entraînés par optimisation guidée par profil et optimisés après l’édition de liens avec BOLT ; voir [Performances](/about-blog/performance).",
                "Les identifiants, certificats et données d’exécution restent hors de Git, et une analyse gitleaks vérifie l’arborescence avant que les modifications ne quittent la machine.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Technologies",
      title: "Backend",
      lede: "Rust sur Axum et Tokio, PostgreSQL via Diesel asynchrone, et une limite explicite sur tout ce qu’un client peut garder ouvert.",
      sections: [
        {
          id: "stack",
          heading: "Pile technique",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "Exécution", items: ["Axum", "Tokio", "hyper", "axum-server", "rustls", "mimalloc"] },
                { label: "Données", items: ["PostgreSQL 18", "Diesel", "diesel-async", "tantivy", "bitcode", "zstd"] },
                { label: "Intégrations", items: ["openidconnect", "lettre", "Amazon S3", "webrtc-rs", "image + AVIF"] },
                { label: "Contrats", items: ["utoipa", "OpenAPI", "Swagger UI", "Generated TypeScript"] },
              ],
            },
            {
              kind: "paragraph",
              text: "Le service est un processus unique à état partagé : sessions en mémoire, synchronisation du cache au démarrage et tâches de maintenance planifiées. mimalloc est l’allocateur global ; quand une machine a de vrais cœurs, le comportement de l’allocateur mérite d’être pris au sérieux.",
            },
          ],
        },
        {
          id: "data",
          heading: "Modèle de données",
          blocks: [
            {
              kind: "paragraph",
              text: "Dans les startups et les entreprises coréennes que j’ai connues, MySQL et MariaDB étaient considérées comme les seules bases de données relationnelles sérieuses. PostgreSQL les a depuis longtemps dépassées dans les domaines qui m’importent, des UUID natifs au JSON binaire.",
            },
            {
              kind: "list",
              items: [
                "Les **articles** comportent des slugs, des résumés, des métadonnées JSONB, un état de publication, des compteurs dénormalisés et des associations d’étiquettes. Les commentaires sont imbriqués via un parent nullable, et les votes sur les articles et sur les commentaires ont des tables distinctes.",
                "Les **comptes** conservent le pays et la langue, des tables de rôles et de permissions, ainsi que des photos de profil versionnées dans leur propre table plutôt que sur la ligne utilisateur très sollicitée.",
                "Les **clés** sont des UUIDv7 ordonnés dans le temps : impossibles à énumérer comme des identifiants séquentiels, et bien plus favorables à la localité des arbres B que des UUIDv4 aléatoires.",
                "La **recherche** utilise un index tantivy persisté sur disque, portant sur les titres et les étiquettes des articles et maintenu cohérent avec PostgreSQL, qui reste la source de référence.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Limites d’exécution",
          blocks: [
            {
              kind: "list",
              items: [
                "Les connexions sont admises avant TLS par un limiteur partagé : 4 096 connexions ouvertes et 64 par adresse IPv4 ou /64 IPv6. Les sockets refusés sont fermés sans poignée de main.",
                "Les en-têtes des requêtes HTTP/1 doivent arriver en moins de 20 secondes ; les connexions HTTP/2 reçoivent un ping toutes les 30 secondes et autorisent 128 flux simultanés.",
                "Chaque cache a une limite fixe et une source de référence qu’il peut lire en transparence ; aucun ne grandit avec des clés contrôlées par un client. Le cache des bundles WebAssembly, par exemple, contient au plus 256 Mio.",
                "Les tâches bloquantes, comme l’encodage d’images, s’exécutent dans des pools de workers bornés plutôt que sur les workers Tokio.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "En-têtes de sécurité",
          blocks: [
            {
              kind: "paragraph",
              text: "La Content-Security-Policy est construite une seule fois au démarrage. Les scripts inline ne sont autorisés que par des empreintes SHA-256 calculées à partir de l’`index.html` compilé dans le binaire, si bien que le script de thème exécuté avant le premier rendu ne peut pas diverger de la politique. L’origine principale n’instancie jamais WebAssembly : les démonstrations et l’application EU5 s’exécutent dans des cadres isolés soumis à leurs propres politiques.",
            },
            {
              kind: "paragraph",
              text: "Les cookies de session sont `Secure` et `HttpOnly`, les opérations de superutilisateur vérifient les rôles côté serveur, les comptes peuvent être liés à des identités OpenID Connect, et Swagger UI exige une authentification en production.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Contrats et ressources",
          blocks: [
            {
              kind: "list",
              items: [
                "Les gestionnaires portent des annotations OpenAPI, les clients TypeScript du frontend en sont générés, et la CI échoue lorsque les fichiers générés divergent.",
                "Le manifeste de Vite est intégré avec les ressources. Seuls les fichiers listés dans le manifeste et dotés d’une empreinte reçoivent une politique de cache `immutable` d’un an ; le HTML est toujours revalidé.",
                "Les chemins de ressources manquants renvoient une 404 avec `no-store` plutôt que le squelette de l’application, si bien qu’une page obsolète ne peut pas mettre en cache du HTML sous l’URL d’un script.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Technologies",
      title: "Frontend",
      lede: "Une application SolidJS 2 rendue côté client, compilée en ressources statiques et intégrée au binaire du serveur.",
      sections: [
        {
          id: "stack",
          heading: "Pile technique",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "Application", items: ["SolidJS 2", "Solid Router 2", "TypeScript", "Vite 8"] },
                { label: "Styles", items: ["Tailwind CSS 4", "IBM Plex Sans", "IBM Plex Mono", "OKLCH"] },
                { label: "Composants", items: ["Toast UI Editor", "Leaflet", "Chart.js", "highlight.js", "DOMPurify"] },
                { label: "Tests", items: ["Vitest", "Playwright", "axe-core"] },
              ],
            },
            {
              kind: "paragraph",
              text: "Les signaux à granularité fine de Solid ne mettent à jour que les nœuds du DOM qui dépendent des valeurs modifiées, et ses versions candidates 2.0 permettent aux lectures asynchrones de se suspendre jusqu’à la frontière de chargement la plus proche. Le routeur charge chaque page à la demande, et il n’y a pas de serveur JavaScript en production : le binaire Rust distribue des fichiers statiques.",
            },
          ],
        },
        {
          id: "budget",
          heading: "Budget de chargement",
          blocks: [
            {
              kind: "list",
              items: [
                "Un plugin Vite suit les imports statiques depuis le chunk d’entrée et fait échouer le build si les scripts et styles initiaux dépassent **130 Kio** gzip.",
                "Seuls les textes d’interface en anglais sont inclus dans le chunk initial. Les autres langues, ainsi que les pages À propos de chaque langue, sont des chunks distincts chargés lors de leur sélection.",
                "Des tests de performance mobile avec bridage visent un LCP d’au plus 2,5 s, des interactions d’au plus 200 ms et un CLS d’au plus 0,1.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Localisation",
          blocks: [
            {
              kind: "paragraph",
              text: "L’interface prend en charge l’anglais, le coréen, le français, l’espagnol, le chinois simplifié et traditionnel, le japonais et l’allemand. Un catalogue d’environ 650 clés est validé par rapport au registre du backend, et les espaces réservés doivent survivre à la traduction. Ces pages À propos sont des modules de données typés, si bien qu’une traduction ne peut pas omettre en silence une section présente en anglais.",
            },
          ],
        },
        {
          id: "design",
          heading: "Système de design",
          blocks: [
            {
              kind: "paragraph",
              text: "Le style est chaleureux et technique : papier crème ou noir pur, bordures fines, police à chasse fixe pour tout ce qui est technique et une seule couleur d’accent à la fois. Les couleurs sont des variables CSS sémantiques, si bien que les composants n’ont jamais besoin de variantes pour le mode sombre. Les préréglages d’accent sont définis en OKLCH, qui maintient une luminosité perçue constante d’une teinte à l’autre, et le mode clair ou sombre ainsi que l’accent sont appliqués avant le premier rendu.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Animations",
          blocks: [
            {
              kind: "list",
              items: [
                "L’apparition des sections et la barre de progression de lecture utilisent des animations CSS pilotées par le défilement, qui s’exécutent hors du thread principal lorsque c’est pris en charge.",
                "Les statistiques s’incrémentent grâce à un entier `@property` enregistré, la valeur finale étant présente dans le balisage pour les technologies d’assistance.",
                "Le changement de thème se déploie depuis le bouton de bascule grâce à l’API View Transitions.",
                "Le champ de lignes de niveau de la page d’accueil est un petit shader WebGL qui se charge une fois la page inactive, ne s’exécute que sur les navigateurs de bureau capables, plafonne sa fréquence d’images et se met en pause hors de l’écran.",
                "Tout élément décoratif est ignoré lorsque le système demande des animations réduites.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Technologies",
      title: "Sous-systèmes",
      lede: "Les parties du site plus intéressantes que ce dont un blog a strictement besoin.",
      sections: [
        {
          id: "live-chat",
          heading: "Discussion en direct et appels",
          blocks: [
            {
              kind: "paragraph",
              text: "La discussion passe par un WebSocket avec un codec binaire compact, une modération et un historique borné. Les appels audio et vidéo transitent par une unité de transfert sélectif (SFU) qui s’exécute dans le processus Axum sur webrtc-rs : chaque navigateur maintient une seule connexion pair à pair, et la SFU relaie les médias encodés sans transcodage, via des canaux bornés par publication et un nombre maximal de participants. [Ouvrir la discussion en direct](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Explorateur de terrain Minecraft",
          blocks: [
            {
              kind: "paragraph",
              text: "Un plugin Paper expose un socket UNIX réservé au même utilisateur pour des requêtes bornées sur le monde, avec une seule opération à la fois et des limites strictes de taille de trame. L’[explorateur](/minecraft) affiche les tuiles squaremap avec son propre client Leaflet au lieu d’exécuter le JavaScript de la carte générée.",
            },
            {
              kind: "paragraph",
              text: "Les calques de biomes utilisent **CYBM**, un format binaire compact de tuiles servi compressé en gzip, ou un PNG dont le chunk privé `cyBM` contient les identités exactes des biomes, de sorte que le survol ne devine jamais à partir des couleurs des pixels. Les prédictions fondées sur la graine pour le terrain inexploré proviennent d’un worker de génération de monde distinct, construit sur Pumpkin.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Recherches Geo-IP",
          blocks: [
            {
              kind: "paragraph",
              text: "La [carte des visiteurs](/visitor-board) et la [base Geo-IP](/geo-ip-db) localisent les adresses sans appeler d’API externe. Un outil de prétraitement transforme les fichiers CSV IP2Location LITE en `BTreeMap` indexées par début de plage, les encode avec bitcode et les compresse avec zstd au niveau 22.",
            },
            {
              kind: "table",
              columns: ["Jeu de données", "CSV", "Bundle"],
              numericColumns: [1, 2],
              rows: [
                ["IPv4", "312,5 Mo", "24,9 Mo"],
                ["IPv6", "794,2 Mo", "42,4 Mo"],
              ],
            },
            {
              kind: "paragraph",
              text: "Au démarrage, le serveur décode un bundle à la fois pour limiter le pic de mémoire et applique l’interning aux noms de lieux répétés. Une recherche trouve la dernière plage qui commence à l’adresse ou avant, puis vérifie sa borne de fin.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "Les [bundles de démonstration](/projects) téléversés sont placés sur disque, identifiés par leurs octets magiques, limités à 50 Mio une fois décompressés et servis depuis un cache borné dans des cadres isolés. L’espace de travail compile aussi des démonstrations en Rust, dont un casse-briques et un moteur de lancer de rayons.",
            },
            {
              kind: "paragraph",
              text: "[EU5 Location DB](/eu5-locations-db) est une application Slint compilée en WebAssembly. La page transmet son mode clair ou sombre via un petit protocole de messages, si bien que la palette intégrée change avec le site.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Chaîne de traitement des photos",
          blocks: [
            {
              kind: "paragraph",
              text: "Les fichiers téléversés sont décodés dans des limites fixes de pixels et encodés en AVIF avec le préréglage le plus lent, en échangeant du temps de traitement contre des fichiers plus petits. Les lieux issus des données EXIF peuvent être confirmés sur une carte avant publication, et deux tâches de traitement au plus s’exécutent simultanément.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Technologies",
      title: "Performances",
      lede: "Optimisation guidée par profil et BOLT sur le backend, mesurées avec un générateur de charge qui n’est pas le goulot d’étranglement.",
      sections: [
        {
          id: "approach",
          heading: "Démarche",
          blocks: [
            {
              kind: "paragraph",
              text: "Les builds optimisés sont entraînés sur une charge de travail représentative. Un binaire instrumenté enregistre des profils face à un environnement de test jetable doté de points de terminaison PostgreSQL, SMTP, S3, OpenID Connect et Minecraft synthétiques. LLVM recompile le serveur avec ces profils (PGO), puis BOLT réorganise la disposition du code du binaire final à partir d’une exécution échantillonnée. Toutes les variantes partagent la LTO, les pointeurs de trame et une bibliothèque standard recompilée.",
            },
          ],
        },
        {
          id: "correction",
          heading: "Le benchmark qui se mesurait lui-même",
          blocks: [
            {
              kind: "paragraph",
              text: "La première campagne générait la charge depuis Playwright et semblait montrer des régressions. Elles n’étaient pas réelles : les appels d’API Playwright sans portée parcourent les étapes de test précédentes pour trouver leur parent, si bien que le client ralentissait au fil de l’exécution. Face à un serveur HTTP trivial, le client seul est passé de 2 684 à 1 314 requêtes/s sur neuf échantillons.",
            },
            {
              kind: "paragraph",
              text: "La génération de charge est passée à oha. Playwright vérifie toujours la réponse de chaque route, mais en dehors de la fenêtre chronométrée.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Configuration",
          blocks: [
            {
              kind: "list",
              items: [
                "Serveur, fournisseurs de l’environnement de test et PostgreSQL épinglés sur quatre cœurs d’un AMD Ryzen 7 9800X3D ; le client de charge épinglé sur les quatre autres.",
                "HTTPS sur HTTP/1.1 avec un pool de 32 connexions, lecture complète des réponses et délais de deux secondes.",
                "Trois manches en rotation avec cinq échantillons par variante, chacun après 10 000 requêtes d’échauffement, à partir d’un instantané de base de données neuf.",
                "27 000 000 de requêtes chronométrées au total ; toutes ont renvoyé HTTP 200.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Résultats",
          blocks: [
            {
              kind: "bars",
              caption: "Nombre médian de requêtes par seconde sur 15 échantillons par variante.",
              groups: [
                {
                  label: "Lectures publiques sur dix routes",
                  items: [
                    { label: "Référence", value: 44874.12, display: "44 874" },
                    { label: "PGO", value: 52925.46, display: "52 925" },
                    { label: "PGO + BOLT", value: 53888.61, display: "53 889", emphasis: true },
                  ],
                },
                {
                  label: "Petit JSON de contrôle de santé",
                  items: [
                    { label: "Référence", value: 272837.41, display: "272 837" },
                    { label: "PGO", value: 308685.26, display: "308 685", emphasis: true },
                    { label: "PGO + BOLT", value: 306276.64, display: "306 277" },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "Latence de queue pour le mélange de dix routes.",
              columns: ["Variante", "p95 (ms)", "p99 (ms)"],
              numericColumns: [1, 2],
              rows: [
                ["Référence", "2,380", "2,967"],
                ["PGO", "2,020", "2,524"],
                ["PGO + BOLT", "1,987", "2,483"],
              ],
            },
            {
              kind: "paragraph",
              text: "PGO a augmenté le débit mixte de **17,9 %** et PGO avec BOLT de **20,1 %**, et les deux builds optimisés ont remporté chaque manche. BOLT a ajouté 1,8 % par rapport à PGO sur les lectures mixtes et s’est montré 0,8 % plus lent sur la route de santé triviale. Ce sont des résultats sur quatre cœurs ; la capacité en production dépend aussi du réseau, de l’ensemble de travail de la base de données et du trafic réel.",
            },
          ],
        },
        {
          id: "transport",
          heading: "Les petites corrections comptent aussi",
          blocks: [
            {
              kind: "paragraph",
              text: "L’accepteur Rustls par défaut d’axum-server ne définissait pas `TCP_NODELAY`, si bien que les réponses HTTP/2 découpées en trames d’en-tête et de données pouvaient attendre un accusé de réception différé. L’activer sur chaque socket accepté a modifié ainsi une réponse de tuile de 59 571 octets sur une connexion réutilisée.",
            },
            {
              kind: "table",
              columns: ["TCP_NODELAY", "Protocole", "Premier octet (médiane)", "Réception complète (médiane)"],
              numericColumns: [2, 3],
              rows: [
                ["Désactivé", "HTTP/2", "0,290 ms", "40,981 ms"],
                ["Activé", "HTTP/2", "0,113 ms", "0,152 ms"],
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
