import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "November 2026",
  chrome: {
    overview: "Ringkasan",
    updated: "Terakhir diperbarui",
    onThisPage: "Di halaman ini",
    sectionNavigation: "Halaman Tentang",
    previous: "Sebelumnya",
    next: "Berikutnya",
    readingTime: "{minutes} menit baca",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "Tentang",
      lede: "Insinyur backend dan infrastruktur yang bekerja dengan Rust, PostgreSQL, dan platform cloud. Saya membangun layanan yang memanfaatkan perangkat keras di bawahnya secara efektif, beserta pekerjaan deployment, pemantauan, dan keamanan yang menjaganya tetap berjalan.",
      sections: [
        {
          id: "introduction",
          heading: "Pendahuluan",
          blocks: [
            {
              kind: "paragraph",
              text: "Saya seorang software engineer asal Korea yang tinggal di Colorado. Di **Soundpatrol** saya bekerja sebagai Rust Engineer untuk keamanan siber, DevOps, layanan web, dan perkakas pengembang, terutama dengan Rust, Python, PostgreSQL, Kubernetes, dan Google Cloud.",
            },
            {
              kind: "paragraph",
              text: "Sebelumnya, saya memimpin pekerjaan backend dan infrastruktur untuk digital twin gedung Samsung C&T, berkontribusi pada backend aplikasi Hyundai, Kia, dan Genesis, serta membangun pengumpulan data untuk platform analitik YouTube. Sejak 2023, pekerjaan saya berpusat pada layanan backend, pipeline data, dan infrastruktur di bawahnya.",
            },
            {
              kind: "paragraph",
              text: "Saya meyakini pentingnya membangun layanan yang memanfaatkan perangkat keras dan bahasa pemrograman modern secara efektif. Dalam praktiknya, itu berarti perancangan skema yang cermat, antrean dan cache yang dibatasi, penanganan galat yang eksplisit, serta deployment yang dapat diperiksa dan dipulihkan. Saya senang membantu rekan kerja mempelajari Rust, dan saya ingin bekerja dengan orang-orang yang bangga akan keahliannya.",
            },
          ],
        },
        {
          id: "results",
          heading: "Hasil terpilih",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "Penurunan biaya cloud bulanan untuk digital twin Samsung C&T, dari sekitar $5.000 menjadi $150",
                },
                {
                  value: "80",
                  count: 80,
                  label: "Endpoint REST yang diimplementasikan untuk digital twin, dengan logika domain teknik konstruksi",
                },
                {
                  value: "9 bulan",
                  count: 9,
                  suffix: " bulan",
                  label: "Operasi produksi digital twin tanpa henti saat runtime",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "Kenaikan throughput baca campuran pada backend situs ini berkat [PGO dan BOLT](/about-blog/performance)",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Ukuran container layanan yang di-link statis dari build multi-tahap",
                },
                {
                  value: "30.000",
                  label: "Baris kode yang saya tulis untuk backend digital twin, sekitar 75% dari seluruh kodenya",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Perangkat kerja",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Bahasa pemrograman",
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
                  label: "Keandalan dan keamanan",
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
                  label: "Antarmuka dan perangkat",
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
          heading: "Lebih lanjut tentang saya",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Pengalaman",
                  href: "/about/experience",
                  meta: "2016 - sekarang",
                  text: "Peran dari Soundpatrol hingga magang pertama saya, beserta proyek dan hasil di balik masing-masing.",
                },
                {
                  title: "Proyek",
                  href: "/about/projects",
                  meta: "Rust · sumber terbuka",
                  text: "Sillok, EU5 Location DB, situs web ini, dan karya mandiri serta sumber terbuka lainnya.",
                },
                {
                  title: "Latar belakang",
                  href: "/about/background",
                  meta: "Pendidikan · riset",
                  text: "Pendidikan, riset yang dipublikasikan, penghargaan, bahasa, dan pekerjaan di luar bidang perangkat lunak.",
                },
                {
                  title: "Cara kerja situs ini",
                  href: "/about-blog",
                  meta: "Stack teknologi",
                  text: "Hosting, backend, frontend, subsistem, dan pengukuran performa di balik cyhdev.com.",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "Kontak",
          blocks: [
            {
              kind: "list",
              items: [
                "**Email**: [younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**: [github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**: [linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "Email adalah cara paling andal untuk menghubungi saya. Jika Anda ingin bergabung dengan server Minecraft, sampaikan juga melalui email.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "Tentang",
      title: "Pengalaman",
      lede: "Pekerjaan backend, infrastruktur, dan keamanan di sebuah startup AS, untuk klien korporat Korea, dan dalam magang awal, disusul pekerjaan sebelum saya terjun ke dunia perangkat lunak.",
      sections: [
        {
          id: "software",
          heading: "Rekayasa perangkat lunak",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Rust Engineer",
                  period: "Mei 2026 - sekarang",
                  location: "Amerika Serikat · Jarak jauh",
                  summary: "Keamanan siber, DevOps, pengembangan layanan web, dan perkakas pengembang untuk perusahaan deteksi musik dan audio.",
                  highlights: [
                    "Memelihara infrastruktur Kubernetes di Google Cloud, infrastructure as code, dan alur deployment yang ditinjau dengan kredensial berumur pendek yang dibatasi per lingkungan.",
                    "Merencanakan dan menjalankan migrasi penyimpanan data produksi, memverifikasi keadaan data hasil migrasi dan melatih pemulihan sebelum setiap cutover.",
                    "Menyatukan metrik, log, dan trace di OpenTelemetry, serta membangun perkakas Rust untuk kesehatan layanan, diagnostik, dan visibilitas biaya.",
                    "Mengerjakan kesiapan keamanan: administrasi akses, rotasi kredensial dan pelacakan masa berlakunya, kebijakan, penanganan insiden, dan bukti audit.",
                    "Membangun layanan dan perkakas internal dengan Rust dan Python untuk alur kerja operasional dan dukungan.",
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
                  title: "Software Engineer",
                  period: "Jan 2025 - Jul 2025",
                  location: "Seongnam, Korea Selatan",
                  summary: "Layanan backend dan perkakas infrastruktur untuk platform internal dan aplikasi resmi penggemar sebuah grup K-pop.",
                  highlights: [
                    "Menyumbangkan API, perbaikan bug, dan integrasi untuk backend aplikasi resmi Hyundai, Kia, dan Genesis, bekerja dalam basis kode Java yang besar dan berkoordinasi dengan vendor untuk layanan yang digunakan puluhan juta orang di Asia dan Eropa.",
                    "Membangun pipeline internasionalisasi dengan Rust dan Python yang mengubah dan memuat konten terjemahan ke basis data korporat, berkoordinasi dengan administrator basis data yang bertanggung jawab atas dataset yang terhubung ke sistem pabrik.",
                    "Mengembangkan perkakas pengelolaan infrastruktur untuk layanan internal.",
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
                  title: "Software Engineer Kontrak",
                  period: "Nov 2024 - Des 2024",
                  location: "Seoul, Korea Selatan",
                  highlights: [
                    "Merancang skema PostgreSQL dan backend Rust untuk platform analitik YouTube berbantuan AI, serta memimpin deployment-nya di AWS dan Google Cloud.",
                    "Membangun alur kerja pengumpulan data YouTube API yang mencakup puluhan juta komentar dan pengguna, menggabungkan data kanal, transkrip, dan konten halaman.",
                    "Mengintegrasikan model bahasa yang di-host dan model sumber terbuka di server sendiri untuk menghasilkan ringkasan dan wawasan pemasaran.",
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
                  title: "Backend Software Engineer (Lead)",
                  period: "Agu 2023 - Agu 2024",
                  location: "Seoul, Korea Selatan",
                  summary: "Memimpin pengembangan backend, basis data, dan infrastruktur cloud untuk dua platform gedung dan energi.",
                  highlights: [],
                  projects: [
                    {
                      name: "Digital twin Samsung C&T Global Engineering Center",
                      highlights: [
                        "Mengganti backend Express dan susunan microservice cloud dengan monolit Rust/Axum yang dioptimalkan, menurunkan biaya infrastruktur bulanan dari sekitar **$5.000 menjadi $150**.",
                        "Menulis sekitar **30.000 baris**, kira-kira 75% dari backend, termasuk **80 endpoint REST** yang memuat logika domain teknik konstruksi, integrasi data, dan operasi real-time.",
                        "Menurunkan banyak jalur permintaan dengan P99 beberapa detik menjadi satu atau dua digit milidetik pada deployment dua inti dan 4 GB, melalui Rust asinkron, multithreading, caching, serta penyetelan skema dan kueri PostgreSQL.",
                        "Mengintegrasikan beberapa ribu sensor, meter, serta pengendali HVAC dan sistem air melalui BACnet dan Modbus, langsung di lokasi kantor pusat Samsung C&T.",
                        "Beroperasi selama sembilan bulan di produksi tanpa henti saat runtime, dengan cakupan penanganan galat sekitar 90%.",
                        "Membangun CI/CD dengan runner GitHub, AWS CodeDeploy, dan Docker; build statis yang disetel untuk prosesor menghasilkan container portabel berukuran sekitar 20 MB.",
                        "Menghubungkan inferensi model prediktif melalui ECS, EC2, SQS, SNS, S3, dan SageMaker, mengimplementasikan persyaratan keamanan Samsung SDS, dan mengintegrasikan backend dengan antarmuka Unreal Engine 4 buatan studio mitra.",
                      ],
                    },
                    {
                      name: "Layanan manajemen energi berbasis AI",
                      highlights: [
                        "Merancang skema PostgreSQL dan API Rust/Axum dari nol serta menulis sekitar **25.000 baris**, kira-kira 70% dari backend.",
                        "Memilih SQL langsung melalui tokio-postgres alih-alih ORM demi kendali atas kueri dan waktu kompilasi, dengan menerapkan pelajaran dari digital twin.",
                        "Membantu dua engineer dari latar belakang Node.js dan Java menjadi produktif dengan Rust dalam sekitar satu bulan, dalam tim sembilan orang yang terdiri dari engineer backend, frontend, dan data science.",
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
                  title: "Software Engineering Intern",
                  period: "Agu 2022 - Mar 2023",
                  location: "Austin, Texas · Jarak jauh",
                  highlights: [
                    "Membangun bot Discord dengan Python yang menggunakan Reddit API untuk menemukan permintaan komisi karya seni dan mengirimkannya ke kanal komunitas, dijadwalkan di AWS Lambda setiap lima menit.",
                    "Mengimplementasikan logika Lambda untuk konversi mata uang, penagihan, dan perhitungan biaya tambahan lintas yurisdiksi.",
                    "Berkontribusi pada revisi frontend React untuk tata letak, aset, dan teks.",
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
          heading: "Sebelum perangkat lunak",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Korea Institute of Maritime Science and Technology",
                  title: "Penerjemah Tulis dan Lisan",
                  period: "Jun - Jul 2023, Agu 2024",
                  location: "Seoul, Korea Selatan · Kontrak",
                  highlights: [
                    "Menjadi juru bahasa dalam bahasa Inggris, Korea, dan Norwegia pada konferensi riset maritim yang dihadiri duta besar Norwegia untuk Korea, seorang sekretaris negara Norwegia, pimpinan institut, dan ketua Dongwon Group.",
                    "Menerjemahkan pidato dan melatih staf institut serta direkturnya untuk konferensi internasional dan untuk acara Departemen Energi AS mengenai proyek bersama.",
                  ],
                },
                {
                  organization: "Angkatan Darat Republik Korea · Divisi Infanteri ke-2 Angkatan Darat AS",
                  title: "Sersan, penghubung dan instruktur KATUSA",
                  period: "Mar 2016 - Des 2017",
                  location: "Dongducheon, Korea Selatan",
                  highlights: [
                    "Bertugas di Batalyon Markas divisi melalui program [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army), dalam lingkungan gabungan Korea-Amerika.",
                    "Mengajar dan memproses penerimaan sekitar 6.000 personel Amerika dan Korea dalam kursus kebugaran, budaya, sejarah, dan pencegahan.",
                    "Memimpin tim beranggotakan 13 orang dan menjaga kesiapan selama [krisis 2017](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) di semenanjung Korea.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Petugas pemuat logistik",
                  period: "2020 - 2022",
                  location: "Korea Selatan · Kontrak sesekali",
                  highlights: [
                    "Pekerjaan gudang shift malam di samping kuliah.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "Tentang",
      title: "Proyek",
      lede: "Karya independen dan sumber terbuka, sebagian besar dalam Rust. Demo interaktif ada di [halaman proyek](/projects).",
      sections: [
        {
          id: "featured",
          heading: "Unggulan",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "Kronik kerja berbasis baris perintah. Sasaran, tugas yang selesai, koreksi, dan penarikan dicatat sebagai peristiwa append-only dengan proyeksi terkini yang terindeks, dan arsip berbasis Git menggabungkan riwayat dari berbagai mesin.",
                  tags: [
                    "Event sourcing",
                    "Sync",
                  ],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Aplikasi desktop dan peramban untuk data peta Europa Universalis V, dengan dataset terkompresi yang disematkan, indeks pencarian dan pengurutan yang dihitung sebelumnya, filter kategorikal, dan tabel yang ukurannya dapat diubah. [Coba di sini](/eu5-locations-db).",
                  tags: [
                    "Native + web",
                    "Indexes",
                  ],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Situs ini: penerbitan tulisan, fotografi, forum, obrolan langsung dengan panggilan, penjelajah medan Minecraft, dan demo WebAssembly, dilayani oleh satu binary Rust. [Cara pembuatannya](/about-blog).",
                  tags: [
                    "Monorepo",
                    "Self-hosted",
                  ],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "Generator dan pemformat UUIDv4 dengan keluaran ke terminal dan berkas, pemeriksaan duplikat opsional, serta format yang kompatibel dengan Python dan JSON; sekitar tiga kali lebih cepat daripada uuidgen dalam benchmark lokal awal.",
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
          heading: "Karya lainnya",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: sebuah [perbaikan upstream](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) untuk server Minecraft berbasis Rust yang mempertahankan pengaturan overworld Paper saat impor dan penyimpanan; terbuka untuk ditinjau.",
                "**Perkakas peta Minecraft**: plugin Paper dan layanan Rust di balik [penjelajah medan](/minecraft) situs ini, termasuk format ubin bioma biner yang ringkas dan prediksi bioma berbasis seed.",
                "**Sinkronisasi sistem berkas**: utilitas Rust untuk sinkronisasi folder dua arah dan pembersihan artefak build di berbagai sistem berkas yang berbeda, dengan dry run, jurnal operasi, checksum opsional, dan pengaman terhadap penghapusan besar yang tidak disengaja.",
                "**impulsr**: alat transkripsi YouTube dan pengumpul komentar yang menghasilkan ringkasan dari model bahasa untuk keperluan pemasaran.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Cara saya membangun",
          blocks: [
            {
              kind: "paragraph",
              text: "Saya menyukai sistem yang kecil dan mudah diperiksa: satu binary jika satu binary sudah cukup, batas yang jelas pada setiap antrean dan cache, serta format data yang dipilih sesuai pola akses, bukan karena kebiasaan.",
            },
            {
              kind: "paragraph",
              text: "Pengukuran didahulukan sebelum klaim; [catatan performa](/about-blog/performance) situs ini menunjukkan cara pengukurannya.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "Tentang",
      title: "Latar belakang",
      lede: "Pendidikan, penelitian, kualifikasi, dan minat di luar perangkat lunak.",
      sections: [
        {
          id: "education",
          heading: "Pendidikan",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Sungkyunkwan University",
                  title: "B.Eng. Rekayasa Perangkat Lunak",
                  period: "Mar 2015 - Agu 2023",
                  location: "Suwon, Korea Selatan",
                  summary: "Departemen Ilmu dan Teknik Komputer. Rentang studi mencakup wajib militer.",
                  highlights: [
                    "Mata kuliah mencakup perangkat lunak tertanam, rangkaian logika, sistem operasi, jaringan, pemrograman tingkat rendah, dan elektronika, di samping praktik rekayasa perangkat lunak.",
                    "Sungkyunkwan University berawal pada tahun 1398 sebagai akademi kerajaan dinasti Joseon dan merupakan salah satu universitas riset terkemuka di Korea.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Penelitian",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Korean Language NLP Model Based Emotional Analysis of Social Media Communities.** Menerapkan KoBERT pada analisis sentimen dan pengategorian media sosial berbahasa Korea, untuk menelaah perubahan emosi pada masa awal COVID-19.",
            },
            {
              kind: "paragraph",
              text: "Proyek ini meraih Penghargaan Perunggu pada ajang penghargaan makalah dan proyek kelulusan universitas, di antara sekitar 60 peserta.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Penghargaan dan kualifikasi",
          blocks: [
            {
              kind: "table",
              columns: [
                "Tahun",
                "Pengakuan",
                "Keterangan",
              ],
              rows: [
                [
                  "2023",
                  "**Penghargaan Perunggu**",
                  "Penghargaan makalah dan proyek kelulusan Sungkyunkwan University",
                ],
                [
                  "2023",
                  "**IELTS 8.5 / 9.0**",
                  "CEFR C2",
                ],
                [
                  "2021",
                  "**Peringkat ke-7**",
                  "Capstone Design and Idea Hackathon, Korean Society for Internet Information",
                ],
                [
                  "2020",
                  "**Peringkat ke-3**",
                  "Hackathon Fintech, Seoul National University of Science and Technology",
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
          heading: "Bahasa",
          blocks: [
            {
              kind: "list",
              items: [
                "**Korea**: bahasa ibu.",
                "**Inggris**: fasih secara profesional (C2).",
                "**Prancis, Jerman, Spanyol, dan Mandarin**: pengetahuan dasar.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "Di luar perangkat lunak",
          blocks: [
            {
              kind: "list",
              items: [
                "**Fotografi**: fotografi lanskap dan potret sejak 2010, dengan sekitar 30.000 bingkai yang diarsipkan. Sebagian karya terpilih ada di [galeri](/photographs).",
                "**Jurnalisme**: reporter mahasiswa untuk Sungkyun Times, majalah berbahasa Inggris milik universitas, meliput teknologi, keamanan, dan urusan internasional.",
                "**Mengajar dan menjadi relawan**: mengajar bahasa Inggris di pusat komunitas dan menjadi relawan di toko barang bekas yang dikelola penyandang disabilitas.",
                "**Penerjemahan**: juru bahasa dan penerjemah untuk lembaga publik; lihat [Pengalaman](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
