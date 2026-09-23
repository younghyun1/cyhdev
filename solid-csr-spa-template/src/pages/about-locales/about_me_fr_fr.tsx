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
    <main lang="fr-FR" class={`${pageStyles.page} about-page`}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">À propos</h1>
          <p class="text-xs text-ink-muted">
            Dernière mise à jour : septembre 2026
          </p>
        </div>

        <section class="mb-12">
          <h2 class="text-xl font-bold mb-4 border-b border-line pb-2">
            1. Présentation
          </h2>

          <div class="mb-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div>
              <h3 class="text-lg font-semibold">
                Young Hyun Chi / 지영현 / 池營賢 / 池营贤
              </h3>

              <p class="text-ink-muted">
                Ingénieur backend et infrastructure · Rust / fiabilité des plateformes / sécurité
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
                  alt="Portrait"
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
            <h3 class="font-bold mb-2">Contact</h3>
            <ul class="list-none space-y-2 text-sm">
              <li>
                <span class="w-20 inline-block font-medium text-ink-muted">
                  E-mail :
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
            Ingénieur backend installé dans le Colorado, je me spécialise dans les services Rust, les bases de données PostgreSQL et l’infrastructure cloud. Chez Soundpatrol, je travaille sur la cybersécurité, le DevOps, les services web et les outils de développement avec Rust, Python, PostgreSQL, Kubernetes et GCP.
          </p>
          <p class="mb-4 leading-relaxed">
            Parmi mes projets précédents figurent un jumeau numérique déployé sur AWS pour Samsung C&amp;T et intégrant des milliers de capteurs environnementaux, une plateforme d’analyse de chaînes YouTube exploitant les transcriptions, les commentaires et les modèles de langage, ainsi que des backends et pipelines de données pour les applications Hyundai, Kia et Genesis et l’application de fans d’un groupe de K-pop.
          </p>
          <p class="mb-4 leading-relaxed">
            Je crois qu’il faut concevoir des services qui tirent pleinement parti du matériel moderne et des langages de programmation. J’aime approfondir la concurrence, les requêtes de base de données, la mémoire et le déploiement, plutôt que de considérer les ressources cloud comme un substitut illimité à l’ingénierie. J’aime aussi héberger mes propres services, créer des outils en Rust et aider mes collègues à apprendre ce langage.
          </p>
          <p class="mb-4 leading-relaxed">
            Je suis fier d’avoir travaillé dans différents domaines avec des collègues aux cultures et parcours variés. De 2016 à 2017, j’ai servi pendant 21 mois près de la frontière nord-coréenne au sein de la 2e division d’infanterie de l’armée américaine, aux côtés de militaires américains et coréens. À l’université, j’ai travaillé dans le journalisme, la photographie et la traduction, ainsi que de nuit dans l’entreposage et la logistique. Depuis 2023, ma carrière se concentre sur l’ingénierie backend, les pipelines de données et l’infrastructure qui les fait fonctionner.
          </p>
        </section>

        {/* 2) Professional Career */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            2. Carrière en développement
          </h2>

          <div class="space-y-8">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Soundpatrol</h3>
                <span class="text-sm font-mono text-ink-muted">États-Unis · Télétravail | mai 2026 - aujourd’hui</span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">Ingénieur Rust</p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>Développement de services web et d’outils internes en Rust et Python.</li>
                <li>Maintenance de l’infrastructure cloud, des déploiements Kubernetes et des pipelines CI/CD.</li>
                <li>Travail sur la supervision, le contrôle des accès, la gestion des identifiants et d’autres mesures de cybersécurité.</li>
              </ul>
            </div>

            {/* GenesisNest */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">GenesisNest</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Seongnam, Corée du Sud | janv. 2025 - juil. 2025
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Ingénieur logiciel
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Développement d’outils de gestion d’infrastructure pour les services internes et l’application officielle d’un groupe de K-pop. Découverte de l’écosystème Spring Boot.
                </li>
                <li>
                  Mission contractuelle sur les backends des applications officielles Hyundai, Kia et Genesis de Hyundai Motor Company, au sein d’une vaste base de code Java.
                </li>
                <li>
                  Mise en œuvre de nouvelles API et corrections de bogues, en coordination avec les fournisseurs de services utilisés par des dizaines de millions de personnes en Asie et en Europe.
                </li>
                <li>
                  Mise en place de pipelines de données pour leur projet d’internationalisation et rédaction de scripts Rust et Python automatisant l’intégration des contenus traduits dans la base Hyundai. Collaboration avec les administrateurs de bases de données de l’entreprise sur de très grands jeux de données reliés aux systèmes des usines.
                </li>
              </ul>
            </div>

            {/* pampam */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">pampam Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Séoul, Corée du Sud | nov. 2024 - déc. 2024
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Ingénieur logiciel contractuel
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Conception du schéma PostgreSQL et du backend Rust d’une plateforme d’analyse YouTube fondée sur l’IA, et pilotage de son déploiement sur AWS et GCP.
                </li>
                <li>
                  Pilotage d’une collecte de données à grande échelle via l’API YouTube (dizaines de millions de commentaires et d’utilisateurs).
                </li>
                <li>
                  Utilisation de grands modèles de langage et de modèles libres hébergés sur le serveur pour produire des analyses et des synthèses.
                </li>
              </ul>
            </div>

            {/* EAN Technology */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">EAN Technology Co. Ltd</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Séoul, Corée du Sud | août 2023 - août 2024
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Ingénieur backend principal
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>Projet de jumeau numérique Samsung C&amp;T :</b> remplacement d’une architecture de microservices défaillante par un monolithe Axum (Rust) fortement optimisé. Réduction de la facture cloud mensuelle d’environ 5 000 $ à 150 $.
                </li>
                <li>
                  Contribution d’environ 30 000 lignes de code (près de 75 % du total) et mise en œuvre de 80 points de terminaison à logique métier complexe.
                </li>
                <li>
                  Aucun arrêt en cours d’exécution et une couverture de gestion des erreurs d’environ 90 % pendant neuf mois de fonctionnement en production.
                </li>
                <li>
                  Intégration de milliers de capteurs physiques en temps réel (BACnet, Modbus) et de serveurs sur site au siège de Samsung C&amp;T.
                </li>
                <li>
                  Optimisation des requêtes et du schéma PostgreSQL : la latence P99 est passée de plusieurs secondes à quelques dizaines de millisecondes sur certains cas, sur un matériel limité (2 cœurs, 4 Go de RAM).
                </li>
                <li>
                  Mise en œuvre de CI/CD avec GitHub Runners, AWS CodeDeploy et Docker. Utilisation de MUSL, de l’édition de liens statique et de builds à plusieurs étapes avec des images « scratch » pour produire des images Docker d’environ 20 Mo, plus sûres et rapides à démarrer.
                </li>
                <li>
                  Accompagnement de collègues dans leur apprentissage de Rust, leur permettant de migrer depuis Node.js et Java en deux mois.
                </li>
              </ul>
            </div>

            {/* Artifyc */}
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Artifyc Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Austin, Texas (télétravail) | août 2022 - mars 2023
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Stagiaire en ingénierie logicielle
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Création d’un bot Discord en Python utilisant l’API Reddit pour collecter et publier des demandes de commandes artistiques.
                </li>
                <li>
                  Développement sur AWS Lambda d’une logique backend sans serveur pour la conversion monétaire internationale et le calcul des frais supplémentaires.
                </li>
                <li>
                  Participation aux modifications du frontend React et à l’intégration des ressources.
                </li>
              </ul>
            </div>
          </div>
        </section>


        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">Projets sélectionnés</h2>
          <ul class="list-disc list-inside text-sm space-y-3 text-ink">
            <li><a href="https://github.com/younghyun1/sillok" class={pageStyles.link}>Sillok</a> : interface en ligne de commande Rust pour organiser les journaux de travail, avec historique en ajout seul, projections indexées Turso/SQLite, corrections et synchronisation entre machines via Git.</li>
            <li><a href="https://github.com/younghyun1/eu5-location-filter" class={pageStyles.link}>EU5 Location DB</a> : application de bureau et WebAssembly en Rust et Slint, avec données cartographiques compressées intégrées, index précalculés, filtres interrogeables et tableaux redimensionnables.</li>
            <li><a href="https://github.com/younghyun1/cyhdev" class={pageStyles.link}>cyhdev.com</a> : ce site en Rust/Axum, PostgreSQL et SolidJS, avec publication, photographie, discussions, chat et démonstrations dans le navigateur.</li>
            <li><a href="https://github.com/younghyun1/oohid" class={pageStyles.link}>oohid</a> : générateur d’UUID en Rust avec format configurable, sortie dans un fichier et vérification facultative des doublons.</li>
          </ul>
        </section>

        {/* 3) Non-IT Career */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            3. Expériences hors informatique
          </h2>

          <div class="space-y-6">
            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">
                  Institut coréen des sciences et technologies maritimes
                </h3>
                <span class="text-sm font-mono text-ink-muted">
                  Séoul, Corée du Sud | juin-juil. 2023, août 2024
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Traducteur et interprète
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Interprétation (anglais, coréen, norvégien) lors de conférences de haut niveau réunissant notamment l’ambassadeur de Norvège et le président du groupe Dongwon.
                </li>
                <li>
                  Gestion d’informations très sensibles concernant des transactions publiques et commerciales ainsi que des discussions sur la recherche et le développement maritimes.
                </li>
                <li>
                  Préparation du personnel et du directeur du KIMST aux événements du département de l’Énergie des États-Unis.
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
                    Armée américaine
                  </a>{" "}
                  (
                  <a
                    href="https://en.wikipedia.org/wiki/2nd_Infantry_Division_(United_States)"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    2e division d’infanterie
                  </a>
                  ) /{" "}
                  <a
                    href="https://en.wikipedia.org/wiki/Republic_of_Korea_Army"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    Armée de la République de Corée
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
                  , Corée du Sud | mars 2016 - déc. 2017
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
                | Instructeur | Sergent
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Service comme soldat de liaison et instructeur. Formation et accueil administratif d’environ 6 000 militaires américains et coréens.
                </li>
                <li>
                  Enseignement de cours de condition physique, de culture, d’histoire et de prévention des risques.
                </li>
                <li>
                  Maintien de la capacité opérationnelle pendant la{" "}
                  <a
                    href="https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis"
                    class={pageStyles.link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    crise de 2017
                  </a>
                  ; direction d’une équipe de treize personnes.
                </li>
              </ul>
            </div>

            <div>
              <div class="flex flex-wrap justify-between items-baseline mb-1">
                <h3 class="text-lg font-bold">Coupang Inc</h3>
                <span class="text-sm font-mono text-ink-muted">
                  Corée du Sud | 2020 - 2022
                </span>
              </div>
              <p class="text-sm font-medium mb-2 text-ink-muted">
                Agent de chargement
              </p>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  Travail physique intensif dans des entrepôts logistiques pendant des quarts de nuit.
                </li>
              </ul>
            </div>
          </div>
        </section>

        {/* 4) Academics */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            4. Formation
          </h2>

          <div class="mb-6">
            <div class="flex flex-wrap justify-between items-baseline mb-2">
              <h3 class="text-lg font-bold">Sungkyunkwan University</h3>
              <span class="text-sm font-mono text-ink-muted">
                Suwon, Corée du Sud | mars 2015 - août 2023
              </span>
            </div>
            <p class="text-sm font-medium mb-2 text-ink-muted">
              Licence en génie logiciel
            </p>
            <div class="space-y-2 text-sm text-ink">
              <p>
                Formation complète alliant informatique, génie logiciel web et embarqué, et génie électronique. Études axées sur la logique, les langages de bas niveau, les systèmes d’exploitation et les réseaux, ainsi que sur les pratiques du génie logiciel.
              </p>
              <p>
                L’université Sungkyunkwan est la plus ancienne université de Corée. Fondée en 1398, elle servait d’académie confucéenne destinée à former les fonctionnaires de l’État sous le royaume de Joseon. Depuis la libération, elle compte parmi les principales institutions universitaires et historiques du pays et figure régulièrement parmi les 100 meilleures universités mondiales depuis mon arrivée.
              </p>
            </div>
          </div>

          <div>
            <h4 class="font-bold mb-2 text-md">Publications</h4>
            <ul class="list-disc list-inside text-sm space-y-2 text-ink">
              <li>
                <b>
                  Analyse des émotions dans les communautés de réseaux sociaux à l’aide d’un modèle de TAL en coréen
                </b>{" "}
                (IMCOM 2023, IEEE Xplore).
                <br />
                <span class="ml-5 block text-xs text-ink-muted mt-1">
                  Utilisation de KoBERT pour analyser les sentiments dans des données de réseaux sociaux au début de la pandémie de COVID-19. Projet récompensé par le prix de bronze des projets de fin d’études.
                </span>
              </li>
            </ul>
          </div>
        </section>

        {/* 5) Volunteer work and interests */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            5. Bénévolat et centres d’intérêt
          </h2>

          <div class="grid md:grid-cols-2 gap-8">
            <div>
              <h3 class="font-bold mb-3 text-lg">Projets personnels</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>
                  <b>oohid :</b> générateur d’UUIDv4 en ligne de commande écrit en Rust, environ trois fois plus rapide que libuuid. Vérification des doublons et formatage Python/JSON.
                </li>
                <li>
                  <b>impulsr :</b> outil de transcription YouTube et de collecte de commentaires pour créer des synthèses marketing.
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-3 text-lg">Bénévolat</h3>
              <ul class="list-disc list-inside text-sm space-y-2 text-ink">
                <li>Enseignement de l’anglais dans un centre communautaire.</li>
                <li>Bénévolat dans une friperie gérée par des personnes en situation de handicap.</li>
              </ul>
            </div>
          </div>
        </section>

        {/* 6) Hobbies */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            6. Loisirs
          </h2>

          <div class="space-y-4">
            <div>
              <h3 class="font-bold inline mr-2">Photographie</h3>
              <span class="text-sm text-ink-muted">
                Photographe amateur de paysages et de portraits depuis 2010. Environ 30 000 photos archivées.
              </span>
              <div class="mt-2">
                <a
                  href="/photographs"
                  class={`inline-flex items-center ${pageStyles.link} font-medium`}
                >
                  Voir le portfolio photo →
                </a>
              </div>
            </div>

            <div>
              <h3 class="font-bold inline mr-2">Journalisme</h3>
              <p class="text-sm text-ink mt-1">
                Ancien journaliste étudiant au Sungkyun Times.
              </p>
            </div>
          </div>
        </section>

        {/* 7) Qualifications and Other Skills */}
        <section class="mb-12">
          <h2 class="text-xl font-bold mb-6 border-b border-line pb-2">
            7. Qualifications et autres compétences
          </h2>

          <div class="grid md:grid-cols-2 gap-8 mb-8">
            <div>
              <h3 class="font-bold mb-2">Qualifications et distinctions</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>
                  <b>TOEFL :</b> 117/120 (CECR C2)
                </li>
                <li>
                  <b>IELTS :</b> 8.5/9.0 (CECR C2)
                </li>
                <li>
                  <b>7e place :</b> Hackathon de conception et d’idées de fin d’études 2021
                </li>
                <li>
                  <b>3e place :</b> Hackathon Fintech 2020 (SNUST)
                </li>
              </ul>
            </div>

            <div>
              <h3 class="font-bold mb-2">Langues</h3>
              <ul class="list-disc list-inside text-sm space-y-1 text-ink">
                <li>Coréen (langue maternelle)</li>
                <li>Anglais (C2)</li>
                <li>Notions de français, d’allemand, d’espagnol et de mandarin</li>
              </ul>
            </div>
          </div>
        </section>
      </section>
    </main>
  );
}
