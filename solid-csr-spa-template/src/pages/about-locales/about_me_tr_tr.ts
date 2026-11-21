import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "Kasım 2026",
  chrome: {
    overview: "Genel bakış",
    updated: "Son güncelleme",
    onThisPage: "Bu sayfada",
    sectionNavigation: "Hakkında sayfaları",
    previous: "Önceki",
    next: "Sonraki",
    readingTime: "{minutes} dk okuma",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "Hakkında",
      lede: "Rust, PostgreSQL ve bulut platformlarıyla çalışan backend ve altyapı mühendisiyim. Altlarındaki donanımı verimli kullanan servisler geliştiriyor, bunları ayakta tutan dağıtım, izleme ve güvenlik işlerini de yürütüyorum.",
      sections: [
        {
          id: "introduction",
          heading: "Giriş",
          blocks: [
            {
              kind: "paragraph",
              text: "Colorado’da yaşayan Koreli bir yazılım mühendisiyim. **Soundpatrol**’da Rust Mühendisi olarak siber güvenlik, DevOps, web servisleri ve geliştirici araçları üzerinde çalışıyorum; ağırlıklı olarak Rust, Python, PostgreSQL, Kubernetes ve Google Cloud kullanıyorum.",
            },
            {
              kind: "paragraph",
              text: "Bundan önce Samsung C&T binası için bir dijital ikizin backend ve altyapı çalışmalarına liderlik ettim, Hyundai, Kia ve Genesis uygulamalarının backend’lerine katkıda bulundum ve bir YouTube analiz platformunun veri toplama altyapısını kurdum. 2023’ten beri çalışmam backend servisleri, veri hatları ve bunların altındaki altyapı üzerinde yoğunlaşıyor.",
            },
            {
              kind: "paragraph",
              text: "Modern donanımı ve programlama dillerini verimli kullanan servisler kurmaya inanıyorum. Pratikte bu; özenle tasarlanmış şemalar, sınırlı kuyruklar ve önbellekler, açık hata yönetimi ve incelenebilen, kurtarılabilen dağıtımlar demek. Meslektaşlarımın Rust öğrenmesine yardım etmekten hoşlanıyorum ve mesleğiyle gurur duyan insanlarla çalışmak istiyorum.",
            },
          ],
        },
        {
          id: "results",
          heading: "Seçilmiş sonuçlar",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "%97",
                  count: 97,
                  prefix: "%",
                  label: "Samsung C&T dijital ikizinde aylık bulut harcamasındaki azalma; yaklaşık 5.000 dolardan 150 dolara",
                },
                {
                  value: "80",
                  count: 80,
                  label: "Dijital ikiz için, inşaat mühendisliği alan mantığıyla uygulanan REST uç noktası",
                },
                {
                  value: "9 ay",
                  count: 9,
                  suffix: " ay",
                  label: "Dijital ikizin üretimde, çalışma zamanı kapanması olmadan çalışma süresi",
                },
                {
                  value: "+%20",
                  count: 20,
                  prefix: "+%",
                  label: "Bu sitenin backend’inde [PGO ve BOLT](/about-blog/performance) ile elde edilen karma okuma verimi artışı",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Çok aşamalı derlemelerden çıkan, statik bağlı servis konteynerleri",
                },
                {
                  value: "30.000",
                  label: "Dijital ikizin backend’i için yazılan satır sayısı; kodunun yaklaşık %75’i",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Araç seti",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Diller",
                  items: ["Rust", "Python", "TypeScript", "SQL", "Java", "C", "C++"],
                },
                {
                  label: "Backend",
                  items: ["Axum", "Tokio", "Diesel", "tokio-postgres", "Spring Boot", "FastAPI", "WebSockets", "gRPC", "Protocol Buffers"],
                },
                {
                  label: "Veri",
                  items: ["PostgreSQL", "Valkey / Redis", "SQLite / Turso", "MySQL", "FAISS", "tantivy"],
                },
                {
                  label: "Altyapı",
                  items: ["Kubernetes", "Helm", "Docker", "OpenTofu / Terraform", "GitHub Actions", "Google Cloud", "AWS", "Linux"],
                },
                {
                  label: "Güvenilirlik ve güvenlik",
                  items: ["OpenTelemetry", "IAM", "TLS", "Network policy", "Credential lifecycle", "Incident response"],
                },
                {
                  label: "Arayüzler ve cihazlar",
                  items: ["SolidJS", "Leptos", "Slint", "WebAssembly", "WebRTC", "BACnet", "Modbus"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "Hakkımda daha fazlası",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Deneyim",
                  href: "/about/experience",
                  meta: "2016 - günümüz",
                  text: "Soundpatrol’dan ilk stajıma uzanan roller; her birinin ardındaki projeler ve sonuçlar.",
                },
                {
                  title: "Projeler",
                  href: "/about/projects",
                  meta: "Rust · açık kaynak",
                  text: "Sillok, EU5 Location DB, bu web sitesi ve diğer bağımsız ve açık kaynak çalışmalar.",
                },
                {
                  title: "Özgeçmiş",
                  href: "/about/background",
                  meta: "Eğitim · araştırma",
                  text: "Eğitim, yayımlanmış araştırma, ödüller, diller ve yazılım dışındaki işler.",
                },
                {
                  title: "Bu site nasıl çalışıyor",
                  href: "/about-blog",
                  meta: "Teknoloji yığını",
                  text: "cyhdev.com sitesinin arkasındaki sunucu, backend, frontend, alt sistemler ve performans ölçümleri.",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "İletişim",
          blocks: [
            {
              kind: "list",
              items: [
                "**E-posta**: [younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**: [github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**: [linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "Bana ulaşmanın en güvenilir yolu e-postadır. Minecraft sunucusuna katılmak isterseniz bunu da e-postada belirtin.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "Hakkında",
      title: "Deneyim",
      lede: "Bir ABD girişiminde, Koreli kurumsal müşteriler için ve erken bir stajda yürüttüğüm backend, altyapı ve güvenlik çalışmaları; ardından yazılımdan önceki işler.",
      sections: [
        {
          id: "software",
          heading: "Yazılım mühendisliği",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Rust Mühendisi",
                  period: "Mayıs 2026 - günümüz",
                  location: "Amerika Birleşik Devletleri · Uzaktan",
                  summary: "Bir müzik ve ses tespiti şirketi için siber güvenlik, DevOps, web servisi geliştirme ve geliştirici araçları.",
                  highlights: [
                    "Google Cloud üzerindeki Kubernetes altyapısını, kod olarak altyapıyı ve kısa ömürlü, ortama özel kimlik bilgileri kullanan, gözden geçirilmiş dağıtım iş akışlarını sürdürüyorum.",
                    "Üretimdeki veri depolarının taşınmasını planlıyor ve yürütüyorum; her geçişten önce taşınan durumu doğruluyor, kurtarma adımlarını prova ediyorum.",
                    "Metrikleri, günlükleri ve izleri OpenTelemetry üzerinde birleştiriyor; servis sağlığı, tanılama ve maliyet görünürlüğü için Rust araçları geliştiriyorum.",
                    "Güvenlik hazırlığı üzerinde çalışıyorum: erişim yönetimi, kimlik bilgisi döndürme ve süre sonu takibi, politikalar, olay müdahalesi ve denetim kanıtları.",
                    "Operasyon ve destek iş akışları için dahili Rust ve Python servisleri ve araçları geliştiriyorum.",
                  ],
                  tags: ["Rust", "Python", "PostgreSQL", "Kubernetes", "Google Cloud", "OpenTofu", "OpenTelemetry"],
                },
                {
                  organization: "GenesisNest",
                  title: "Yazılım Mühendisi",
                  period: "Oca 2025 - Tem 2025",
                  location: "Seongnam, Güney Kore",
                  summary: "Şirket içi platformlar ve bir K-pop grubunun resmi hayran uygulaması için backend servisleri ve altyapı araçları.",
                  highlights: [
                    "Hyundai, Kia ve Genesis resmi uygulama backend’lerine API’ler, hata düzeltmeleri ve entegrasyonlar kazandırdım; büyük bir Java kod tabanında çalıştım ve Asya ile Avrupa’da on milyonlarca kişinin kullandığı servisler için tedarikçilerle koordinasyon sağladım.",
                    "Çevrilmiş içeriği dönüştürüp kurumsal veritabanlarına yükleyen uluslararasılaştırma hatlarını Rust ve Python ile kurdum; fabrika sistemlerine bağlı veri kümelerinden sorumlu veritabanı yöneticileriyle koordinasyon içinde çalıştım.",
                    "Şirket içi servisler için altyapı yönetim araçları geliştirdim.",
                  ],
                  tags: ["Java", "Spring Boot", "Rust", "Python"],
                },
                {
                  organization: "pampam Inc.",
                  title: "Sözleşmeli Yazılım Mühendisi",
                  period: "Kas 2024 - Ara 2024",
                  location: "Seul, Güney Kore",
                  highlights: [
                    "Yapay zekâ destekli bir YouTube analiz platformunun PostgreSQL şemasını ve Rust backend’ini tasarladım; AWS ve Google Cloud üzerindeki dağıtımına liderlik ettim.",
                    "Kanal verilerini, transkriptleri ve sayfa içeriğini birleştirerek on milyonlarca yorumu ve kullanıcıyı kapsayan YouTube API toplama iş akışları kurdum.",
                    "Özet ve pazarlama içgörüleri üretmek için barındırılan dil modellerini ve sunucuda çalışan açık kaynak modelleri entegre ettim.",
                  ],
                  tags: ["Rust", "PostgreSQL", "AWS", "Google Cloud"],
                },
                {
                  organization: "EAN Technology Co. Ltd.",
                  title: "Backend Yazılım Mühendisi (Lider)",
                  period: "Ağu 2023 - Ağu 2024",
                  location: "Seul, Güney Kore",
                  summary: "İki bina ve enerji platformu için backend, veritabanı ve bulut altyapısı geliştirmeye liderlik ettim.",
                  highlights: [],
                  projects: [
                    {
                      name: "Samsung C&T Global Engineering Center dijital ikizi",
                      highlights: [
                        "Bir Express backend’ini ve bulut mikroservis düzenini optimize edilmiş bir Rust/Axum monolitiyle değiştirdim; aylık altyapı maliyetini yaklaşık **5.000 dolardan 150 dolara** düşürdüm.",
                        "Backend’in kabaca %75’ine denk gelen yaklaşık **30.000 satır** kod yazdım; buna inşaat mühendisliği alan mantığı, veri entegrasyonu ve gerçek zamanlı işlemler taşıyan **80 REST uç noktası** dahildir.",
                        "Çok saniyelik P99 değerine sahip birçok istek yolunu; asenkron Rust, çoklu iş parçacığı, önbellekleme ve PostgreSQL şema ve sorgu ayarıyla, iki çekirdekli ve 4 GB’lık bir dağıtımda tek veya çift haneli milisaniyelere indirdim.",
                        "Samsung C&T genel merkezinde, yerinde çalışarak BACnet ve Modbus üzerinden birkaç bin sensörü, sayacı ile HVAC ve su sistemi denetleyicisini entegre ettim.",
                        "Üretimde dokuz ay boyunca, çalışma zamanı kapanması olmadan ve yaklaşık %90 hata yönetimi kapsamıyla çalıştı.",
                        "GitHub runner’ları, AWS CodeDeploy ve Docker ile CI/CD kurdum; statik, işlemciye göre ayarlanmış derlemeler yaklaşık 20 MB’lık taşınabilir konteynerler üretti.",
                        "Tahmin modeli çıkarımını ECS, EC2, SQS, SNS, S3 ve SageMaker üzerinden bağladım, Samsung SDS güvenlik gereksinimlerini uyguladım ve backend’i bir ortak stüdyonun geliştirdiği Unreal Engine 4 arayüzüyle entegre ettim.",
                      ],
                    },
                    {
                      name: "Yapay zekâ enerji yönetimi servisi",
                      highlights: [
                        "PostgreSQL şemasını ve Rust/Axum API’sini sıfırdan tasarladım ve backend’in kabaca %70’ine denk gelen yaklaşık **25.000 satır** yazdım.",
                        "Sorgular ve derleme süreleri üzerinde denetim sağlamak için ORM yerine tokio-postgres ile doğrudan SQL kullanmayı seçtim; dijital ikizden çıkardığım dersleri uyguladım.",
                        "Node.js ve Java geçmişinden gelen iki mühendisin yaklaşık bir ayda Rust’ta üretken hale gelmesine yardım ettim; ekip backend, frontend ve veri bilimi mühendislerinden oluşan dokuz kişiydi.",
                      ],
                    },
                  ],
                  tags: ["Rust", "Axum", "PostgreSQL", "AWS", "Docker", "BACnet", "Modbus"],
                },
                {
                  organization: "Artifyc Inc.",
                  title: "Yazılım Mühendisliği Stajyeri",
                  period: "Ağu 2022 - Mar 2023",
                  location: "Austin, Teksas · Uzaktan",
                  highlights: [
                    "Reddit API’sini kullanarak sanat siparişi taleplerini bulan ve bir topluluk kanalında paylaşan, AWS Lambda üzerinde beş dakikada bir zamanlanan bir Python Discord botu geliştirdim.",
                    "Yetki alanları arasında para birimi dönüştürme, ücretlendirme ve ek ücret hesaplama için Lambda mantığını uyguladım.",
                    "React frontend’inde düzen, varlıklar ve metinlerle ilgili düzeltmelere katkıda bulundum.",
                  ],
                  tags: ["Python", "AWS Lambda", "React"],
                },
              ],
            },
          ],
        },
        {
          id: "other",
          heading: "Yazılımdan önce",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Kore Denizcilik Bilim ve Teknoloji Enstitüsü",
                  title: "Çevirmen ve Tercüman",
                  period: "Haz - Tem 2023, Ağu 2024",
                  location: "Seul, Güney Kore · Sözleşmeli",
                  highlights: [
                    "Kore’deki Norveç büyükelçisinin, bir Norveç devlet sekreterinin, enstitü yönetiminin ve Dongwon Group başkanının katıldığı bir denizcilik araştırma konferansında İngilizce, Korece ve Norveççe tercümanlık yaptım.",
                    "Uluslararası bir konferans ve ortak bir proje hakkındaki ABD Enerji Bakanlığı etkinliği için konuşmaları çevirdim; enstitü personeline ve müdürüne koçluk yaptım.",
                  ],
                },
                {
                  organization: "Kore Cumhuriyeti Ordusu · ABD Ordusu 2. Piyade Tümeni",
                  title: "Çavuş, KATUSA irtibat görevlisi ve eğitmen",
                  period: "Mar 2016 - Ara 2017",
                  location: "Dongducheon, Güney Kore",
                  highlights: [
                    "Tümenin Karargâh Taburu’nda [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army) programı kapsamında, ortak bir Kore-Amerikan ortamında görev yaptım.",
                    "Yaklaşık 6.000 Amerikalı ve Koreli personele fiziksel zindelik, kültür, tarih ve önleme kursları verdim ve göreve kabul işlemlerini yürüttüm.",
                    "13 kişilik bir ekibe liderlik ettim ve yarımadadaki [2017 krizi](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) boyunca hazırlık durumunu korudum.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Lojistik yükleme görevlisi",
                  period: "2020 - 2022",
                  location: "Güney Kore · Ara sıra sözleşmeler",
                  highlights: [
                    "Üniversite eğitimimle birlikte gece vardiyalarında depo işi.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "Hakkında",
      title: "Projeler",
      lede: "Bağımsız ve açık kaynak çalışmalar; çoğu Rust ile. Etkileşimli demolar [projeler sayfasında](/projects) bulunuyor.",
      sections: [
        {
          id: "featured",
          heading: "Öne çıkanlar",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "Komut satırında bir iş günlüğü. Hedefler, tamamlanan görevler, düzeltmeler ve geri çekmeler yalnızca eklemeli olaylar olarak kaydedilir; güncel durum için dizinlenmiş görünümler tutulur ve Git destekli bir arşiv, makineler arasındaki geçmişleri birleştirir.",
                  tags: ["Event sourcing", "Sync"],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Europa Universalis V harita verileri için bir masaüstü ve tarayıcı uygulaması; gömülü sıkıştırılmış veri kümeleri, önceden hesaplanmış arama ve sıralama indeksleri, kategorik filtreler ve yeniden boyutlandırılabilir tablolar içerir. [Buradan deneyin](/eu5-locations-db).",
                  tags: ["Native + web", "Indexes"],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Bu site: yayıncılık, fotoğrafçılık, forum, aramalı canlı sohbet, Minecraft arazi gezgini ve WebAssembly demoları; hepsi tek bir Rust ikili dosyasından sunulur. [Nasıl kurulduğu](/about-blog).",
                  tags: ["Monorepo", "Self-hosted"],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "Terminal ve dosya çıktısı, isteğe bağlı yinelenen kontrolü, Python ve JSON uyumlu biçimlendirme sunan bir UUIDv4 üreticisi ve biçimlendiricisi; erken bir yerel kıyaslamada uuidgen’den yaklaşık üç kat hızlı.",
                  tags: ["Throughput"],
                },
              ],
            },
          ],
        },
        {
          id: "more",
          heading: "Diğer çalışmalar",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: Rust ile yazılmış Minecraft sunucusu için, içe aktarma ve kaydetme sırasında Paper ana dünya (Overworld) ayarlarını koruyan bir [upstream düzeltmesi](https://github.com/Pumpkin-MC/Pumpkin/pull/3447); incelemeye açık.",
                "**Minecraft harita araçları**: bu sitenin [arazi gezgininin](/minecraft) arkasındaki bir Paper eklentisi ve Rust servisleri; kompakt bir ikili biyom karo biçimi ve seed tabanlı biyom tahminleri dahil.",
                "**Dosya sistemi eşitleme**: farklı dosya sistemleri arasında iki yönlü klasör eşitleme ve derleme çıktısı temizliği için bir Rust aracı; deneme çalıştırmaları, işlem günlükleri, isteğe bağlı sağlama toplamları ve büyük, istenmeyen silmelere karşı korumalar içerir.",
                "**impulsr**: pazarlama işleri için dil modeli özetleri üreten bir YouTube transkripsiyon ve yorum toplama aracı.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Nasıl geliştiriyorum",
          blocks: [
            {
              kind: "paragraph",
              text: "Küçük ve incelenebilir sistemleri tercih ediyorum: bir ikili dosya yetiyorsa tek ikili dosya, her kuyruk ve önbellek için açık bir sınır, alışkanlıktan değil erişim biçimine göre seçilmiş veri biçimleri.",
            },
            {
              kind: "paragraph",
              text: "Ölçümler iddialardan önce gelir; bu sitenin [performans notları](/about-blog/performance) ölçümlerin nasıl alındığını gösterir.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "Hakkında",
      title: "Özgeçmiş",
      lede: "Eğitim, araştırma, nitelikler ve yazılım dışındaki ilgi alanları.",
      sections: [
        {
          id: "education",
          heading: "Eğitim",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Sungkyunkwan Üniversitesi",
                  title: "Yazılım Mühendisliği Lisansı (B.Eng.)",
                  period: "Mar 2015 - Ağu 2023",
                  location: "Suwon, Güney Kore",
                  summary: "Bilgisayar Bilimi ve Mühendisliği Bölümü. Öğrenim süresi zorunlu askerlik hizmetini içerir.",
                  highlights: [
                    "Yazılım mühendisliği uygulamasının yanı sıra gömülü yazılım, mantık devreleri, işletim sistemleri, ağlar, alt düzey programlama ve elektronik dersleri.",
                    "Sungkyunkwan Üniversitesi’nin kökeni, Joseon hanedanının kraliyet akademisi olarak 1398’e uzanır; üniversite Kore’nin önde gelen araştırma üniversitelerinden biridir.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Araştırma",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Korece Doğal Dil İşleme Modeli Tabanlı Sosyal Medya Topluluklarının Duygu Analizi.** Korece sosyal medya içeriğinin duygu analizi ve sınıflandırması için KoBERT uygulandı; COVID-19’un ilk dönemindeki duygusal değişim incelendi.",
            },
            {
              kind: "paragraph",
              text: "Proje, üniversitenin bitirme tezi ve proje ödüllerinde yaklaşık 60 başvuru arasından Bronz Ödül aldı.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Ödüller ve nitelikler",
          blocks: [
            {
              kind: "table",
              columns: ["Yıl", "Tanınma", "Ayrıntı"],
              rows: [
                ["2023", "**Bronz Ödül**", "Sungkyunkwan Üniversitesi bitirme tezi ve proje ödülleri"],
                ["2023", "**IELTS 8.5 / 9.0**", "CEFR C2"],
                ["2021", "**7. sıra**", "Capstone Design ve Fikir Hackathon’u, Kore İnternet Bilgi Derneği"],
                ["2020", "**3. sıra**", "Fintech Hackathon’u, Seul Ulusal Bilim ve Teknoloji Üniversitesi"],
                ["2018", "**TOEFL 117 / 120**", "CEFR C2"],
              ],
            },
          ],
        },
        {
          id: "languages",
          heading: "Diller",
          blocks: [
            {
              kind: "list",
              items: [
                "**Korece**: ana dil.",
                "**İngilizce**: profesyonel akıcılık (C2).",
                "**Fransızca, Almanca, İspanyolca ve Mandarin**: temel düzeyde aşinalık.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "Yazılım dışında",
          blocks: [
            {
              kind: "list",
              items: [
                "**Fotoğrafçılık**: 2010’dan beri manzara ve portre fotoğrafçılığı; yaklaşık 30.000 arşivlenmiş kare. Bir seçki [galeride](/photographs).",
                "**Gazetecilik**: üniversitenin İngilizce dergisi Sungkyun Times’ın öğrenci muhabiri olarak teknoloji, güvenlik ve uluslararası ilişkiler üzerine haberler yaptım.",
                "**Öğretmenlik ve gönüllülük**: bir toplum merkezinde İngilizce öğrettim ve engelli kişilerce işletilen bir ikinci el mağazasında gönüllü çalıştım.",
                "**Çeviri**: kamu kurumları için tercümanlık ve çevirmenlik; bkz. [Deneyim](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
