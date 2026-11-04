import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "November 2026",
  chrome: {
    overview: "Ringkasan",
    updated: "Terakhir diperbarui",
    onThisPage: "Di halaman ini",
    sectionNavigation: "Halaman stack teknologi",
    previous: "Sebelumnya",
    next: "Berikutnya",
    readingTime: "{minutes} menit baca",
  },
  pages: {
    overview: {
      eyebrow: "Tentang situs ini",
      title: "Stack Teknologi Blog",
      lede: "Satu binary Rust, satu basis data PostgreSQL, satu mesin kecil. Halaman-halaman ini menjelaskan cara situs ini dibangun, dihosting, diamankan, dan diukur, serta alasan di balik bentuknya.",
      sections: [
        {
          id: "at-a-glance",
          heading: "Sekilas",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53.889 req/s",
                  label: "Median throughput baca publik campuran dengan PGO dan BOLT pada empat inti server",
                },
                {
                  value: "~150 µs",
                  label: "Round trip basis data yang teramati melalui UNIX socket, dibandingkan ~600 µs melalui TCP localhost",
                },
                {
                  value: "0",
                  label: "Proses Node.js di produksi; frontend disematkan di dalam binary server",
                },
                {
                  value: "24",
                  count: 24,
                  label: "Bahasa antarmuka, masing-masing diunduh hanya saat dipilih",
                },
                {
                  value: "4.096",
                  label: "Koneksi terbuka yang diterima sebelum TLS, maksimal 64 per alamat atau IPv6 /64",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Proses yang melayani TLS, API, WebSocket, media WebRTC, dan frontend yang disematkan",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Jalur permintaan",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Browser",
                  detail: "HTTP/2 atau HTTP/1.1 melalui TLS. Frontend di-cache di bawah URL immutable yang memakai fingerprint.",
                },
                {
                  label: "Penerimaan",
                  detail: "Koneksi TCP diterima sebelum TLS: 4.096 secara total dan 64 per alamat atau IPv6 /64.",
                },
                {
                  label: "rustls",
                  detail: "TLS diakhiri di dalam proses Rust, dengan `TCP_NODELAY` pada setiap socket.",
                },
                {
                  label: "Axum",
                  detail: "Header keamanan, sesi, pembatasan laju, kompresi, dan batas ukuran body, lalu handler fitur.",
                },
                {
                  label: "Cache · Diesel",
                  detail: "Cache dalam memori yang dibatasi melayani pembacaan yang sering diakses; sisanya melalui Diesel asinkron.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "Di host yang sama melalui UNIX socket, dengan I/O asinkron io_uring.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Bab",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Infrastruktur",
                  href: "/about-blog/infrastructure",
                  meta: "Host · Gentoo · jaringan",
                  text: "Mesinnya, sistem operasi dan sistem berkas, jaringan, serta alasan tidak ada reverse proxy.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "Stack layanan, model data, batas runtime, header keamanan, dan kontrak API.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "Shell aplikasi, pemuatan, lokalisasi, sistem desain, dan animasi.",
                },
                {
                  title: "Subsistem",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "Obrolan langsung dengan SFU dalam proses, penjelajah Minecraft, pencarian Geo-IP, dan WebAssembly.",
                },
                {
                  title: "Performa",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Build yang dioptimalkan, cara pengukurannya, dan hasilnya.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Cara perubahan diperiksa",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy pada target native dan WebAssembly sebagai gerbang implementasi.",
                "Pemeriksaan cargo xtask lokal untuk pemformatan, unit test native, penyimpangan kontrak OpenAPI, serta typecheck, lint, unit test, dan build frontend.",
                "Playwright pada Chromium dan WebKit dengan pemeriksaan aksesibilitas axe dan pemeriksaan reflow seluler hingga 320 piksel CSS.",
                "Test integrasi dan rollback migrasi pada instans PostgreSQL sekali pakai, ditambah pemindaian rahasia gitleaks secara lokal.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Stack teknologi",
      title: "Infrastruktur",
      lede: "Mesin kecil di rumah, distribusi Linux berbasis source, dan tanpa lapisan yang tidak dibutuhkan lalu lintas.",
      sections: [
        {
          id: "host",
          heading: "Host",
          blocks: [
            {
              kind: "paragraph",
              text: "Situs ini berjalan di [miniserver](https://store.minisforum.com/products/minisforum-um690l-slim) di tempat tinggal saya: prosesor mobile Ryzen 9 delapan inti yang naik hingga 4,9 GHz, RAM 32 GB, dan SSD NVMe 1 TB. Dengan harga sekitar $400, biayanya lebih rendah daripada setahun memakai perangkat keras cloud yang jauh lebih lemah, dan mencerminkan antusiasme saya terhadap self-hosting.",
            },
            {
              kind: "paragraph",
              text: "Mesin ini menjalankan server gabungan backend dan frontend, PostgreSQL, dan server Minecraft. Jika Anda ingin bermain, kirim email kepada saya.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo dan btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) bukan pilihan yang lazim untuk server; di tempat kerja saya akan memilih Debian stable di atas ext4. Di sini saya senang mengompilasi paket sendiri dengan optimasi untuk prosesor yang persis dipakai.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) menghadirkan snapshot, kompresi, dan copy-on-write, tetapi copy-on-write sangat memfragmentasi berkas basis data. Karena itu direktori data PostgreSQL dan Minecraft dikecualikan darinya. Sebuah [benchmark sistem berkas](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) menunjukkan btrfs bawaan tertinggal dari ext4 dan XFS; saya menduga menonaktifkan copy-on-write menutup sebagian besar selisih itu, dan itu akan menjadi pengukuran yang menarik.",
            },
          ],
        },
        {
          id: "network",
          heading: "Jaringan",
          blocks: [
            {
              kind: "paragraph",
              text: "Lalu lintas masuk melalui sambungan Xfinity berkabel 1 Gbps; modem penyedia tidak mendukung Ethernet 2,5 Gb, dan situs ini tidak akan merasakannya. Route 53 menghosting DNS.",
            },
            {
              kind: "paragraph",
              text: "Di dalam mesin tidak ada reverse proxy, container runtime, atau service mesh. PostgreSQL berjalan langsung di sistem operasi, dan satu binary Rust melayani frontend serta API. Permintaan HTTP biasa dialihkan ke HTTPS oleh proses yang sama.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "UNIX socket",
          blocks: [
            {
              kind: "paragraph",
              text: "Server mengakses PostgreSQL melalui UNIX socket, bukan TCP localhost, sehingga tumpukan jaringan dilewati sepenuhnya. [Benchmark](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) menunjukkan laju transaksi hampir dua kali lipat; di sini saya mengamati round trip serendah **150 µs**, dibandingkan sekitar **600 µs** melalui localhost. Cara ini kuno dan tidak relevan bagi sebagian besar deployment cloud, tetapi lebih cepat.",
            },
            {
              kind: "paragraph",
              text: "PostgreSQL 18, dirilis pada September 2025, menambahkan [I/O asinkron](https://pganalyze.com/blog/postgres-18-async-io), dan io_uring diaktifkan di sini, yang mempercepat pembacaan secara nyata.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Build dan deployment",
          blocks: [
            {
              kind: "list",
              items: [
                "Build frontend disematkan di dalam binary backend bersama salinan zstd dan gzip yang sudah dikompresi sebelumnya, sehingga tidak ada proses Node.js yang berjalan di produksi.",
                "Build yang dioptimalkan dapat dilatih dengan profile-guided optimization dan dioptimalkan pasca-link dengan BOLT; lihat [Performa](/about-blog/performance).",
                "Kredensial, sertifikat, dan data runtime tidak masuk ke Git, dan pemindaian gitleaks memeriksa pohon berkas sebelum perubahan keluar dari mesin.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Stack teknologi",
      title: "Backend",
      lede: "Rust dengan Axum dan Tokio, PostgreSQL melalui Diesel asinkron, dan batas yang eksplisit untuk semua yang dapat dibiarkan terbuka oleh klien.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Runtime",
                  items: [
                    "Axum",
                    "Tokio",
                    "hyper",
                    "axum-server",
                    "rustls",
                    "mimalloc",
                  ],
                },
                {
                  label: "Data",
                  items: [
                    "PostgreSQL 18",
                    "Diesel",
                    "diesel-async",
                    "tantivy",
                    "bitcode",
                    "zstd",
                  ],
                },
                {
                  label: "Integrasi",
                  items: [
                    "openidconnect",
                    "lettre",
                    "Amazon S3",
                    "webrtc-rs",
                    "image + AVIF",
                  ],
                },
                {
                  label: "Kontrak",
                  items: [
                    "utoipa",
                    "OpenAPI",
                    "Swagger UI",
                    "Generated TypeScript",
                  ],
                },
              ],
            },
            {
              kind: "paragraph",
              text: "Layanan ini adalah satu proses dengan state bersama: sesi dalam memori, sinkronisasi cache saat startup, dan job pemeliharaan terjadwal. mimalloc adalah allocator global; jika sebuah mesin punya inti yang sungguhan, perilaku allocator layak dianggap serius.",
            },
          ],
        },
        {
          id: "data",
          heading: "Model data",
          blocks: [
            {
              kind: "paragraph",
              text: "Dalam pekerjaan startup dan korporasi Korea yang pernah saya lihat, MySQL dan MariaDB dianggap satu-satunya basis data relasional yang serius. PostgreSQL sudah lama melampaui keduanya di bidang yang saya pedulikan, mulai dari UUID native hingga JSON biner.",
            },
            {
              kind: "list",
              items: [
                "**Artikel** memiliki slug, ringkasan, metadata JSONB, status publikasi, penghitung denormalisasi, dan pemetaan tag. Komentar bersarang melalui parent yang nullable, dan suara untuk artikel dan komentar memiliki tabel terpisah.",
                "**Akun** menyimpan negara dan bahasa, tabel peran dan izin, serta foto profil berversi di tabelnya sendiri, bukan pada baris pengguna yang sering diakses.",
                "**Kunci** adalah nilai UUIDv7 yang berurutan menurut waktu: tidak dapat ditebak urutannya seperti ID sekuensial, dan jauh lebih ramah terhadap lokalitas B-tree daripada UUIDv4 acak.",
                "**Pencarian** memakai indeks tantivy yang disimpan di disk atas judul artikel dan tag, dijaga tetap koheren dengan PostgreSQL, yang tetap menjadi sumber otoritatif.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Batas runtime",
          blocks: [
            {
              kind: "list",
              items: [
                "Koneksi diterima sebelum TLS melalui limiter bersama: 4.096 koneksi terbuka dan 64 per alamat IPv4 atau IPv6 /64. Socket yang ditolak ditutup tanpa handshake.",
                "Head permintaan HTTP/1 harus tiba dalam 20 detik; koneksi HTTP/2 di-ping setiap 30 detik dan mengizinkan 128 stream bersamaan.",
                "Setiap cache memiliki batas tetap dan sumber otoritatif tempat ia membaca ulang; tidak ada yang tumbuh mengikuti key yang dikendalikan klien. Cache bundle WebAssembly, misalnya, menampung paling banyak 256 MiB.",
                "Pekerjaan yang memblokir seperti pengenkodean gambar berjalan di worker pool yang dibatasi, bukan di worker Tokio.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Header keamanan",
          blocks: [
            {
              kind: "paragraph",
              text: "Content-Security-Policy dibangun sekali saat startup. Skrip inline hanya diizinkan melalui hash SHA-256 yang dihitung dari `index.html` yang dikompilasi ke dalam binary, sehingga skrip tema pra-render tidak dapat menyimpang dari kebijakan. Origin utama tidak pernah menginstansiasi WebAssembly: demo dan aplikasi EU5 berjalan di frame sandbox dengan kebijakan masing-masing.",
            },
            {
              kind: "paragraph",
              text: "Cookie sesi bersifat `Secure` dan `HttpOnly`, operasi superuser memeriksa peran di server, akun dapat menautkan identitas OpenID Connect, dan Swagger UI memerlukan autentikasi di produksi.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Kontrak dan aset",
          blocks: [
            {
              kind: "list",
              items: [
                "Handler memiliki anotasi OpenAPI, klien TypeScript frontend dihasilkan darinya, dan cargo xtask openapi gagal jika berkas hasil generate menyimpang.",
                "Manifest Vite disematkan bersama aset. Hanya berkas berfingerprint yang tercantum di manifest yang mendapat kebijakan cache `immutable` satu tahun; HTML selalu divalidasi ulang.",
                "Path aset yang tidak ada mengembalikan 404 dengan `no-store`, bukan shell aplikasi, sehingga halaman usang tidak dapat men-cache HTML di bawah URL skrip.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Stack teknologi",
      title: "Frontend",
      lede: "Aplikasi SolidJS 2 yang dirender di sisi klien, dikompilasi menjadi aset statis dan disematkan dalam binary server.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Aplikasi",
                  items: [
                    "SolidJS 2",
                    "Solid Router 2",
                    "TypeScript",
                    "Vite 8",
                  ],
                },
                {
                  label: "Gaya tampilan",
                  items: [
                    "Tailwind CSS 4",
                    "IBM Plex Sans",
                    "IBM Plex Mono",
                    "OKLab",
                  ],
                },
                {
                  label: "Komponen",
                  items: [
                    "Toast UI Editor",
                    "Leaflet",
                    "Chart.js",
                    "highlight.js",
                    "DOMPurify",
                  ],
                },
                {
                  label: "Pengujian",
                  items: [
                    "Vitest",
                    "Playwright",
                    "axe-core",
                  ],
                },
              ],
            },
            {
              kind: "paragraph",
              text: "Signal Solid yang bergranularitas halus hanya memperbarui node DOM yang bergantung pada nilai yang berubah, dan rilis kandidat 2.0-nya memungkinkan pembacaan asinkron ditangguhkan hingga batas pemuatan terdekat. Router memuat setiap halaman secara lazy, dan tidak ada server JavaScript di produksi: binary Rust menyajikan berkas statis.",
            },
          ],
        },
        {
          id: "loading",
          heading: "Pemuatan",
          blocks: [
            {
              kind: "list",
              items: [
                "Halaman, panel obrolan di beranda, dan shader konturnya adalah chunk terpisah, dan build melaporkan ukuran gzip dari bagian yang dimuat pertama.",
                "Hanya teks antarmuka bahasa Inggris yang disertakan dalam chunk awal. Bahasa lain, beserta halaman Tentang tiap bahasa, adalah chunk terpisah yang dimuat saat dipilih.",
                "Pengujian performa seluler dengan throttling menargetkan LCP paling lama 2,5 s, interaksi paling lama 200 ms, dan CLS paling besar 0,1.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Lokalisasi",
          blocks: [
            {
              kind: "paragraph",
              text: "Antarmuka mendukung 24 bahasa, termasuk tata letak kanan-ke-kiri untuk bahasa Arab, Persia, dan Urdu. Katalog sekitar 740 kunci divalidasi terhadap registri backend, dan placeholder harus tetap utuh setelah diterjemahkan. Halaman Tentang ini adalah modul data bertipe, sehingga terjemahan tidak dapat diam-diam melewatkan bagian yang ada dalam bahasa Inggris.",
            },
          ],
        },
        {
          id: "design",
          heading: "Sistem desain",
          blocks: [
            {
              kind: "paragraph",
              text: "Tampilannya hangat dan teknis: kertas krem atau hitam pekat, garis tepi tipis, monospace untuk segala hal yang teknis, dan satu warna aksen pada satu waktu. Warna berupa variabel CSS semantik, sehingga komponen tidak memerlukan varian mode gelap. Preset aksen diuji kontrasnya terhadap latar halaman, warna RGB pilihan pengunjung disesuaikan di setiap mode agar tetap terbaca, dan mode terang atau gelap serta aksen diterapkan sebelum render pertama.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Animasi",
          blocks: [
            {
              kind: "list",
              items: [
                "Kemunculan bagian dan bilah progres membaca memakai animasi CSS berbasis gulir, yang berjalan di luar thread utama jika didukung.",
                "Statistik menghitung naik melalui bilangan bulat `@property` terdaftar, dengan nilai akhir tersedia di markup untuk teknologi bantu.",
                "Pergantian tema melebar dari tombol pengalih melalui View Transitions API.",
                "Bidang kontur di beranda adalah shader WebGL kecil yang dimuat setelah halaman menganggur, hanya berjalan di browser desktop yang mampu, membatasi frame rate, dan berhenti saat berada di luar layar.",
                "Pengaturan situs dapat mematikan semua ini. Animasi aktif secara bawaan dan tidak mengikuti pengaturan reduced-motion sistem operasi.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Stack teknologi",
      title: "Subsistem",
      lede: "Bagian-bagian situs yang lebih menarik daripada yang sebenarnya dibutuhkan sebuah blog.",
      sections: [
        {
          id: "live-chat",
          heading: "Obrolan dan panggilan langsung",
          blocks: [
            {
              kind: "paragraph",
              text: "Obrolan berjalan melalui WebSocket dengan codec biner ringkas, moderasi, dan riwayat terbatas. Panggilan suara dan video melewati Selective Forwarding Unit yang berjalan di dalam proses Axum menggunakan webrtc-rs: setiap browser memegang satu koneksi peer, dan SFU meneruskan media yang sudah dienkode tanpa transcoding, melalui kanal terbatas per publikasi dan batas jumlah peserta. [Buka obrolan langsung](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Penjelajah medan Minecraft",
          blocks: [
            {
              kind: "paragraph",
              text: "Plugin Paper membuka UNIX socket untuk pengguna yang sama guna kueri dunia yang terbatas, dengan satu operasi pada satu waktu dan batas ukuran frame yang ketat. [Penjelajah](/minecraft) menggambar ubin squaremap dengan klien Leaflet sendiri, bukan menjalankan JavaScript peta yang dihasilkan.",
            },
            {
              kind: "paragraph",
              text: "Lapisan bioma memakai **CYBM**, format ubin biner ringkas yang disajikan terkompresi gzip, atau PNG yang chunk privat `cyBM`-nya membawa identitas bioma yang tepat sehingga saat kursor diarahkan, hasilnya tidak ditebak dari warna piksel. Prediksi berbasis seed untuk medan yang belum dijelajahi berasal dari worker pembangkit dunia terpisah yang dibangun di atas Pumpkin.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Pencarian Geo-IP",
          blocks: [
            {
              kind: "paragraph",
              text: "[Papan pengunjung](/visitor-board) dan [pencarian IP](/geo-ip-db) menentukan lokasi alamat tanpa memanggil API eksternal. Alat praproses mengubah berkas CSV IP2Location LITE menjadi `BTreeMap` dengan kunci berupa awal rentang, mengenkodenya dengan bitcode, dan mengompresinya dengan zstd pada level 22.",
            },
            {
              kind: "table",
              columns: [
                "Dataset",
                "CSV",
                "Bundel",
              ],
              numericColumns: [
                1,
                2,
              ],
              rows: [
                [
                  "IPv4",
                  "312,5 MB",
                  "24,9 MB",
                ],
                [
                  "IPv6",
                  "794,2 MB",
                  "42,4 MB",
                ],
              ],
            },
            {
              kind: "paragraph",
              text: "Saat startup, server mendekode satu bundel pada satu waktu untuk membatasi puncak memori dan meng-intern nama tempat yang berulang. Pencarian menemukan rentang terakhir yang dimulai pada atau di bawah alamat tersebut, lalu memeriksa batas akhirnya.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "[Bundel demo](/projects) yang diunggah ditampung sementara di disk, dikenali dari magic byte-nya, dibatasi 50 MiB setelah didekompresi, dan disajikan dari cache terbatas di dalam frame ber-sandbox. Workspace ini juga membangun demo Rust, termasuk block breaker dan ray tracer.",
            },
            {
              kind: "paragraph",
              text: "[EU5 Location DB](/eu5-locations-db) adalah aplikasi Slint yang dikompilasi ke WebAssembly. Halaman mengirim mode terang atau gelapnya melalui protokol pesan kecil, sehingga palet yang tersemat berubah mengikuti situs.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Pipeline fotografi",
          blocks: [
            {
              kind: "paragraph",
              text: "Unggahan didekode dalam batas piksel tetap dan dienkode ke AVIF dengan preset paling lambat, menukar waktu pemrosesan dengan berkas yang lebih kecil. Lokasi dari data EXIF dapat dikonfirmasi di peta sebelum diterbitkan, dan tidak lebih dari dua pekerjaan pemrosesan berjalan sekaligus.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Stack teknologi",
      title: "Performa",
      lede: "Profile-guided optimization dan BOLT pada backend, diukur dengan generator beban yang bukan menjadi bottleneck.",
      sections: [
        {
          id: "approach",
          heading: "Pendekatan",
          blocks: [
            {
              kind: "paragraph",
              text: "Build yang dioptimalkan dilatih dengan beban kerja yang representatif. Binary berinstrumen merekam profil terhadap fixture sekali pakai dengan endpoint PostgreSQL, SMTP, S3, OpenID Connect, dan Minecraft sintetis. LLVM membangun ulang server dengan profil tersebut (PGO), lalu BOLT menyusun ulang tata letak kode binary akhir berdasarkan sampel eksekusi. Semua varian memakai LTO, frame pointer, dan pustaka standar yang dibangun ulang.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Penyiapan",
          blocks: [
            {
              kind: "list",
              items: [
                "Server, penyedia fixture, dan PostgreSQL dikunci pada empat inti AMD Ryzen 7 9800X3D; klien beban dikunci pada empat inti lainnya.",
                "HTTPS melalui HTTP/1.1 dengan connection pool berisi 32 koneksi, pembacaan respons secara penuh, dan batas waktu dua detik.",
                "Tiga putaran yang diacak urutannya dengan lima sampel per varian, masing-masing setelah 10.000 permintaan pemanasan, dari snapshot basis data baru.",
                "Total 27.000.000 permintaan yang diukur waktunya; semuanya mengembalikan HTTP 200.",
                "Beban berasal dari oha; Playwright memeriksa respons setiap rute di luar jendela pengukuran waktu.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Hasil",
          blocks: [
            {
              kind: "bars",
              caption: "Median permintaan per detik dari 15 sampel per varian.",
              groups: [
                {
                  label: "Pembacaan publik sepuluh rute",
                  items: [
                    {
                      label: "Garis dasar",
                      value: 44874.12,
                      display: "44.874",
                    },
                    {
                      label: "PGO",
                      value: 52925.46,
                      display: "52.925",
                    },
                    {
                      label: "PGO + BOLT",
                      value: 53888.61,
                      display: "53.889",
                      emphasis: true,
                    },
                  ],
                },
                {
                  label: "JSON health kecil",
                  items: [
                    {
                      label: "Garis dasar",
                      value: 272837.41,
                      display: "272.837",
                    },
                    {
                      label: "PGO",
                      value: 308685.26,
                      display: "308.685",
                      emphasis: true,
                    },
                    {
                      label: "PGO + BOLT",
                      value: 306276.64,
                      display: "306.277",
                    },
                  ],
                },
              ],
            },
            {
              kind: "table",
              caption: "Latensi ekor untuk campuran sepuluh rute.",
              columns: [
                "Varian",
                "p95 (ms)",
                "p99 (ms)",
              ],
              numericColumns: [
                1,
                2,
              ],
              rows: [
                [
                  "Garis dasar",
                  "2,380",
                  "2,967",
                ],
                [
                  "PGO",
                  "2,020",
                  "2,524",
                ],
                [
                  "PGO + BOLT",
                  "1,987",
                  "2,483",
                ],
              ],
            },
            {
              kind: "paragraph",
              text: "PGO meningkatkan throughput campuran sebesar **17,9%** dan PGO dengan BOLT sebesar **20,1%**, dan kedua build yang dioptimalkan menang di setiap putaran yang diacak. BOLT menambah 1,8% di atas PGO pada pembacaan campuran dan 0,8% lebih lambat pada rute health yang sepele. Ini adalah hasil pada empat inti; kapasitas produksi juga bergantung pada jaringan, working set basis data, dan lalu lintas nyata.",
            },
          ],
        },
        {
          id: "corrections",
          heading: "Koreksi",
          blocks: [
            {
              kind: "list",
              items: [
                "`TCP_NODELAY` awalnya tidak diaktifkan: acceptor Rustls bawaan axum-server membiarkannya mati, sehingga respons HTTP/2 bisa menunggu sekitar 40 ms akibat delayed acknowledgement. Kini setiap socket yang diterima mengaktifkannya.",
                "Benchmark pertama memberi beban dari Playwright, yang kliennya melambat seiring berjalannya pengujian, sehingga keliru menunjukkan regresi. Angka-angka itu dibuang.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
