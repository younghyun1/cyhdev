import { RepositorySourceMap } from "../../components/RepositorySourceMap";
import { pageStyles } from "../../styles/pageStyles";

export default function AboutBlog() {
  return (
    <main lang="fr-FR" class={pageStyles.page}>
      <section
        class={`${pageStyles.pageInnerNarrow} text-ink`}
      >
        <div class="border-l-4 border-line-strong pl-3 mb-8">
          <h1 class="text-2xl font-bold mb-1 tracking-tight">
            Technologies du blog
          </h1>
          <p class="text-xs text-ink-muted">
            Dernière mise à jour : 2026-08-30
            <br />
          </p>
        </div>

        <section class="space-y-8">
          {/* 0) Hébergement sur site */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              0) Machine hôte, système d’exploitation, système de fichiers et configuration réseau
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              Le site est hébergé sur un mini-serveur à mon domicile, derrière une connexion Xfinity filaire de 1 Gbit/s. L’{" "}
              <a
                href="https://store.minisforum.com/products/minisforum-um690l-slim"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                hôte
              </a>{" "}
              embarque un processeur mobile Ryzen 9 à huit cœurs, jusqu’à 4,9 GHz, 32 Go de mémoire à 6 400 MT/s et un SSD NVMe de 1 To. À environ 400 $, c’est une excellente affaire : sur AWS, un montant équivalent ne donnerait accès qu’à du matériel bien moins puissant pendant un an environ. Cela reflète mon goût pour l’auto-hébergement. La machine exécute le serveur intégré backend-frontend, la base PostgreSQL et un serveur Minecraft. Écrivez-moi si vous souhaitez jouer.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-1) Gentoo
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Ce n’est pas vraiment le système d’exploitation de prédilection pour un serveur d’entreprise ; dans un cadre professionnel, j’aurais choisi Debian Stable, un système de fichiers ext4 et un moteur de base de données classique. Mais j’aime bricoler, compiler moi-même les logiciels et créer des paquets optimisés pour l’architecture du processeur, ce que permet le formidable{" "}
              <a
                href="https://www.gentoo.org/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                Gentoo
              </a>{" "}
              projet.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-2) Utilisation de btrfs sur un hôte base de données, backend/frontend et Minecraft
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              <a
                href="https://en.wikipedia.org/wiki/Btrfs"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                btrfs
              </a>{" "}
              est un système de fichiers moderne qui offre instantanés, CoW, compression et bien d’autres fonctions. Il ne convient toutefois pas idéalement à un hôte de base de données, car CoW provoque une fragmentation importante. J’en ai limité les effets en excluant les répertoires de données PostgreSQL et Minecraft de CoW.{" "}
              <a
                href="https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                Un benchmark de systèmes de fichiers pour PostgreSQL
              </a>{" "}
              montre que btrfs n’offre pas de très bonnes performances dans sa configuration par défaut. Je soupçonne toutefois que la désactivation de CoW pourrait le mettre au niveau d’ext4 et de xfs. Un benchmark comparatif serait intéressant.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              0-3) Configuration des réseaux interne et externe
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              En souscrivant à l’offre Internet par câble de Xfinity à 2 Gbit/s, je n’avais pas réalisé que le modem-routeur fourni ne prend <em>pas</em> en charge l’Ethernet 2,5 Gbit/s. Je devrai donc me contenter de 1 Gbit/s, mais j’imagine mal que cela pose problème à mon petit site. Route 53 fournit le DNS de mon domaine.
              <br />
              <br />
              En interne, rien ne justifie vraiment un proxy inverse, la conteneurisation ou des outils de services distribués : il s’agit simplement d’un moteur PostgreSQL exécuté sur le système et d’un binaire Rust qui sert à la fois le frontend et l’API, reliés à la base par un socket UNIX, nettement plus rapide que{" "}
              <a
                href="https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                le simple recours à localhost, comme l’explique cette page.
              </a>{" "}
              Le débit peut presque doubler et éviter la pile réseau superflue réduit aussi la latence de moitié ; j’ai mesuré seulement 150 microsecondes entre la base et le serveur. Avec localhost, j’obtenais généralement environ 600 microsecondes. Ce n’est pas pertinent pour les architectures cloud d’entreprise, mais c’est intéressant. C’est classique, et c’est plus rapide ! PostgreSQL 18, publié en novembre 2025, a aussi introduit les E/S asynchrones sous la forme de{" "}
              <a
                href="https://pganalyze.com/blog/postgres-18-async-io"
                class={pageStyles.link}
                target="_blank"
                rel="noopener noreferrer"
              >
                la prise en charge d’io_uring
              </a>{" "}
              que j’ai activée. Elle accélère nettement les lectures.
            </p>
          </section>

          {/* 1) PostgreSQL */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              1) Données
            </h2>
            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-1) PostgreSQL 18
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Au cours de mes expériences en startup et en entreprise en Corée, j’ai constaté une culture qui considère MySQL ou MariaDB comme les seuls SGBDR qui valent la peine, sans que j’en comprenne encore la raison. D’anciens responsables m’ont dit que PostgreSQL n’était autrefois pas pris au sérieux, ce qui est étonnant : depuis une dizaine d’années, il a fortement progressé en performances, extensibilité, types de données et outils, et a probablement dépassé MySQL à bien des égards, notamment pour la prise en charge des UUID et l’encodage binaire des données JSON.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-2) Points clés du schéma (blog et authentification)
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Le schéma est volontairement simple, dans le bon sens du terme : utilisateurs, sessions en mémoire, jetons de vérification d’adresse e-mail et de réinitialisation de mot de passe, articles, commentaires, tables de votes, étiquettes, photos de profil et quelques tables géographiques et d’internationalisation. Le blog ne se résume pas à une simple table
              <code>posts</code> et à une prière. Les articles comportent des slugs, des résumés, des métadonnées JSONB, un état de publication, des compteurs dénormalisés et des associations d’étiquettes. Les commentaires sont imbriqués au moyen d’un identifiant de commentaire parent nullable. Les votes sont répartis dans des tables distinctes pour les articles et les commentaires, ce qui simplifie les requêtes et évite ensuite des conditions inutilement alambiquées.
              <br />
              <br />
              L’authentification suit la même approche pragmatique : la fiche utilisateur enregistre le pays et la langue, afin que le site ne se contente pas de demander une adresse e-mail puis de vous oublier. Les rôles et permissions évitent de s’enfermer dans des choix d’autorisation limitants. Les photos de profil ont leur propre table et sont versionnées, ce qui simplifie leur remplacement et évite de surcharger la fiche utilisateur fréquemment consultée avec des données sans rapport.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-3) UUIDv7
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Presque toutes les ressources visibles par les utilisateurs sont identifiées par des UUID, et ce n’est pas un hasard. Je préfère éviter d’exposer des identifiants séquentiels qui permettent de déduire le nombre de lignes ou d’énumérer les ressources comme en 2009. Les UUID ordonnés par le temps perturbent aussi moins les index que les UUIDv4 entièrement aléatoires, ce qui est utile dès que les écritures sont réelles. Ils sont uniques à l’échelle mondiale, difficiles à deviner et plus respectueux de la localité. Les insertions sont plus régulières, les arbres B subissent moins de remaniements et la base a moins de tâches d’entretien évitables.
            </p>

            <h3 class="mt-5 text-sm font-semibold text-ink">
              1-4) Schéma (chemin des requêtes et des données)
            </h3>
            <p class="mt-2 text-sm leading-6 text-ink-muted">
              Voici un schéma d’architecture d’entreprise très glamour, présenté en texte parce que je n’ai pas encore pris le temps de dessiner des boîtes :
              <br />
              <br />
              Requête du navigateur -&gt; routeur Axum -&gt; chaîne de middlewares (journalisation, recherche d’authentification/session, limitation de débit, vérification de la taille du corps) -&gt; gestionnaire -&gt; requête asynchrone Diesel ou consultation du cache mémoire -&gt; PostgreSQL via un socket UNIX -&gt; réponse DTO -&gt; réponse HTTPS compressée vers le navigateur.
              <br />
              <br />
              Les ressources statiques suivent un chemin encore plus court. L’application SolidJS compilée est intégrée directement au binaire Rust et servie avec négociation de contenu zstd/gzip si le navigateur le permet. Aucun processus Node en production, aucun hôte de fichiers statiques séparé et aucun saut supplémentaire entre l’arrivée d’une requête et l’envoi des octets.
            </p>
          </section>

          {/* 2) Backend */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              2) Serveur backend (Rust)
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              Le backend est un service Rust construit avec Axum, Tokio, Diesel et PostgreSQL. Il utilise mimalloc comme allocateur global : lorsqu’on auto-héberge sur une machine dotée de vrais cœurs, autant prendre un minimum au sérieux le comportement de l’allocateur. Le serveur termine directement TLS avec rustls, sert l’application monopage intégrée, expose des API JSON pour l’authentification, le blog, la photographie et l’internationalisation, et transmet les statistiques de l’hôte par WebSockets au tableau de bord. Le routeur comprend la compression des requêtes, un CORS actuellement permissif, une limitation de débit assez généreuse et une interface Swagger protégée par authentification en production.
              <br />
              <br />
              Côté application, l’architecture privilégie un processus unique doté d’un état serveur partagé, de sessions en mémoire, d’une synchronisation du cache au démarrage et de tâches de maintenance planifiées. L’authentification utilise des cookies sécurisés HTTP-only, la vérification des e-mails, des jetons de réinitialisation de mot de passe et des contrôles explicites de rôle pour les opérations superutilisateur. Le profil de latence est plus intéressant : le trafic vers la base reste sur un socket UNIX, la taille du pool de connexions est adaptée au nombre de cœurs physiques, les listes d’articles sont principalement lues depuis le cache et les réponses sont enrichies par lots au lieu d’accumuler de minuscules requêtes. Le système reste ainsi rapide là où cela compte : moins de sauts, de copies et d’allers-retours, et moins d’attente liée à des abstractions qui se félicitent elles-mêmes.
            </p>
          </section>

          {/* 3) Interface web */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              3) Interface web
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              Le frontend est une application monopage SolidJS côté client, construite avec Vite et TypeScript. Les routes sont chargées à la demande, l’état reste relativement simple et l’ensemble est compilé en ressources statiques intégrées au binaire Rust lors du déploiement. Aucun processus serveur JavaScript n’est donc à surveiller en production ; le serveur Rust distribue simplement les fichiers statiques aussi efficacement que possible.
              <br />
              <br />
              L’interface du blog allie pragmatisme et refus des éditeurs rudimentaires. La rédaction Markdown passe par Toast UI Editor, et les images collées ou téléversées transitent directement par l’API de téléversement photo, pour rendre la rédaction agréable. La recherche porte sur les titres et les étiquettes, la navigation entre les pages utilise des paramètres de requête, l’état d’authentification est suivi côté client mais vérifié côté serveur, et l’application lit les en-têtes de réponse pour afficher les informations de build du serveur. Les styles reposent sur Tailwind et un système partagé. Solid reste léger à l’exécution et limite les modifications du DOM, exactement ce que j’attends d’une couche d’interface dont le rôle principal est de ne pas gêner.
            </p>
          </section>

          {/* 4) Network & HTTPS */}
          <section class={pageStyles.cardPadded}>
            <h2 class="text-lg font-bold mb-2 text-ink">
              4) HTTPS, routage et garde-fous
            </h2>
            <p class="text-sm leading-6 text-ink-muted">
              HTTPS est géré directement par le serveur Rust avec rustls. Les requêtes HTTP sont redirigées vers HTTPS, les cookies sont marqués Secure et HTTP-only, et leur domaine en production est limité à celui du site. Les ressources statiques sont servies en zstd ou gzip lorsque le navigateur le prend en charge ; le routage de repli de l’application monopage permet aux liens profonds de fonctionner sans proxy inverse distinct.
              <br />
              <br />
              Les garde-fous ne sont pas particulièrement sophistiqués, mais ils sont bien présents : limites de taille du corps des requêtes pour les téléversements, journalisation par middleware, authentification sur les routes protégées, vérification des superutilisateurs sur les points de terminaison sensibles, limitation de débit pour décourager les abus et prise en charge des clés API pour les requêtes clientes. La forme du déploiement reste surtout assez simple pour être comprise : un binaire, une base, un hôte, TLS dans l’application et peu de place pour les latences mystérieuses ou la dérive de configuration. Cette architecture n’est pas à la mode, mais elle est rapide, observable et économe en ressources.
            </p>
          </section>

          <RepositorySourceMap />
        </section>
      </section>
    </main>
  );
}
