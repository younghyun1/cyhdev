import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "Tháng 11 năm 2026",
  chrome: {
    overview: "Tổng quan",
    updated: "Cập nhật lần cuối",
    onThisPage: "Trong trang này",
    sectionNavigation: "Các trang về công nghệ",
    previous: "Trước",
    next: "Tiếp",
    readingTime: "{minutes} phút đọc",
  },
  pages: {
    overview: {
      eyebrow: "Về trang web này",
      title: "Công nghệ của blog",
      lede: "Một binary Rust, một cơ sở dữ liệu PostgreSQL, một máy nhỏ. Các trang này mô tả cách trang web được xây dựng, lưu trữ, bảo mật và đo lường, cùng lý do nó có hình dạng như vậy.",
      sections: [
        {
          id: "at-a-glance",
          heading: "Tóm tắt nhanh",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53.889 req/s",
                  label: "Thông lượng trung vị khi đọc công khai hỗn hợp, dùng PGO và BOLT trên bốn lõi máy chủ",
                },
                {
                  value: "~150 µs",
                  label: "Một lượt truy vấn cơ sở dữ liệu qua UNIX socket đo được, so với ~600 µs qua TCP localhost",
                },
                {
                  value: "0",
                  label: "Tiến trình Node.js trong môi trường production; frontend được nhúng vào binary của máy chủ",
                },
                {
                  value: "24",
                  count: 24,
                  label: "Ngôn ngữ giao diện, mỗi ngôn ngữ chỉ được tải khi được chọn",
                },
                {
                  value: "4.096",
                  label: "Kết nối mở được chấp nhận trước TLS, tối đa 64 kết nối cho mỗi địa chỉ hoặc IPv6 /64",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Tiến trình phục vụ TLS, API, WebSocket, dữ liệu đa phương tiện WebRTC và frontend được nhúng",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Đường đi của yêu cầu",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Trình duyệt",
                  detail: "HTTP/2 hoặc HTTP/1.1 qua TLS. Frontend được lưu đệm dưới các URL bất biến có dấu vân tay.",
                },
                {
                  label: "Tiếp nhận",
                  detail: "Kết nối TCP được chấp nhận trước TLS: tổng cộng 4.096 và 64 cho mỗi địa chỉ hoặc IPv6 /64.",
                },
                {
                  label: "rustls",
                  detail: "TLS kết thúc ngay trong tiến trình Rust, với `TCP_NODELAY` trên mọi socket.",
                },
                {
                  label: "Axum",
                  detail: "Header bảo mật, phiên, giới hạn tần suất, nén và giới hạn kích thước body, rồi đến handler của tính năng.",
                },
                {
                  label: "Bộ nhớ đệm · Diesel",
                  detail: "Các bộ nhớ đệm trong bộ nhớ có giới hạn xử lý những lượt đọc nóng; mọi thứ còn lại đi qua Diesel bất đồng bộ.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "Trên cùng máy, qua UNIX socket, với I/O bất đồng bộ io_uring.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Các chương",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Hạ tầng",
                  href: "/about-blog/infrastructure",
                  meta: "Máy chủ · Gentoo · mạng",
                  text: "Chiếc máy, hệ điều hành và hệ thống tệp, mạng, và lý do không có reverse proxy.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "Các thành phần dịch vụ, mô hình dữ liệu, giới hạn khi chạy, header bảo mật và đặc tả API.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "Khung ứng dụng, cách tải, bản địa hóa, hệ thống thiết kế và chuyển động.",
                },
                {
                  title: "Hệ thống con",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "Chat trực tiếp với SFU chạy trong cùng tiến trình, trình khám phá Minecraft, tra cứu Geo-IP và WebAssembly.",
                },
                {
                  title: "Hiệu năng",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Các bản build tối ưu, cách đo và kết quả.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Cách kiểm tra thay đổi",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy trên các target native và WebAssembly làm cổng kiểm tra khi triển khai.",
                "Các lệnh cargo xtask chạy cục bộ để kiểm tra định dạng, unit test native, sai lệch đặc tả OpenAPI, cùng typecheck, lint, unit test và build của frontend.",
                "Playwright trên Chromium và WebKit, kèm kiểm tra khả năng truy cập bằng axe và kiểm tra bố cục di động xuống đến 320 điểm ảnh CSS.",
                "Test tích hợp và test rollback migration trên một instance PostgreSQL dùng một lần, cùng quét secret cục bộ bằng gitleaks.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Công nghệ sử dụng",
      title: "Hạ tầng",
      lede: "Một chiếc máy nhỏ ở nhà, một bản phân phối Linux build từ mã nguồn, và không có lớp nào mà lưu lượng không cần đến.",
      sections: [
        {
          id: "host",
          heading: "Máy chủ vật lý",
          blocks: [
            {
              kind: "paragraph",
              text: "Trang web chạy trên một [miniserver](https://store.minisforum.com/products/minisforum-um690l-slim) tại nhà tôi: bộ xử lý di động Ryzen 9 tám lõi có thể tăng xung đến 4,9 GHz, 32 GB RAM và SSD NVMe 1 TB. Với giá khoảng $400, nó rẻ hơn chi phí thuê phần cứng đám mây yếu hơn rõ rệt trong chưa đến một năm, và nó thể hiện niềm đam mê tự lưu trữ của tôi.",
            },
            {
              kind: "paragraph",
              text: "Máy này chạy máy chủ gộp backend và frontend, PostgreSQL và một máy chủ Minecraft. Nếu bạn muốn chơi, hãy gửi email cho tôi.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo và btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "[Gentoo](https://www.gentoo.org/) không phải lựa chọn hiển nhiên cho máy chủ; ở chỗ làm tôi sẽ chọn Debian stable trên ext4. Ở đây tôi thích tự biên dịch các gói với tối ưu hóa cho đúng bộ xử lý này.",
            },
            {
              kind: "paragraph",
              text: "[btrfs](https://en.wikipedia.org/wiki/Btrfs) mang lại snapshot, nén và copy-on-write, nhưng copy-on-write làm phân mảnh tệp cơ sở dữ liệu rất nặng. Vì vậy các thư mục dữ liệu của PostgreSQL và Minecraft được loại khỏi nó. Một [benchmark hệ thống tệp](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) cho thấy btrfs mặc định thua ext4 và XFS; tôi đoán việc tắt copy-on-write thu hẹp phần lớn khoảng cách đó, và đây sẽ là một phép đo thú vị.",
            },
          ],
        },
        {
          id: "network",
          heading: "Mạng",
          blocks: [
            {
              kind: "paragraph",
              text: "Lưu lượng đến qua đường truyền Xfinity có dây 1 Gbps; modem của nhà mạng không có Ethernet 2,5 Gb, điều mà trang web này sẽ không nhận ra. Route 53 lo phần DNS.",
            },
            {
              kind: "paragraph",
              text: "Bên trong máy không có reverse proxy, container runtime hay service mesh. PostgreSQL chạy thẳng trên hệ điều hành, và một binary Rust phục vụ cả frontend lẫn API. Các yêu cầu HTTP không mã hóa được chính tiến trình đó chuyển hướng sang HTTPS.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "UNIX socket",
          blocks: [
            {
              kind: "paragraph",
              text: "Máy chủ kết nối tới PostgreSQL qua UNIX socket thay vì TCP localhost, nhờ đó bỏ qua hoàn toàn ngăn xếp mạng. [Các benchmark](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) cho thấy tốc độ giao dịch gần như tăng gấp đôi; ở đây tôi đã quan sát thấy thời gian khứ hồi thấp tới **150 µs**, so với khoảng **600 µs** qua localhost. Cách này cũ kỹ và không liên quan đến phần lớn các hệ thống triển khai trên đám mây, nhưng nó nhanh hơn.",
            },
            {
              kind: "paragraph",
              text: "PostgreSQL 18, phát hành tháng 9 năm 2025, bổ sung [I/O bất đồng bộ](https://pganalyze.com/blog/postgres-18-async-io), và io_uring được bật ở đây, giúp tăng tốc đọc rõ rệt.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Build và triển khai",
          blocks: [
            {
              kind: "list",
              items: [
                "Bản build frontend được nhúng vào binary backend cùng các bản nén sẵn bằng zstd và gzip, nên không có tiến trình Node.js nào chạy trong production.",
                "Các bản build tối ưu có thể được huấn luyện bằng tối ưu hóa theo profile và tối ưu hóa sau liên kết bằng BOLT; xem [Hiệu năng](/about-blog/performance).",
                "Thông tin xác thực, chứng chỉ và dữ liệu khi chạy không nằm trong Git, và một lần quét gitleaks kiểm tra cây mã nguồn trước khi thay đổi rời khỏi máy.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Công nghệ sử dụng",
      title: "Backend",
      lede: "Rust trên Axum và Tokio, PostgreSQL qua Diesel bất đồng bộ, và giới hạn tường minh cho mọi thứ mà client có thể giữ mở.",
      sections: [
        {
          id: "stack",
          heading: "Công nghệ",
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
                  label: "Dữ liệu",
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
                  label: "Tích hợp",
                  items: [
                    "openidconnect",
                    "lettre",
                    "Amazon S3",
                    "webrtc-rs",
                    "image + AVIF",
                  ],
                },
                {
                  label: "Đặc tả",
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
              text: "Dịch vụ là một tiến trình duy nhất với trạng thái dùng chung: phiên trong bộ nhớ, đồng bộ bộ nhớ đệm khi khởi động và các tác vụ bảo trì theo lịch. mimalloc là bộ cấp phát toàn cục; nếu máy có nhiều lõi thật, hành vi của bộ cấp phát đáng được xem xét nghiêm túc.",
            },
          ],
        },
        {
          id: "data",
          heading: "Mô hình dữ liệu",
          blocks: [
            {
              kind: "paragraph",
              text: "Trong các công việc ở startup và doanh nghiệp Hàn Quốc mà tôi từng thấy, MySQL và MariaDB được coi là những cơ sở dữ liệu quan hệ nghiêm túc duy nhất. PostgreSQL từ lâu đã vượt qua chúng ở những mặt tôi quan tâm, từ UUID gốc đến JSON nhị phân.",
            },
            {
              kind: "list",
              items: [
                "**Bài viết** có slug, tóm tắt, metadata JSONB, trạng thái xuất bản, bộ đếm phi chuẩn hóa và ánh xạ thẻ. Bình luận được lồng nhau qua một khóa cha có thể null, còn lượt bình chọn bài viết và bình luận nằm ở các bảng riêng.",
                "**Tài khoản** lưu quốc gia và ngôn ngữ, các bảng vai trò và quyền, cùng ảnh đại diện có phiên bản trong bảng riêng thay vì trên dòng người dùng được truy cập thường xuyên.",
                "**Khóa** là các giá trị UUIDv7 sắp theo thời gian: không thể liệt kê như ID tuần tự, và thân thiện với tính cục bộ của B-tree hơn nhiều so với UUIDv4 ngẫu nhiên.",
                "**Tìm kiếm** dùng chỉ mục tantivy lưu trên đĩa cho tiêu đề và thẻ của bài viết, được giữ nhất quán với PostgreSQL, vốn vẫn là nguồn chuẩn.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Giới hạn khi chạy",
          blocks: [
            {
              kind: "list",
              items: [
                "Kết nối được chấp nhận trước TLS thông qua một bộ giới hạn dùng chung: 4.096 kết nối mở và 64 cho mỗi địa chỉ IPv4 hoặc IPv6 /64. Các socket bị từ chối sẽ đóng mà không thực hiện handshake.",
                "Phần đầu yêu cầu HTTP/1 phải đến trong vòng 20 giây; kết nối HTTP/2 được ping mỗi 30 giây và cho phép 128 luồng đồng thời.",
                "Mỗi bộ nhớ đệm có giới hạn cố định và một nguồn chuẩn để nạp lại khi thiếu; không bộ nào tăng theo các khóa do client kiểm soát. Chẳng hạn, bộ nhớ đệm bundle WebAssembly chứa tối đa 256 MiB.",
                "Công việc có thể chặn luồng, như mã hóa ảnh, chạy trên các nhóm worker có giới hạn thay vì trên worker của Tokio.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Header bảo mật",
          blocks: [
            {
              kind: "paragraph",
              text: "Content-Security-Policy được dựng một lần khi khởi động. Script nội tuyến chỉ được cho phép theo mã băm SHA-256 tính từ `index.html` được biên dịch vào binary, nên script giao diện chạy trước khi vẽ không thể lệch khỏi chính sách. Origin chính không bao giờ khởi tạo WebAssembly: các bản demo và ứng dụng EU5 chạy trong các frame cô lập với chính sách riêng.",
            },
            {
              kind: "paragraph",
              text: "Cookie phiên là `Secure` và `HttpOnly`, các thao tác của superuser kiểm tra vai trò trên máy chủ, tài khoản có thể liên kết danh tính OpenID Connect, và Swagger UI yêu cầu xác thực trong production.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Đặc tả API và tài nguyên tĩnh",
          blocks: [
            {
              kind: "list",
              items: [
                "Các handler mang chú thích OpenAPI, client TypeScript của frontend được sinh từ đó, và cargo xtask openapi báo lỗi khi các tệp được sinh bị lệch.",
                "Manifest của Vite được nhúng cùng các tài nguyên. Chỉ những tệp có dấu vân tay và được liệt kê trong manifest mới nhận chính sách lưu đệm `immutable` một năm; HTML luôn được kiểm tra lại.",
                "Đường dẫn tài nguyên không tồn tại trả về 404 với `no-store` thay vì khung ứng dụng, để một trang cũ không thể lưu đệm HTML dưới URL của script.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Công nghệ sử dụng",
      title: "Frontend",
      lede: "Một ứng dụng SolidJS 2 render phía client, được biên dịch thành tài nguyên tĩnh và nhúng vào binary của máy chủ.",
      sections: [
        {
          id: "stack",
          heading: "Công nghệ",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Ứng dụng",
                  items: [
                    "SolidJS 2",
                    "Solid Router 2",
                    "TypeScript",
                    "Vite 8",
                  ],
                },
                {
                  label: "Tạo kiểu",
                  items: [
                    "Tailwind CSS 4",
                    "IBM Plex Sans",
                    "IBM Plex Mono",
                    "OKLab",
                  ],
                },
                {
                  label: "Thành phần",
                  items: [
                    "Toast UI Editor",
                    "Leaflet",
                    "Chart.js",
                    "highlight.js",
                    "DOMPurify",
                  ],
                },
                {
                  label: "Kiểm thử",
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
              text: "Các signal chi tiết của Solid chỉ cập nhật những nút DOM phụ thuộc vào giá trị đã thay đổi, và các bản release candidate 2.0 cho phép thao tác đọc bất đồng bộ tạm dừng đến ranh giới tải gần nhất. Router tải từng trang theo yêu cầu, và môi trường production không có máy chủ JavaScript: binary Rust phục vụ trực tiếp các tệp tĩnh.",
            },
          ],
        },
        {
          id: "loading",
          heading: "Tải trang",
          blocks: [
            {
              kind: "list",
              items: [
                "Các trang, khung chat ở trang chủ và shader đường đồng mức của trang đó là những chunk riêng, và bản build báo cáo dung lượng gzip của phần được tải đầu tiên.",
                "Chỉ văn bản giao diện tiếng Anh nằm trong chunk ban đầu. Các ngôn ngữ khác, cùng các trang Giới thiệu của từng ngôn ngữ, là những chunk riêng và được tải khi người dùng chọn.",
                "Các bài kiểm tra hiệu năng trên thiết bị di động bị giới hạn tốc độ đặt mục tiêu LCP không quá 2,5 s, tương tác không quá 200 ms và CLS không quá 0,1.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Bản địa hóa",
          blocks: [
            {
              kind: "paragraph",
              text: "Giao diện hỗ trợ 24 ngôn ngữ, gồm cả bố cục từ phải sang trái cho tiếng Ả Rập, tiếng Ba Tư và tiếng Urdu. Một danh mục khoảng 740 khóa được đối chiếu với registry của backend, và các placeholder phải được giữ nguyên khi dịch. Các trang Giới thiệu này là những module dữ liệu có kiểu, nên bản dịch không thể âm thầm bỏ sót một mục mà bản tiếng Anh có.",
            },
          ],
        },
        {
          id: "design",
          heading: "Hệ thống thiết kế",
          blocks: [
            {
              kind: "paragraph",
              text: "Giao diện mang cảm giác ấm và kỹ thuật: giấy màu kem hoặc nền đen tuyền, đường viền mảnh, phông monospace cho mọi nội dung kỹ thuật, và mỗi lần chỉ dùng một màu nhấn. Màu sắc là các biến CSS theo ngữ nghĩa, nên các component không cần biến thể riêng cho chế độ tối. Các màu nhấn có sẵn được kiểm tra độ tương phản với nền trang, màu RGB riêng của khách truy cập được điều chỉnh trong từng chế độ để vẫn dễ đọc, và chế độ sáng hoặc tối cùng màu nhấn được áp dụng trước lần hiển thị đầu tiên.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Chuyển động",
          blocks: [
            {
              kind: "list",
              items: [
                "Hiệu ứng hiện dần của các mục và thanh tiến độ đọc dùng animation CSS điều khiển bằng cuộn trang, chạy ngoài luồng chính ở nơi được hỗ trợ.",
                "Các số thống kê đếm lên thông qua một số nguyên `@property` đã đăng ký, với giá trị cuối cùng có sẵn trong markup cho công nghệ hỗ trợ.",
                "Nút chuyển giao diện sáng tối lan rộng ra từ chính nút bấm thông qua View Transitions API.",
                "Trường đường đồng mức ở trang chủ là một shader WebGL nhỏ, được tải sau khi trang rảnh, chỉ chạy trên trình duyệt desktop đủ khả năng, giới hạn tốc độ khung hình và tạm dừng khi ra khỏi màn hình.",
                "Một tùy chọn của trang web tắt toàn bộ những hiệu ứng này. Hiệu ứng mặc định là bật và không theo cài đặt giảm chuyển động của hệ điều hành.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Công nghệ sử dụng",
      title: "Hệ thống con",
      lede: "Những phần của trang web thú vị hơn mức một blog thực sự cần.",
      sections: [
        {
          id: "live-chat",
          heading: "Chat trực tiếp và cuộc gọi",
          blocks: [
            {
              kind: "paragraph",
              text: "Chat chạy trên WebSocket với codec nhị phân gọn nhẹ, có kiểm duyệt và lịch sử giới hạn dung lượng. Cuộc gọi thoại và video đi qua một Selective Forwarding Unit chạy ngay trong tiến trình Axum trên webrtc-rs: mỗi trình duyệt giữ một kết nối peer, và SFU chuyển tiếp dữ liệu đa phương tiện đã mã hóa mà không chuyển mã, qua các kênh giới hạn cho từng luồng phát và có giới hạn số người tham gia. [Mở chat trực tiếp](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Trình khám phá địa hình Minecraft",
          blocks: [
            {
              kind: "paragraph",
              text: "Một plugin Paper mở socket UNIX cho cùng người dùng để truy vấn thế giới có giới hạn, mỗi lần một thao tác và với giới hạn kích thước khung nghiêm ngặt. [Trình khám phá](/minecraft) vẽ các ô squaremap bằng client Leaflet riêng thay vì chạy JavaScript của bản đồ được tạo sẵn.",
            },
            {
              kind: "paragraph",
              text: "Lớp phủ biome dùng **CYBM**, một định dạng ô nhị phân gọn nhẹ được phục vụ ở dạng nén gzip, hoặc một tệp PNG có chunk riêng `cyBM` chứa chính xác định danh biome, nên khi di chuột không bao giờ phải đoán từ màu điểm ảnh. Các dự đoán dựa trên seed cho vùng đất chưa khám phá đến từ một worker tạo thế giới riêng, xây dựng trên Pumpkin.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Tra cứu Geo-IP",
          blocks: [
            {
              kind: "paragraph",
              text: "[Bảng khách truy cập](/visitor-board) và [tra cứu IP](/geo-ip-db) xác định vị trí địa chỉ mà không gọi API bên ngoài. Một công cụ tiền xử lý chuyển các tệp CSV IP2Location LITE thành các `BTreeMap` khóa theo điểm bắt đầu của dải, mã hóa bằng bitcode và nén bằng zstd ở mức 22.",
            },
            {
              kind: "table",
              columns: [
                "Tập dữ liệu",
                "CSV",
                "Gói",
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
              text: "Khi khởi động, máy chủ giải mã từng gói một để hạn chế mức bộ nhớ đỉnh và dùng chung các tên địa danh lặp lại. Một lần tra cứu tìm dải cuối cùng bắt đầu tại hoặc dưới địa chỉ đó rồi kiểm tra cận cuối của dải.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "Các [gói demo](/projects) được tải lên sẽ được lưu tạm xuống đĩa, nhận diện bằng magic byte, giới hạn 50 MiB sau khi giải nén, và phục vụ từ một bộ nhớ đệm có giới hạn bên trong các khung sandbox. Workspace cũng build các demo Rust, gồm trò phá gạch và một ray tracer.",
            },
            {
              kind: "paragraph",
              text: "[EU5 Location DB](/eu5-locations-db) là một ứng dụng Slint được biên dịch sang WebAssembly. Trang gửi chế độ sáng hoặc tối của mình qua một giao thức thông điệp nhỏ, nên bảng màu nhúng thay đổi theo trang web.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Quy trình xử lý ảnh",
          blocks: [
            {
              kind: "paragraph",
              text: "Ảnh tải lên được giải mã trong giới hạn điểm ảnh cố định và mã hóa sang AVIF ở preset chậm nhất, đánh đổi thời gian xử lý lấy tệp nhỏ hơn. Vị trí lấy từ dữ liệu EXIF có thể được xác nhận trên bản đồ trước khi xuất bản, và có tối đa hai tác vụ xử lý chạy cùng lúc.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Công nghệ sử dụng",
      title: "Hiệu năng",
      lede: "Tối ưu hóa theo hồ sơ chạy thực tế (PGO) và BOLT cho backend, được đo bằng bộ tạo tải không phải là điểm nghẽn.",
      sections: [
        {
          id: "approach",
          heading: "Phương pháp",
          blocks: [
            {
              kind: "paragraph",
              text: "Các bản build tối ưu được huấn luyện trên một khối lượng công việc đại diện. Một tệp nhị phân có gắn công cụ đo ghi lại hồ sơ chạy trên một fixture dùng một lần, với các điểm cuối giả lập cho PostgreSQL, SMTP, S3, OpenID Connect và Minecraft. LLVM build lại máy chủ với các hồ sơ đó (PGO), sau đó BOLT sắp xếp lại bố cục mã của tệp nhị phân cuối cùng dựa trên dữ liệu lấy mẫu khi thực thi. Mọi biến thể đều dùng chung LTO, frame pointer và thư viện chuẩn được build lại.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Thiết lập",
          blocks: [
            {
              kind: "list",
              items: [
                "Máy chủ, các nhà cung cấp fixture và PostgreSQL được gắn cố định vào bốn lõi của AMD Ryzen 7 9800X3D; máy khách tạo tải được gắn vào bốn lõi còn lại.",
                "HTTPS qua HTTP/1.1 có gom nhóm kết nối với 32 kết nối, đọc toàn bộ phản hồi và thời hạn chờ hai giây.",
                "Ba vòng luân phiên, mỗi biến thể năm mẫu, mỗi mẫu chạy sau 10.000 yêu cầu khởi động, từ một bản chụp cơ sở dữ liệu mới.",
                "Tổng cộng 27.000.000 yêu cầu được tính giờ; tất cả đều trả về HTTP 200.",
                "Tải được tạo bằng oha; Playwright kiểm tra phản hồi của từng route ngoài khoảng thời gian đo.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Kết quả",
          blocks: [
            {
              kind: "bars",
              caption: "Số yêu cầu mỗi giây ở trung vị trên 15 mẫu cho mỗi biến thể.",
              groups: [
                {
                  label: "Đọc công khai mười route",
                  items: [
                    {
                      label: "Mức chuẩn",
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
                  label: "JSON kiểm tra tình trạng nhỏ",
                  items: [
                    {
                      label: "Mức chuẩn",
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
              caption: "Độ trễ đuôi cho hỗn hợp mười route.",
              columns: [
                "Biến thể",
                "p95 (ms)",
                "p99 (ms)",
              ],
              numericColumns: [
                1,
                2,
              ],
              rows: [
                [
                  "Mức chuẩn",
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
              text: "PGO tăng thông lượng hỗn hợp thêm **17,9%** và PGO kết hợp BOLT tăng thêm **20,1%**, và cả hai bản build tối ưu đều thắng ở mọi vòng luân phiên. BOLT cải thiện thêm 1,8% so với PGO ở các lần đọc hỗn hợp và chậm hơn 0,8% ở route kiểm tra tình trạng đơn giản. Đây là kết quả trên bốn lõi; năng lực thực tế khi vận hành còn phụ thuộc vào mạng, tập dữ liệu làm việc của cơ sở dữ liệu và lưu lượng thật.",
            },
          ],
        },
        {
          id: "corrections",
          heading: "Đính chính",
          blocks: [
            {
              kind: "list",
              items: [
                "Ban đầu `TCP_NODELAY` chưa được đặt: bộ chấp nhận Rustls mặc định của axum-server để tùy chọn này tắt, nên phản hồi HTTP/2 có thể phải chờ khoảng 40 ms vì xác nhận bị trì hoãn. Giờ mọi socket được chấp nhận đều đặt tùy chọn này.",
                "Benchmark đầu tiên tạo tải từ Playwright, mà máy khách của nó chậm dần trong suốt một lần chạy, nên cho thấy các đợt suy giảm hiệu năng một cách sai lệch. Những số liệu đó đã bị loại bỏ.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
