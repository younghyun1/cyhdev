import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "Kasım 2026",
  chrome: {
    overview: "Genel bakış",
    updated: "Son güncelleme",
    onThisPage: "Bu sayfada",
    sectionNavigation: "Teknoloji yığını sayfaları",
    previous: "Önceki",
    next: "Sonraki",
    readingTime: "{minutes} dk okuma",
  },
  pages: {
    overview: {
      eyebrow: "Bu site hakkında",
      title: "Blogun teknoloji yığını",
      lede: "Tek bir Rust ikili dosyası, tek bir PostgreSQL veritabanı, tek bir küçük makine. Bu sayfalar sitenin nasıl kurulduğunu, barındırıldığını, güvenceye alındığını ve ölçüldüğünü; neden bu biçimde olduğunu anlatır.",
      sections: [
        {
          id: "at-a-glance",
          heading: "Bir bakışta",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53.889 req/s",
                  label: "Dört sunucu çekirdeğinde PGO ve BOLT ile karma herkese açık okuma veriminin medyanı",
                },
                {
                  value: "~150 µs",
                  label: "UNIX soketi üzerinden gözlenen veritabanı gidiş-dönüş süresi; localhost TCP üzerinden yaklaşık 600 µs",
                },
                {
                  value: "0",
                  label: "Üretimde çalışan Node.js süreci; frontend sunucu ikili dosyasına gömülüdür",
                },
                {
                  value: "27",
                  count: 27,
                  label: "Arayüz dili; her biri yalnızca seçildiğinde indirilir",
                },
                {
                  value: "4.096",
                  label: "TLS’den önce kabul edilen açık bağlantı; adres veya IPv6 /64 başına en fazla 64",
                },
                {
                  value: "1",
                  count: 1,
                  label: "TLS’yi, API’yi, WebSocket’leri, WebRTC medyasını ve gömülü frontend’i sunan süreç",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "İstek yolu",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Tarayıcı",
                  detail: "TLS üzerinden HTTP/2 veya HTTP/1.1. Frontend, parmak izli ve değişmez URL’ler altında önbelleğe alınır.",
                },
                {
                  label: "Kabul",
                  detail: "TCP bağlantıları TLS’den önce kabul edilir: toplamda 4.096, adres veya IPv6 /64 başına 64.",
                },
                {
                  label: "rustls",
                  detail: "TLS, her sokette `TCP_NODELAY` açık olacak şekilde Rust sürecinin içinde sonlandırılır.",
                },
                {
                  label: "Axum",
                  detail: "Güvenlik başlıkları, oturumlar, hız sınırları, sıkıştırma ve gövde sınırları; ardından bir özellik işleyicisi.",
                },
                {
                  label: "Önbellek · Diesel",
                  detail: "Sınırlı bellek içi önbellekler sık okumaları yanıtlar; geri kalan her şey asenkron Diesel üzerinden gider.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "Aynı makinede, UNIX soketi üzerinden ve io_uring asenkron G/Ç ile.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Bölümler",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Altyapı",
                  href: "/about-blog/infrastructure",
                  meta: "Sunucu · Gentoo · ağ",
                  text: "Makine, işletim sistemi ve dosya sistemi, ağ ve neden ters proxy bulunmadığı.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "Servis yığını, veri modeli, çalışma zamanı sınırları, güvenlik başlıkları ve API sözleşmeleri.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "Uygulama kabuğu, yükleme, yerelleştirme, tasarım sistemi ve hareket.",
                },
                {
                  title: "Alt sistemler",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "Süreç içi bir SFU ile canlı sohbet, Minecraft gezgini, Geo-IP sorguları ve WebAssembly.",
                },
                {
                  title: "Performans",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Optimize edilmiş derlemeler, nasıl ölçüldükleri ve sonuçlar.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Değişiklikler nasıl denetlenir",
          blocks: [
            {
              kind: "list",
              items: [
                "Uygulama aşaması kapısı olarak yerel ve WebAssembly hedeflerinde Clippy.",
                "Biçimlendirme, yerel birim testleri, OpenAPI sözleşme kayması ile frontend tür denetimi, lint, birim testleri ve derlemesi için yerel cargo xtask denetimleri.",
                "Chromium ve WebKit üzerinde Playwright; axe erişilebilirlik denetimleri ve 320 CSS pikseline kadar mobil yeniden akış denetimleri.",
                "Atılabilir bir PostgreSQL örneğine karşı entegrasyon ve geçiş geri alma testleri, ayrıca yerel bir gitleaks gizli anahtar taraması.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Teknoloji yığını",
      title: "Altyapı",
      lede: "Evde küçük bir makine, kaynak tabanlı bir Linux dağıtımı ve trafiğin ihtiyaç duymadığı hiçbir katman yok.",
      sections: [
        {
          id: "host",
          heading: "Sunucu",
          blocks: [
            {
              kind: "paragraph",
              text: "Site, evimdeki bir [mini sunucuda](https://store.minisforum.com/products/minisforum-um690l-slim) çalışıyor: 4,9 GHz’e kadar yükselen sekiz çekirdekli bir Ryzen 9 mobil işlemci, 32 GB RAM ve 1 TB’lık bir NVMe SSD. Yaklaşık 400 dolarlık maliyetiyle, belirgin biçimde daha zayıf bulut donanımının bir yıllık ücretinden daha ucuz; aynı zamanda kendi sunucumda barındırmaya olan hevesimi yansıtıyor.",
            },
            {
              kind: "paragraph",
              text: "Birleşik backend ve frontend sunucusunu, PostgreSQL’i ve bir Minecraft sunucusunu çalıştırıyor. Oynamak isterseniz bana e-posta gönderin.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo ve btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) bir sunucu için bariz bir seçim değil; iş yerinde ext4 üzerinde Debian stable seçerdim. Burada paketleri tam olarak işlemciye göre optimizasyonlarla kendim derlemekten keyif alıyorum.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) anlık görüntüler, sıkıştırma ve yazarken kopyalama (copy-on-write) getirir, ancak yazarken kopyalama veritabanı dosyalarını ciddi biçimde parçalar. Bu yüzden PostgreSQL ve Minecraft veri dizinleri bunun dışında tutulur. Bir [dosya sistemi kıyaslaması](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) varsayılan btrfs’in ext4 ve XFS’in gerisinde kaldığını gösteriyor; yazarken kopyalamayı kapatmanın bu farkın büyük bölümünü kapattığını tahmin ediyorum, bu da ilginç bir ölçüm olurdu.",
            },
          ],
        },
        {
          id: "network",
          heading: "Ağ",
          blocks: [
            {
              kind: "paragraph",
              text: "Trafik, kablolu 1 Gbps’lik bir Xfinity bağlantısı üzerinden geliyor; sağlayıcının modemi 2,5 Gb Ethernet desteklemiyor, bu site bunu fark etmeyecek. DNS’i Route 53 barındırıyor.",
            },
            {
              kind: "paragraph",
              text: "Makinenin içinde ters proxy, konteyner çalışma zamanı veya servis ağı (service mesh) yok. PostgreSQL doğrudan işletim sisteminde çalışır ve tek bir Rust ikili dosyası hem frontend’i hem API’yi sunar. Düz HTTP istekleri aynı süreç tarafından HTTPS’ye yönlendirilir.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "UNIX soketleri",
          blocks: [
            {
              kind: "paragraph",
              text: "Sunucu, PostgreSQL’e localhost TCP yerine bir UNIX soketi üzerinden ulaşır; bu, ağ yığınını tamamen atlar. [Kıyaslamalar](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) işlem hızının neredeyse iki katına çıktığını gösteriyor; burada gidiş-dönüşleri **150 µs** kadar düşük gözlemledim, localhost üzerinden ise yaklaşık **600 µs**. Eski moda ve çoğu bulut dağıtımı için önemsiz, ama daha hızlı.",
            },
            {
              kind: "paragraph",
              text: "Eylül 2025’te yayımlanan PostgreSQL 18, [asenkron G/Ç](https://pganalyze.com/blog/postgres-18-async-io) ekledi ve burada io_uring etkin; bu, okumaları belirgin biçimde hızlandırıyor.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Derleme ve dağıtım",
          blocks: [
            {
              kind: "list",
              items: [
                "Frontend derlemesi, önceden sıkıştırılmış zstd ve gzip kopyalarıyla birlikte backend ikili dosyasına gömülür; bu yüzden üretimde Node.js süreci çalışmaz.",
                "Optimize edilmiş derlemeler profil güdümlü optimizasyonla eğitilebilir ve BOLT ile bağlama sonrası optimize edilebilir; bkz. [Performans](/about-blog/performance).",
                "Kimlik bilgileri, sertifikalar ve çalışma zamanı verileri Git dışında kalır; değişiklikler makineden çıkmadan önce bir gitleaks taraması ağacı denetler.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Teknoloji yığını",
      title: "Backend",
      lede: "Axum ve Tokio üzerinde Rust, asenkron Diesel ile PostgreSQL ve bir istemcinin açık tutabileceği her şey için açık bir sınır.",
      sections: [
        {
          id: "stack",
          heading: "Yığın",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "Çalışma zamanı", items: ["Axum", "Tokio", "hyper", "axum-server", "rustls", "mimalloc"] },
                { label: "Veri", items: ["PostgreSQL 18", "Diesel", "diesel-async", "tantivy", "bitcode", "zstd"] },
                { label: "Entegrasyonlar", items: ["openidconnect", "lettre", "Amazon S3", "webrtc-rs", "image + AVIF"] },
                { label: "Sözleşmeler", items: ["utoipa", "OpenAPI", "Swagger UI", "Generated TypeScript"] },
              ],
            },
            {
              kind: "paragraph",
              text: "Servis, paylaşılan duruma sahip tek bir süreçtir: bellek içi oturumlar, açılışta önbellek eşitleme ve zamanlanmış bakım işleri. mimalloc genel ayırıcıdır; bir makinenin gerçek çekirdekleri varsa ayırıcı davranışı ciddiye almaya değer.",
            },
          ],
        },
        {
          id: "data",
          heading: "Veri modeli",
          blocks: [
            {
              kind: "paragraph",
              text: "Gördüğüm Koreli girişim ve kurumsal işlerde MySQL ve MariaDB tek ciddi ilişkisel veritabanları olarak görülüyordu. PostgreSQL, yerel UUID’lerden ikili JSON’a kadar önemsediğim alanlarda onları çoktan geride bıraktı.",
            },
            {
              kind: "list",
              items: [
                "**Yazılar** slug’lar, özetler, JSONB meta verileri, yayın durumu, denormalize sayaçlar ve etiket eşlemeleri taşır. Yorumlar boş olabilen bir üst öğe üzerinden iç içe geçer; yazı ve yorum oylarının ayrı tabloları vardır.",
                "**Hesaplar** ülkeyi ve dili, rol ve izin tablolarını ve sürümlü profil fotoğraflarını sık kullanılan kullanıcı satırında değil, kendi tablosunda tutar.",
                "**Anahtarlar** zamana göre sıralı UUIDv7 değerleridir: sıralı kimlikler gibi tahmin edilebilir değildir ve B-tree yerelliği açısından rastgele UUIDv4’ten çok daha uygundur.",
                "**Arama**, yazı başlıkları ve etiketleri üzerinde diske kalıcı yazılan bir tantivy indeksi kullanır; indeks PostgreSQL ile tutarlı tutulur ve yetkili kaynak PostgreSQL olarak kalır.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Çalışma zamanı sınırları",
          blocks: [
            {
              kind: "list",
              items: [
                "Bağlantılar TLS’den önce paylaşılan bir sınırlayıcıyla kabul edilir: 4.096 açık bağlantı ve IPv4 adresi veya IPv6 /64 başına 64. Reddedilen soketler el sıkışma olmadan kapanır.",
                "HTTP/1 istek başlıkları 20 saniye içinde gelmelidir; HTTP/2 bağlantılarına her 30 saniyede bir ping gönderilir ve 128 eşzamanlı akışa izin verilir.",
                "Her önbelleğin sabit bir sınırı ve üzerinden okuyabildiği bir yetkili kaynağı vardır; hiçbiri istemcinin denetlediği anahtarlarla büyümez. Örneğin WebAssembly paket önbelleği en fazla 256 MiB tutar.",
                "Görsel kodlama gibi engelleyici işler Tokio çalışanları yerine sınırlı çalışan havuzlarında yürür.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Güvenlik başlıkları",
          blocks: [
            {
              kind: "paragraph",
              text: "Content-Security-Policy açılışta bir kez oluşturulur. Satır içi betiklere yalnızca ikili dosyaya derlenmiş `index.html` dosyasından hesaplanan SHA-256 özetleriyle izin verilir; böylece boyama öncesi tema betiği politikadan sapamaz. Ana origin hiçbir zaman WebAssembly örneklemez: demolar ve EU5 uygulaması kendi politikaları altında korumalı çerçevelerde çalışır.",
            },
            {
              kind: "paragraph",
              text: "Oturum çerezleri `Secure` ve `HttpOnly` olarak işaretlenir, süper kullanıcı işlemleri rolleri sunucuda denetler, hesaplar OpenID Connect kimlikleri bağlayabilir ve Swagger UI üretimde kimlik doğrulaması gerektirir.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Sözleşmeler ve varlıklar",
          blocks: [
            {
              kind: "list",
              items: [
                "İşleyiciler OpenAPI açıklamaları taşır, frontend’in TypeScript istemcileri bunlardan üretilir ve üretilen dosyalar kaydığında cargo xtask openapi başarısız olur.",
                "Vite’in manifest dosyası varlıklarla birlikte gömülür. Yalnızca manifest’te listelenen, parmak izli dosyalar bir yıllık `immutable` önbellek politikası alır; HTML her zaman yeniden doğrulanır.",
                "Eksik varlık yolları uygulama kabuğu yerine `no-store` ile 404 döndürür; böylece eski bir sayfa bir betik URL’si altında HTML’i önbelleğe alamaz.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Teknoloji yığını",
      title: "Frontend",
      lede: "İstemci tarafında işlenen, statik varlıklara derlenip sunucu ikili dosyasına gömülen bir SolidJS 2 uygulaması.",
      sections: [
        {
          id: "stack",
          heading: "Yığın",
          blocks: [
            {
              kind: "tags",
              groups: [
                { label: "Uygulama", items: ["SolidJS 2", "Solid Router 2", "TypeScript", "Vite 8"] },
                { label: "Stil", items: ["Tailwind CSS 4", "IBM Plex Sans", "IBM Plex Mono", "OKLab"] },
                { label: "Bileşenler", items: ["Toast UI Editor", "Leaflet", "Chart.js", "highlight.js", "DOMPurify"] },
                { label: "Test", items: ["Vitest", "Playwright", "axe-core"] },
              ],
            },
            {
              kind: "paragraph",
              text: "Solid’in ince taneli sinyalleri yalnızca değişen değerlere bağlı DOM düğümlerini günceller ve 2.0 sürüm adayları, asenkron okumaların en yakın yükleme sınırına kadar askıya alınmasına izin verir. Yönlendirici her sayfayı tembel yükler ve üretimde JavaScript sunucusu yoktur: Rust ikili dosyası statik dosyaları dağıtır.",
            },
          ],
        },
        {
          id: "loading",
          heading: "Yükleme",
          blocks: [
            {
              kind: "list",
              items: [
                "Sayfalar, ana sayfanın sohbet paneli ve kontur gölgelendiricisi ayrı parçalardır; derleme, ilk yüklenenin gzip boyutunu raporlar.",
                "İlk parçada yalnızca İngilizce arayüz metni bulunur. Diğer diller ve her dilin Hakkında sayfalarının içeriği, seçildiğinde yüklenen ayrı parçalardır.",
                "Kısıtlanmış mobil performans testleri LCP için 2,5 sn veya altını, etkileşimler için 200 ms veya altını ve CLS için 0,1 veya altını hedefler.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Yerelleştirme",
          blocks: [
            {
              kind: "paragraph",
              text: "Arayüz, Arapça, Farsça ve Urduca için sağdan sola düzenler dahil 27 dili destekler. Yaklaşık 830 anahtarlık bir katalog backend’in kayıt defterine karşı doğrulanır ve yer tutucuların çeviride korunması gerekir. Hakkında sayfalarının içeriği türlendirilmiş veri modülleridir; dolayısıyla bir çeviri, İngilizcede bulunan bir bölümü sessizce atlayamaz.",
            },
          ],
        },
        {
          id: "design",
          heading: "Tasarım sistemi",
          blocks: [
            {
              kind: "paragraph",
              text: "Görünüm sıcak ve teknik: krem rengi kâğıt ya da saf siyah, kıl inceliğinde kenarlıklar, teknik olan her şey için tek aralıklı yazı tipi ve aynı anda tek bir vurgu rengi. Renkler anlamsal CSS değişkenleridir; bu yüzden bileşenlerin koyu mod varyantlarına hiç ihtiyacı olmaz. Vurgu ön ayarları sayfa arka planlarına karşı kontrast açısından test edilir, ziyaretçinin kendi RGB rengi okunabilir kalması için her modda ayarlanır ve açık veya koyu mod ile vurgu ilk boyamadan önce uygulanır.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Hareket",
          blocks: [
            {
              kind: "list",
              items: [
                "Bölüm açılışları ve okuma ilerleme çubuğu CSS kaydırma güdümlü animasyonlar kullanır; bunlar desteklenen yerlerde ana iş parçacığı dışında çalışır.",
                "İstatistikler kayıtlı bir `@property` tam sayısı üzerinden sayarak artar; son değer, yardımcı teknolojiler için işaretlemede bulunur.",
                "Tema anahtarı, View Transitions API ile düğmeden başlayarak genişler.",
                "Ana sayfanın kontur alanı küçük bir WebGL gölgelendiricisidir; sayfa boşta kaldıktan sonra yüklenir, yalnızca yetenekli masaüstü tarayıcılarında çalışır, kare hızını sınırlar ve ekran dışındayken durur.",
                "Bir site ayarı bunların hepsini kapatır. Animasyonlar varsayılan olarak açıktır ve işletim sisteminin azaltılmış hareket ayarını izlemez.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Teknoloji yığını",
      title: "Alt sistemler",
      lede: "Sitenin, bir blogun kesinlikle ihtiyaç duyduğundan daha ilginç olan bölümleri.",
      sections: [
        {
          id: "live-chat",
          heading: "Canlı sohbet ve aramalar",
          blocks: [
            {
              kind: "paragraph",
              text: "Sohbet; kompakt bir ikili codec, moderasyon ve sınırlı geçmişle bir WebSocket üzerinden çalışır. Sesli ve görüntülü aramalar, Axum sürecinin içinde webrtc-rs üzerinde çalışan bir Selective Forwarding Unit (SFU) üzerinden geçer: her tarayıcı tek bir eş bağlantısı tutar ve SFU kodlanmış medyayı yeniden kodlamadan, yayın başına sınırlı kanallar ve bir katılımcı üst sınırıyla iletir. [Canlı sohbeti aç](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Minecraft arazi gezgini",
          blocks: [
            {
              kind: "paragraph",
              text: "Bir Paper eklentisi, sınırlı dünya sorguları için aynı kullanıcıya ait bir UNIX soketi sunar; aynı anda tek işlem ve sıkı çerçeve boyutu sınırları uygulanır. [Gezgin](/minecraft), üretilmiş haritanın JavaScript’ini çalıştırmak yerine squaremap karolarını kendi Leaflet istemcisiyle çizer.",
            },
            {
              kind: "paragraph",
              text: "Biyom katmanları, gzip ile sıkıştırılmış olarak sunulan kompakt bir ikili karo biçimi olan **CYBM**’yi ya da özel `cyBM` parçasında tam biyom kimliklerini taşıyan bir PNG’yi kullanır; böylece üzerine gelindiğinde piksel renklerinden tahmin yürütülmez. Keşfedilmemiş araziler için seed tabanlı tahminler, Pumpkin üzerine kurulu ayrı bir dünya üretimi çalışanından gelir.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Geo-IP sorguları",
          blocks: [
            {
              kind: "paragraph",
              text: "[Ziyaretçi panosu](/visitor-board) ve [IP sorgusu](/geo-ip-db) adresleri harici bir API çağırmadan konumlandırır. Bir ön işleme aracı IP2Location LITE CSV dosyalarını başlangıç aralığına göre anahtarlanmış `BTreeMap` yapılarına dönüştürür, bitcode ile kodlar ve zstd ile 22. seviyede sıkıştırır.",
            },
            {
              kind: "table",
              columns: ["Veri kümesi", "CSV", "Paket"],
              numericColumns: [1, 2],
              rows: [
                ["IPv4", "312,5 MB", "24,9 MB"],
                ["IPv6", "794,2 MB", "42,4 MB"],
              ],
            },
            {
              kind: "paragraph",
              text: "Açılışta sunucu, tepe belleği sınırlamak için paketleri tek tek çözer ve yinelenen yer adlarını paylaşımlı olarak saklar. Bir sorgu, başlangıcı adresin altında veya adresin kendisinde olan son aralığı bulur ve bitiş sınırını denetler.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "Yüklenen [demo paketleri](/projects) diske aşamalanır, sihirli baytlarıyla tanımlanır, açılmış halde 50 MiB ile sınırlanır ve korumalı çerçeveler içinde sınırlı bir önbellekten sunulur. Çalışma alanı ayrıca blok kırıcı ve ışın izleyici dahil Rust demoları derler.",
            },
            {
              kind: "paragraph",
              text: "[EU5 Location DB](/eu5-locations-db), WebAssembly’ye derlenmiş bir Slint uygulamasıdır. Sayfa açık veya koyu modunu küçük bir mesaj protokolüyle iletir; böylece gömülü palet site ile birlikte değişir.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Fotoğraf işleme hattı",
          blocks: [
            {
              kind: "paragraph",
              text: "Yüklemeler sabit piksel sınırları içinde çözülür ve en yavaş ön ayarda AVIF olarak kodlanır; işlem süresi, daha küçük dosyalar karşılığında feda edilir. EXIF verisindeki konumlar yayımlamadan önce haritada onaylanabilir ve aynı anda en fazla iki işleme işi çalışır.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Teknoloji yığını",
      title: "Performans",
      lede: "Backend’de profil güdümlü optimizasyon ve BOLT; darboğaz olmayan bir yük üreticisiyle ölçüldü.",
      sections: [
        {
          id: "approach",
          heading: "Yaklaşım",
          blocks: [
            {
              kind: "paragraph",
              text: "Optimize edilmiş derlemeler temsili bir iş yükü üzerinde eğitilir. Araçlandırılmış bir ikili dosya, sentetik PostgreSQL, SMTP, S3, OpenID Connect ve Minecraft uç noktalarına sahip atılabilir bir test düzeneğine karşı profil kaydeder. LLVM sunucuyu bu profillerle yeniden derler (PGO), ardından BOLT örneklenmiş yürütmeye bakarak son ikili dosyanın kod yerleşimini yeniden sıralar. Tüm varyantlar LTO, çerçeve işaretçileri ve yeniden derlenmiş bir standart kitaplığı paylaşır.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Kurulum",
          blocks: [
            {
              kind: "list",
              items: [
                "Sunucu, test düzeneği sağlayıcıları ve PostgreSQL bir AMD Ryzen 7 9800X3D’nin dört çekirdeğine sabitlenmiş; yük istemcisi diğer dört çekirdeğe sabitlenmiş.",
                "32 bağlantılı havuzlanmış HTTP/1.1 üzerinden HTTPS, tam yanıt okumaları ve iki saniyelik zaman aşımları.",
                "Varyant başına beş örnekle üç döndürülmüş tur; her biri 10.000 ısınma isteğinden sonra, yeni bir veritabanı anlık görüntüsünden başlatıldı.",
                "Toplam 27.000.000 zamanlanmış istek; hepsi HTTP 200 döndürdü.",
                "Yük oha ile üretilir; Playwright her rotanın yanıtını zamanlanan pencerenin dışında denetler.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Sonuçlar",
          blocks: [
            {
              kind: "bars",
              caption: "Varyant başına 15 örnek üzerinden saniyedeki istek sayısının medyanı.",
              groups: [
                {
                  label: "On rotalı herkese açık okumalar",
                  items: [
                    { label: "Referans", value: 44874.12, display: "44.874" },
                    { label: "PGO", value: 52925.46, display: "52.925" },
                    { label: "PGO + BOLT", value: 53888.61, display: "53.889", emphasis: true },
                  ],
                },
                {
                  label: "Küçük sağlık JSON’u",
                  items: [
                    { label: "Referans", value: 272837.41, display: "272.837" },
                    { label: "PGO", value: 308685.26, display: "308.685", emphasis: true },
                    { label: "PGO + BOLT", value: 306276.64, display: "306.277" },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "On rotalı karışım için kuyruk gecikmesi.",
              columns: ["Varyant", "p95 (ms)", "p99 (ms)"],
              numericColumns: [1, 2],
              rows: [
                ["Referans", "2,380", "2,967"],
                ["PGO", "2,020", "2,524"],
                ["PGO + BOLT", "1,987", "2,483"],
              ],
            },
            {
              kind: "paragraph",
              text: "PGO karma veriminde **%17,9**, PGO ile BOLT birlikte **%20,1** artış sağladı ve her iki optimize derleme de döndürülen her turu kazandı. BOLT, karma okumalarda PGO’ya göre %1,8 ek kazanç sağladı ve basit sağlık rotasında %0,8 daha yavaştı. Bunlar dört çekirdekli sonuçlardır; üretim kapasitesi ayrıca ağa, veritabanının çalışma kümesine ve gerçek trafiğe bağlıdır.",
            },
          ],
        },
        {
          id: "corrections",
          heading: "Düzeltmeler",
          blocks: [
            {
              kind: "list",
              items: [
                "`TCP_NODELAY` başlangıçta ayarlanmamıştı: axum-server’ın varsayılan Rustls kabul edicisi bunu kapalı bırakıyor, bu yüzden HTTP/2 yanıtları gecikmeli bir onayda yaklaşık 40 ms bekleyebiliyordu. Kabul edilen her sokette artık bu ayar yapılıyor.",
                "İlk kıyaslama yükü Playwright ile üretiyordu; bu istemci çalışma ilerledikçe yavaşlıyordu ve yanlış gerilemeler gösterdi. Bu sayılar atıldı.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
