import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "Tháng 11 năm 2026",
  chrome: {
    overview: "Tổng quan",
    updated: "Cập nhật lần cuối",
    onThisPage: "Trong trang này",
    sectionNavigation: "Các trang giới thiệu",
    previous: "Trước",
    next: "Tiếp",
    readingTime: "{minutes} phút đọc",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "Giới thiệu",
      lede: "Kỹ sư backend và hạ tầng làm việc với Rust, PostgreSQL và các nền tảng đám mây. Tôi xây dựng các dịch vụ tận dụng hiệu quả phần cứng bên dưới, cùng với công việc triển khai, giám sát và bảo mật giúp chúng vận hành ổn định.",
      sections: [
        {
          id: "introduction",
          heading: "Giới thiệu bản thân",
          blocks: [
            {
              kind: "paragraph",
              text: "Tôi là kỹ sư phần mềm người Hàn Quốc, sống tại Colorado. Tại **Soundpatrol**, tôi làm kỹ sư Rust phụ trách an ninh mạng, DevOps, dịch vụ web và công cụ cho lập trình viên, chủ yếu với Rust, Python, PostgreSQL, Kubernetes và Google Cloud.",
            },
            {
              kind: "paragraph",
              text: "Trước đó, tôi dẫn dắt phần backend và hạ tầng cho một bản sao số (digital twin) của tòa nhà Samsung C&T, đóng góp vào backend ứng dụng của Hyundai, Kia và Genesis, và xây dựng phần thu thập dữ liệu cho một nền tảng phân tích YouTube. Từ năm 2023, công việc của tôi tập trung vào các dịch vụ backend, pipeline dữ liệu và hạ tầng bên dưới chúng.",
            },
            {
              kind: "paragraph",
              text: "Tôi tin vào việc xây dựng các dịch vụ tận dụng hiệu quả phần cứng và ngôn ngữ lập trình hiện đại. Trong thực tế, điều đó nghĩa là thiết kế lược đồ cẩn thận, hàng đợi và bộ nhớ đệm có giới hạn, xử lý lỗi tường minh, và các bản triển khai có thể kiểm tra và khôi phục. Tôi thích giúp đồng nghiệp làm quen với Rust, và tôi muốn làm việc với những người tự hào về nghề của mình.",
            },
          ],
        },
        {
          id: "results",
          heading: "Kết quả tiêu biểu",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "Giảm chi phí đám mây hằng tháng của bản sao số Samsung C&T, từ khoảng $5.000 xuống $150",
                },
                {
                  value: "80",
                  count: 80,
                  label: "Endpoint REST được triển khai cho bản sao số, kèm logic nghiệp vụ của ngành xây dựng",
                },
                {
                  value: "9 tháng",
                  count: 9,
                  suffix: " tháng",
                  label: "Bản sao số vận hành trên môi trường production mà không có lần dừng nào khi chạy",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "Thông lượng đọc hỗn hợp tăng thêm trên backend của trang này nhờ [PGO và BOLT](/about-blog/performance)",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Container dịch vụ liên kết tĩnh, tạo ra từ các bản build nhiều giai đoạn",
                },
                {
                  value: "30.000",
                  label: "Dòng mã tôi viết cho backend của bản sao số, khoảng 75% toàn bộ mã nguồn",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Bộ công cụ",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Ngôn ngữ",
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
                  label: "Dữ liệu",
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
                  label: "Hạ tầng",
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
                  label: "Độ tin cậy và bảo mật",
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
                  label: "Giao diện và thiết bị",
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
          heading: "Thêm về tôi",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Kinh nghiệm",
                  href: "/about/experience",
                  meta: "2016 - nay",
                  text: "Các vai trò từ Soundpatrol ngược về kỳ thực tập đầu tiên, cùng các dự án và kết quả của từng vai trò.",
                },
                {
                  title: "Dự án",
                  href: "/about/projects",
                  meta: "Rust · mã nguồn mở",
                  text: "Sillok, EU5 Location DB, trang web này và các dự án độc lập, mã nguồn mở khác.",
                },
                {
                  title: "Lý lịch",
                  href: "/about/background",
                  meta: "Học vấn · nghiên cứu",
                  text: "Học vấn, nghiên cứu đã công bố, giải thưởng, ngôn ngữ và công việc ngoài lĩnh vực phần mềm.",
                },
                {
                  title: "Trang này hoạt động thế nào",
                  href: "/about-blog",
                  meta: "Công nghệ sử dụng",
                  text: "Máy chủ, backend, frontend, các hệ thống con và số đo hiệu năng đứng sau cyhdev.com.",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "Liên hệ",
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
              text: "Email là cách đáng tin cậy nhất để liên lạc với tôi. Nếu bạn muốn tham gia máy chủ Minecraft, hãy nói rõ trong email đó.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "Giới thiệu",
      title: "Kinh nghiệm",
      lede: "Công việc backend, hạ tầng và bảo mật tại một startup Mỹ, cho các khách hàng doanh nghiệp Hàn Quốc và trong kỳ thực tập đầu tiên, tiếp theo là những công việc trước khi tôi làm phần mềm.",
      sections: [
        {
          id: "software",
          heading: "Kỹ thuật phần mềm",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Kỹ sư Rust",
                  period: "Tháng 5/2026 - nay",
                  location: "Hoa Kỳ · Làm từ xa",
                  summary: "An ninh mạng, DevOps, phát triển dịch vụ web và công cụ cho lập trình viên tại một công ty nhận diện nhạc và âm thanh.",
                  highlights: [
                    "Duy trì hạ tầng Kubernetes trên Google Cloud, hạ tầng dưới dạng mã và quy trình triển khai có xét duyệt, dùng thông tin xác thực ngắn hạn theo từng môi trường.",
                    "Lập kế hoạch và thực hiện di chuyển kho dữ liệu production, kiểm tra trạng thái sau di chuyển và diễn tập khôi phục trước mỗi lần chuyển đổi.",
                    "Hợp nhất metric, log và trace trên OpenTelemetry, đồng thời xây dựng công cụ Rust để theo dõi tình trạng dịch vụ, chẩn đoán và chi phí.",
                    "Làm việc về mức sẵn sàng bảo mật: quản trị truy cập, xoay vòng và theo dõi hạn của thông tin xác thực, chính sách, ứng phó sự cố và bằng chứng kiểm toán.",
                    "Xây dựng các dịch vụ và công cụ nội bộ bằng Rust và Python cho quy trình vận hành và hỗ trợ.",
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
                  title: "Kỹ sư phần mềm",
                  period: "Tháng 1/2025 - Tháng 7/2025",
                  location: "Seongnam, Hàn Quốc",
                  summary: "Dịch vụ backend và công cụ hạ tầng cho các nền tảng nội bộ và ứng dụng fan chính thức của một nhóm nhạc K-pop.",
                  highlights: [
                    "Đóng góp API, bản sửa lỗi và tích hợp cho backend ứng dụng chính thức của Hyundai, Kia và Genesis, làm việc trong một codebase Java lớn và phối hợp với các nhà cung cấp cho những dịch vụ phục vụ hàng chục triệu người ở châu Á và châu Âu.",
                    "Xây dựng pipeline quốc tế hóa bằng Rust và Python để chuyển đổi và nạp nội dung đã dịch vào cơ sở dữ liệu của công ty, phối hợp với các quản trị viên cơ sở dữ liệu phụ trách những bộ dữ liệu kết nối với hệ thống nhà máy.",
                    "Phát triển công cụ quản lý hạ tầng cho các dịch vụ nội bộ.",
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
                  title: "Kỹ sư phần mềm hợp đồng",
                  period: "Tháng 11/2024 - Tháng 12/2024",
                  location: "Seoul, Hàn Quốc",
                  highlights: [
                    "Thiết kế lược đồ PostgreSQL và backend Rust cho một nền tảng phân tích YouTube có hỗ trợ AI, và dẫn dắt việc triển khai trên AWS và Google Cloud.",
                    "Xây dựng quy trình thu thập qua YouTube API bao phủ hàng chục triệu bình luận và người dùng, kết hợp dữ liệu kênh, bản chép lời và nội dung trang.",
                    "Tích hợp các mô hình ngôn ngữ dạng dịch vụ và các mô hình mã nguồn mở chạy trên máy chủ để tạo bản tóm tắt và thông tin chi tiết cho tiếp thị.",
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
                  title: "Kỹ sư phần mềm backend (Trưởng nhóm)",
                  period: "Tháng 8/2023 - Tháng 8/2024",
                  location: "Seoul, Hàn Quốc",
                  summary: "Dẫn dắt phát triển backend, cơ sở dữ liệu và hạ tầng đám mây cho hai nền tảng quản lý tòa nhà và năng lượng.",
                  highlights: [],
                  projects: [
                    {
                      name: "Bản sao số của Samsung C&T Global Engineering Center",
                      highlights: [
                        "Thay một backend Express và hệ microservice trên đám mây bằng một monolith Rust/Axum đã tối ưu, giảm chi phí hạ tầng hằng tháng từ khoảng **$5.000 xuống $150**.",
                        "Viết khoảng **30.000 dòng**, xấp xỉ 75% backend, gồm **80 endpoint REST** chứa logic nghiệp vụ ngành xây dựng, tích hợp dữ liệu và các thao tác thời gian thực.",
                        "Đưa nhiều đường xử lý yêu cầu có P99 vài giây xuống còn một hoặc hai chữ số mili giây trên bản triển khai hai nhân, 4 GB, nhờ Rust bất đồng bộ, đa luồng, bộ nhớ đệm và tinh chỉnh lược đồ cùng truy vấn PostgreSQL.",
                        "Tích hợp vài nghìn cảm biến, đồng hồ đo và bộ điều khiển hệ thống HVAC và cấp nước qua BACnet và Modbus, ngay tại trụ sở Samsung C&T.",
                        "Vận hành chín tháng trên production mà không có lần dừng nào khi chạy, với độ phủ xử lý lỗi khoảng 90%.",
                        "Xây dựng CI/CD với GitHub runner, AWS CodeDeploy và Docker; các bản build tĩnh, tinh chỉnh theo bộ xử lý tạo ra container di động khoảng 20 MB.",
                        "Kết nối suy luận của mô hình dự đoán qua ECS, EC2, SQS, SNS, S3 và SageMaker, triển khai các yêu cầu bảo mật của Samsung SDS, và tích hợp backend với giao diện Unreal Engine 4 do một studio đối tác xây dựng.",
                      ],
                    },
                    {
                      name: "Dịch vụ quản lý năng lượng bằng AI",
                      highlights: [
                        "Thiết kế lược đồ PostgreSQL và API Rust/Axum từ đầu, và viết khoảng **25.000 dòng**, xấp xỉ 70% backend.",
                        "Chọn SQL trực tiếp qua tokio-postgres thay vì ORM để kiểm soát truy vấn và thời gian biên dịch, áp dụng bài học từ bản sao số.",
                        "Giúp hai kỹ sư đến từ Node.js và Java làm việc hiệu quả với Rust trong khoảng một tháng, trong một nhóm chín người gồm kỹ sư backend, frontend và khoa học dữ liệu.",
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
                  title: "Thực tập sinh kỹ thuật phần mềm",
                  period: "Tháng 8/2022 - Tháng 3/2023",
                  location: "Austin, Texas · Làm từ xa",
                  highlights: [
                    "Xây dựng một bot Discord bằng Python, dùng Reddit API để tìm các yêu cầu đặt vẽ tranh và đăng chúng lên kênh cộng đồng, chạy theo lịch trên AWS Lambda mỗi năm phút.",
                    "Triển khai logic Lambda cho quy đổi tiền tệ, tính phí và tính phụ phí theo từng khu vực pháp lý.",
                    "Đóng góp các chỉnh sửa frontend React về bố cục, tài nguyên và nội dung chữ.",
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
          heading: "Trước khi làm phần mềm",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Korea Institute of Maritime Science and Technology",
                  title: "Biên dịch viên và phiên dịch viên",
                  period: "Tháng 6 - Tháng 7/2023, Tháng 8/2024",
                  location: "Seoul, Hàn Quốc · Hợp đồng",
                  highlights: [
                    "Phiên dịch tiếng Anh, tiếng Hàn và tiếng Na Uy tại một hội nghị nghiên cứu hàng hải có sự tham dự của đại sứ Na Uy tại Hàn Quốc, một thứ trưởng Na Uy, ban lãnh đạo viện và chủ tịch Tập đoàn Dongwon.",
                    "Dịch các bài phát biểu và hướng dẫn nhân viên cùng giám đốc viện cho một hội nghị quốc tế và một sự kiện của Bộ Năng lượng Hoa Kỳ về một dự án chung.",
                  ],
                },
                {
                  organization: "Quân đội Đại Hàn Dân Quốc · Sư đoàn Bộ binh số 2 Lục quân Hoa Kỳ",
                  title: "Trung sĩ, liên lạc viên và giảng viên KATUSA",
                  period: "Tháng 3/2016 - Tháng 12/2017",
                  location: "Dongducheon, Hàn Quốc",
                  highlights: [
                    "Phục vụ tại Tiểu đoàn Bộ chỉ huy của sư đoàn qua chương trình [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army), trong môi trường chung Hàn - Mỹ.",
                    "Huấn luyện và tiếp nhận khoảng 6.000 quân nhân Mỹ và Hàn Quốc trong các khóa về thể lực, văn hóa, lịch sử và phòng ngừa.",
                    "Chỉ huy một nhóm 13 người và duy trì trạng thái sẵn sàng trong [cuộc khủng hoảng năm 2017](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) trên bán đảo.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Nhân viên bốc xếp hậu cần",
                  period: "2020 - 2022",
                  location: "Hàn Quốc · Hợp đồng thời vụ",
                  highlights: [
                    "Làm việc ca đêm tại kho hàng song song với việc học đại học.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "Giới thiệu",
      title: "Dự án",
      lede: "Các công việc độc lập và mã nguồn mở, chủ yếu bằng Rust. Các bản demo tương tác nằm ở [trang dự án](/projects).",
      sections: [
        {
          id: "featured",
          heading: "Nổi bật",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "Một công cụ dòng lệnh để ghi nhật ký công việc. Mục tiêu, tác vụ đã hoàn thành, chỉnh sửa và rút lại được ghi thành các sự kiện chỉ thêm vào (append-only) cùng các bản chiếu (projection) trạng thái hiện tại có lập chỉ mục, còn kho lưu trữ nền Git hợp nhất lịch sử giữa các máy.",
                  tags: [
                    "Event sourcing",
                    "Sync",
                  ],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Một ứng dụng máy tính và trình duyệt cho dữ liệu bản đồ Europa Universalis V, với các tập dữ liệu nén nhúng sẵn, chỉ mục tìm kiếm và sắp xếp tính trước, bộ lọc theo danh mục và bảng có thể đổi kích thước. [Dùng thử tại đây](/eu5-locations-db).",
                  tags: [
                    "Native + web",
                    "Indexes",
                  ],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Chính trang web này: xuất bản bài viết, nhiếp ảnh, diễn đàn, chat trực tiếp có cuộc gọi, trình khám phá địa hình Minecraft và các bản demo WebAssembly, tất cả do một tệp thực thi Rust phục vụ. [Cách xây dựng](/about-blog).",
                  tags: [
                    "Monorepo",
                    "Self-hosted",
                  ],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "Một công cụ tạo và định dạng UUIDv4 có thể xuất ra terminal hoặc tệp, tùy chọn kiểm tra trùng lặp, và định dạng tương thích Python và JSON; nhanh hơn uuidgen khoảng ba lần trong một benchmark cục bộ ban đầu.",
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
          heading: "Các công việc khác",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: một [bản sửa lỗi gửi lên upstream](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) cho máy chủ Minecraft viết bằng Rust, giữ nguyên các thiết lập overworld của Paper khi nhập và lưu; đang chờ review.",
                "**Công cụ bản đồ Minecraft**: một plugin Paper và các dịch vụ Rust đứng sau [trình khám phá địa hình](/minecraft) của trang này, gồm định dạng ô biome nhị phân gọn nhẹ và dự đoán biome dựa trên seed.",
                "**Đồng bộ hệ thống tệp**: một tiện ích Rust để đồng bộ hai chiều thư mục và dọn dẹp artifact build giữa các hệ thống tệp khác loại, có chạy thử (dry run), nhật ký thao tác, checksum tùy chọn và cơ chế chặn các lần xóa hàng loạt ngoài ý muốn.",
                "**impulsr**: một công cụ phiên âm video YouTube và thu thập bình luận, tạo bản tóm tắt bằng mô hình ngôn ngữ cho công việc marketing.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Cách tôi xây dựng",
          blocks: [
            {
              kind: "paragraph",
              text: "Tôi thích những hệ thống nhỏ, dễ kiểm tra: một tệp thực thi khi một là đủ, giới hạn rõ ràng cho mọi hàng đợi và bộ nhớ đệm, và định dạng dữ liệu chọn theo kiểu truy cập chứ không theo thói quen.",
            },
            {
              kind: "paragraph",
              text: "Đo đạc đi trước nhận định; [ghi chú về hiệu năng](/about-blog/performance) của trang này cho thấy cách thực hiện các phép đo.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "Giới thiệu",
      title: "Lý lịch",
      lede: "Học vấn, nghiên cứu, chứng chỉ và những sở thích ngoài phần mềm.",
      sections: [
        {
          id: "education",
          heading: "Học vấn",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Sungkyunkwan University",
                  title: "Cử nhân Kỹ thuật phần mềm (B.Eng.)",
                  period: "Tháng 3/2015 - Tháng 8/2023",
                  location: "Suwon, Hàn Quốc",
                  summary: "Khoa Khoa học và Kỹ thuật máy tính. Thời gian học bao gồm cả nghĩa vụ quân sự bắt buộc.",
                  highlights: [
                    "Các môn học về phần mềm nhúng, mạch logic, hệ điều hành, mạng, lập trình cấp thấp và điện tử, cùng với thực hành kỹ thuật phần mềm.",
                    "Sungkyunkwan University có nguồn gốc từ năm 1398 với tư cách học viện hoàng gia của triều đại Joseon và là một trong những trường đại học nghiên cứu hàng đầu của Hàn Quốc.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Nghiên cứu",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Korean Language NLP Model Based Emotional Analysis of Social Media Communities.** Áp dụng KoBERT vào phân tích cảm xúc và phân loại nội dung mạng xã hội tiếng Hàn, khảo sát sự thay đổi cảm xúc trong giai đoạn đầu của đại dịch COVID-19.",
            },
            {
              kind: "paragraph",
              text: "Dự án nhận Giải Đồng tại giải thưởng luận văn và dự án tốt nghiệp của trường, trong khoảng 60 bài dự thi.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Giải thưởng và chứng chỉ",
          blocks: [
            {
              kind: "table",
              columns: [
                "Năm",
                "Ghi nhận",
                "Chi tiết",
              ],
              rows: [
                [
                  "2023",
                  "**Giải Đồng**",
                  "Giải thưởng luận văn và dự án tốt nghiệp của Sungkyunkwan University",
                ],
                [
                  "2023",
                  "**IELTS 8.5 / 9.0**",
                  "CEFR C2",
                ],
                [
                  "2021",
                  "**Hạng 7**",
                  "Hackathon Thiết kế Capstone và Ý tưởng, Hiệp hội Thông tin Internet Hàn Quốc",
                ],
                [
                  "2020",
                  "**Hạng 3**",
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
          heading: "Ngôn ngữ",
          blocks: [
            {
              kind: "list",
              items: [
                "**Tiếng Hàn**: tiếng mẹ đẻ.",
                "**Tiếng Anh**: thành thạo ở mức chuyên nghiệp (C2).",
                "**Tiếng Pháp, tiếng Đức, tiếng Tây Ban Nha và tiếng Trung**: hiểu biết cơ bản.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "Ngoài phần mềm",
          blocks: [
            {
              kind: "list",
              items: [
                "**Nhiếp ảnh**: chụp phong cảnh và chân dung từ năm 2010, với khoảng 30.000 khung hình lưu trữ. Một số ảnh chọn lọc có trong [thư viện ảnh](/photographs).",
                "**Báo chí**: phóng viên sinh viên của Sungkyun Times, tạp chí tiếng Anh của trường, viết về công nghệ, an ninh và các vấn đề quốc tế.",
                "**Giảng dạy và tình nguyện**: dạy tiếng Anh tại một trung tâm cộng đồng và làm tình nguyện tại một cửa hàng đồ cũ do người khuyết tật điều hành.",
                "**Dịch thuật**: phiên dịch và biên dịch cho các cơ quan công; xem [Kinh nghiệm](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
