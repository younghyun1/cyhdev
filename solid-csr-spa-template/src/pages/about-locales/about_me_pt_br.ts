import type { AboutMeContent } from "./content_types";

const content: AboutMeContent = {
  updated: "novembro de 2026",
  chrome: {
    overview: "Visão geral",
    updated: "Última atualização",
    onThisPage: "Nesta página",
    sectionNavigation: "Páginas sobre mim",
    previous: "Anterior",
    next: "Próxima",
    readingTime: "{minutes} min de leitura",
  },
  pages: {
    overview: {
      eyebrow: "Young Hyun Chi · 지영현 · 池營賢",
      title: "Sobre",
      lede: "Engenheiro de backend e infraestrutura que trabalha com Rust, PostgreSQL e plataformas de nuvem. Construo serviços que aproveitam bem o hardware em que rodam, além do trabalho de implantação, monitoramento e segurança que os mantém no ar.",
      sections: [
        {
          id: "introduction",
          heading: "Introdução",
          blocks: [
            {
              kind: "paragraph",
              text: "Sou um engenheiro de software coreano que mora no Colorado. Na **Soundpatrol** atuo como engenheiro Rust em cibersegurança, DevOps, serviços web e ferramentas para desenvolvedores, principalmente com Rust, Python, PostgreSQL, Kubernetes e Google Cloud.",
            },
            {
              kind: "paragraph",
              text: "Antes disso, liderei o trabalho de backend e infraestrutura de um gêmeo digital de edifício da Samsung C&T, contribuí com os backends dos aplicativos da Hyundai, Kia e Genesis e construí a coleta de dados de uma plataforma de análise do YouTube. Desde 2023, meu trabalho se concentra em serviços de backend, pipelines de dados e na infraestrutura por baixo deles.",
            },
            {
              kind: "paragraph",
              text: "Acredito em construir serviços que aproveitam bem o hardware e as linguagens de programação modernas. Na prática, isso significa modelagem cuidadosa de esquemas, filas e caches limitados, tratamento explícito de erros e implantações que podem ser inspecionadas e recuperadas. Gosto de ajudar colegas a aprender Rust e quero trabalhar com pessoas que têm orgulho do próprio ofício.",
            },
          ],
        },
        {
          id: "results",
          heading: "Resultados selecionados",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "97%",
                  count: 97,
                  suffix: "%",
                  label: "Redução do custo mensal de nuvem do gêmeo digital da Samsung C&T, de cerca de US$ 5.000 para US$ 150",
                },
                {
                  value: "80",
                  count: 80,
                  label: "Endpoints REST implementados para o gêmeo digital, com lógica de domínio da engenharia de construção",
                },
                {
                  value: "9 meses",
                  count: 9,
                  suffix: " meses",
                  label: "Operação do gêmeo digital em produção sem nenhuma interrupção em tempo de execução",
                },
                {
                  value: "+20%",
                  count: 20,
                  prefix: "+",
                  suffix: "%",
                  label: "Ganho de vazão em leitura mista no backend deste site com [PGO e BOLT](/about-blog/performance)",
                },
                {
                  value: "~20 MB",
                  count: 20,
                  prefix: "~",
                  suffix: " MB",
                  label: "Contêineres de serviço estaticamente vinculados, gerados por builds em vários estágios",
                },
                {
                  value: "30.000",
                  label: "Linhas escritas para o backend do gêmeo digital, cerca de 75% do código",
                },
              ],
            },
          ],
        },
        {
          id: "toolbox",
          heading: "Ferramentas",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Linguagens",
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
                  label: "Dados",
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
                  label: "Infraestrutura",
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
                  label: "Confiabilidade e segurança",
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
                  label: "Interfaces e dispositivos",
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
          heading: "Mais sobre mim",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Experiência",
                  href: "/about/experience",
                  meta: "2016 - presente",
                  text: "Funções desde a Soundpatrol até meu primeiro estágio, com os projetos e resultados de cada uma.",
                },
                {
                  title: "Projetos",
                  href: "/about/projects",
                  meta: "Rust · código aberto",
                  text: "Sillok, EU5 Location DB, este site e outros trabalhos independentes e de código aberto.",
                },
                {
                  title: "Trajetória",
                  href: "/about/background",
                  meta: "Formação · pesquisa",
                  text: "Formação acadêmica, pesquisas publicadas, prêmios, idiomas e trabalho fora da área de software.",
                },
                {
                  title: "Como este site funciona",
                  href: "/about-blog",
                  meta: "Stack técnica",
                  text: "O host, o backend, o frontend, os subsistemas e as medições de desempenho por trás do cyhdev.com.",
                },
              ],
            },
          ],
        },
        {
          id: "contact",
          heading: "Contato",
          blocks: [
            {
              kind: "list",
              items: [
                "**E-mail**: [younghyun1@gmail.com](mailto:younghyun1@gmail.com)",
                "**GitHub**: [github.com/younghyun1](https://github.com/younghyun1)",
                "**LinkedIn**: [linkedin.com/in/young-hyun-chi-553431376](https://www.linkedin.com/in/young-hyun-chi-553431376/)",
              ],
            },
            {
              kind: "paragraph",
              text: "O e-mail é a forma mais confiável de falar comigo. Se quiser entrar no servidor de Minecraft, avise por lá também.",
            },
          ],
        },
      ],
    },
    experience: {
      eyebrow: "Sobre",
      title: "Experiência",
      lede: "Trabalho de backend, infraestrutura e segurança em uma startup americana, para clientes corporativos coreanos e em um estágio inicial, seguido do trabalho que veio antes do software.",
      sections: [
        {
          id: "software",
          heading: "Engenharia de software",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Soundpatrol",
                  title: "Engenheiro Rust",
                  period: "maio de 2026 - presente",
                  location: "Estados Unidos · Remoto",
                  summary: "Cibersegurança, DevOps, desenvolvimento de serviços web e ferramentas para desenvolvedores em uma empresa de detecção de música e áudio.",
                  highlights: [
                    "Mantenho a infraestrutura Kubernetes no Google Cloud, a infraestrutura como código e fluxos de implantação revisados que usam credenciais de curta duração, restritas a cada ambiente.",
                    "Planejo e executo migrações de armazenamentos de dados em produção, verificando o estado migrado e ensaiando a recuperação antes de cada virada.",
                    "Consolido métricas, logs e traces no OpenTelemetry e construo ferramentas em Rust para saúde dos serviços, diagnósticos e visibilidade de custos.",
                    "Atuo na prontidão de segurança: administração de acessos, rotação de credenciais e controle de validade, políticas, resposta a incidentes e evidências de auditoria.",
                    "Construo serviços e ferramentas internas em Rust e Python para fluxos de operações e suporte.",
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
                  title: "Engenheiro de software",
                  period: "jan. de 2025 - jul. de 2025",
                  location: "Seongnam, Coreia do Sul",
                  summary: "Serviços de backend e ferramentas de infraestrutura para plataformas internas e para o aplicativo oficial de fãs de um grupo de K-pop.",
                  highlights: [
                    "Contribuí com APIs, correções de bugs e integrações nos backends dos aplicativos oficiais da Hyundai, Kia e Genesis, trabalhando em uma grande base de código Java e coordenando com fornecedores serviços usados por dezenas de milhões de pessoas na Ásia e na Europa.",
                    "Construí pipelines de internacionalização em Rust e Python que transformam e carregam conteúdo traduzido em bancos de dados corporativos, em coordenação com os administradores de banco de dados responsáveis por conjuntos de dados ligados a sistemas de fábrica.",
                    "Desenvolvi ferramentas de gerenciamento de infraestrutura para serviços internos.",
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
                  title: "Engenheiro de software contratado",
                  period: "nov. de 2024 - dez. de 2024",
                  location: "Seul, Coreia do Sul",
                  highlights: [
                    "Projetei o esquema PostgreSQL e o backend em Rust de uma plataforma de análise do YouTube assistida por IA e liderei sua implantação na AWS e no Google Cloud.",
                    "Construí fluxos de coleta pela API do YouTube que cobrem dezenas de milhões de comentários e usuários, combinando dados de canais, transcrições e conteúdo de páginas.",
                    "Integrei modelos de linguagem hospedados e modelos de código aberto no próprio servidor para gerar resumos e insights de marketing.",
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
                  title: "Engenheiro de software de backend (líder)",
                  period: "ago. de 2023 - ago. de 2024",
                  location: "Seul, Coreia do Sul",
                  summary: "Liderei o desenvolvimento de backend, banco de dados e infraestrutura de nuvem de duas plataformas de edifícios e energia.",
                  highlights: [],
                  projects: [
                    {
                      name: "Gêmeo digital do Global Engineering Center da Samsung C&T",
                      highlights: [
                        "Substituí um backend Express e um conjunto de microsserviços na nuvem por um monolito otimizado em Rust/Axum, reduzindo o gasto mensal de infraestrutura de cerca de **US$ 5.000 para US$ 150**.",
                        "Escrevi cerca de **30.000 linhas**, aproximadamente 75% do backend, incluindo **80 endpoints REST** com lógica de domínio da engenharia de construção, integração de dados e operações em tempo real.",
                        "Reduzi muitos caminhos de requisição com P99 de vários segundos para um ou dois dígitos de milissegundos em uma implantação de dois núcleos e 4 GB, com Rust assíncrono, multithreading, cache e ajuste de esquema e consultas do PostgreSQL.",
                        "Integrei vários milhares de sensores, medidores e controladores de HVAC e de sistemas de água via BACnet e Modbus, no local, na sede da Samsung C&T.",
                        "O sistema operou por nove meses em produção sem nenhuma interrupção em tempo de execução, com cerca de 90% de cobertura de tratamento de erros.",
                        "Construí o CI/CD com runners do GitHub, AWS CodeDeploy e Docker; builds estáticos, ajustados ao processador, geraram contêineres portáteis de cerca de 20 MB.",
                        "Conectei a inferência de modelos preditivos por meio de ECS, EC2, SQS, SNS, S3 e SageMaker, implementei os requisitos de segurança da Samsung SDS e integrei o backend a uma interface em Unreal Engine 4 criada por um estúdio parceiro.",
                      ],
                    },
                    {
                      name: "Serviço de gestão de energia com IA",
                      highlights: [
                        "Projetei do zero o esquema PostgreSQL e a API em Rust/Axum e escrevi cerca de **25.000 linhas**, aproximadamente 70% do backend.",
                        "Escolhi SQL direto com tokio-postgres em vez de um ORM, para ter controle sobre as consultas e os tempos de compilação, aplicando as lições do gêmeo digital.",
                        "Ajudei dois engenheiros vindos de Node.js e Java a se tornarem produtivos em Rust em cerca de um mês, em uma equipe de nove pessoas entre engenheiros de backend, frontend e ciência de dados.",
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
                  title: "Estagiário de engenharia de software",
                  period: "ago. de 2022 - mar. de 2023",
                  location: "Austin, Texas · Remoto",
                  highlights: [
                    "Construí um bot de Discord em Python que usava a API do Reddit para encontrar pedidos de encomendas de arte e publicá-los em um canal da comunidade, agendado no AWS Lambda a cada cinco minutos.",
                    "Implementei no Lambda a lógica de conversão de moedas, cobrança e cálculo de taxas adicionais entre jurisdições.",
                    "Contribuí com revisões no frontend em React, em layout, recursos visuais e textos.",
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
          heading: "Antes do software",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Instituto Coreano de Ciência e Tecnologia Marítima",
                  title: "Tradutor e intérprete",
                  period: "jun. - jul. de 2023, ago. de 2024",
                  location: "Seul, Coreia do Sul · Contrato",
                  highlights: [
                    "Fiz interpretação em inglês, coreano e norueguês em uma conferência de pesquisa marítima com a presença do embaixador da Noruega na Coreia, de um secretário de Estado norueguês, da direção do instituto e do presidente do Grupo Dongwon.",
                    "Traduzi discursos e orientei a equipe do instituto e seu diretor para uma conferência internacional e para um evento do Departamento de Energia dos EUA sobre um projeto conjunto.",
                  ],
                },
                {
                  organization: "Exército da República da Coreia · 2ª Divisão de Infantaria do Exército dos EUA",
                  title: "Sargento, elo de ligação e instrutor da KATUSA",
                  period: "mar. de 2016 - dez. de 2017",
                  location: "Dongducheon, Coreia do Sul",
                  highlights: [
                    "Servi no Batalhão de Quartel-General da divisão por meio do programa [KATUSA](https://en.wikipedia.org/wiki/Korean_Augmentation_to_the_United_States_Army), em um ambiente conjunto coreano-americano.",
                    "Instruí e recebi cerca de 6.000 militares americanos e coreanos em cursos de condicionamento físico, cultura, história e prevenção.",
                    "Liderei uma equipe de 13 pessoas e mantive a prontidão durante a [crise de 2017](https://en.wikipedia.org/wiki/2017%E2%80%932018_North_Korea_crisis) na península.",
                  ],
                },
                {
                  organization: "Coupang",
                  title: "Carregador de logística",
                  period: "2020 - 2022",
                  location: "Coreia do Sul · Contratos ocasionais",
                  highlights: [
                    "Trabalho em armazém no turno da noite, junto com os estudos universitários.",
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
    projects: {
      eyebrow: "Sobre",
      title: "Projetos",
      lede: "Trabalhos independentes e de código aberto, principalmente em Rust. As demos interativas estão na [página de projetos](/projects).",
      sections: [
        {
          id: "featured",
          heading: "Destaques",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Sillok",
                  href: "https://github.com/younghyun1/sillok",
                  meta: "Rust · CLI · Turso/SQLite",
                  text: "Um registro de trabalho em linha de comando. Objetivos, tarefas concluídas, correções e retratações são gravados como eventos somente de acréscimo, com projeções atuais indexadas, e um arquivo versionado em Git mescla os históricos entre máquinas.",
                  tags: [
                    "Event sourcing",
                    "Sync",
                  ],
                },
                {
                  title: "EU5 Location DB",
                  href: "https://github.com/younghyun1/eu5-location-filter",
                  meta: "Rust · Slint · WebAssembly",
                  text: "Um aplicativo de desktop e de navegador para os dados do mapa de Europa Universalis V, com conjuntos de dados compactados embutidos, índices de busca e ordenação pré-calculados, filtros categóricos e tabelas redimensionáveis. [Experimente aqui](/eu5-locations-db).",
                  tags: [
                    "Native + web",
                    "Indexes",
                  ],
                },
                {
                  title: "cyhdev.com",
                  href: "https://github.com/younghyun1/cyhdev",
                  meta: "Rust · Axum · PostgreSQL · SolidJS",
                  text: "Este site: publicações, fotografia, um fórum, chat ao vivo com chamadas, um explorador de terreno de Minecraft e demos em WebAssembly, servidos por um único binário em Rust. [Como foi construído](/about-blog).",
                  tags: [
                    "Monorepo",
                    "Self-hosted",
                  ],
                },
                {
                  title: "oohid",
                  href: "https://github.com/younghyun1/oohid",
                  meta: "Rust · CLI",
                  text: "Um gerador e formatador de UUIDv4 com saída no terminal e em arquivo, verificação opcional de duplicatas e formatação compatível com Python e JSON; cerca de três vezes mais rápido que o uuidgen em um benchmark local inicial.",
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
          heading: "Mais trabalhos",
          blocks: [
            {
              kind: "list",
              items: [
                "**Pumpkin**: uma [correção upstream](https://github.com/Pumpkin-MC/Pumpkin/pull/3447) para o servidor de Minecraft em Rust que preserva as configurações do overworld do Paper ao importar e salvar; aberta para revisão.",
                "**Ferramentas de mapa do Minecraft**: um plugin para Paper e serviços em Rust por trás do [explorador de terreno](/minecraft) deste site, incluindo um formato binário compacto de tiles de bioma e previsões de bioma a partir da semente.",
                "**Sincronização de sistemas de arquivos**: um utilitário em Rust para sincronização bidirecional de pastas e limpeza de artefatos de build entre sistemas de arquivos distintos, com simulações, registros de operações, checksums opcionais e proteções contra exclusões grandes não intencionais.",
                "**impulsr**: uma ferramenta de transcrição e coleta de comentários do YouTube que gera resumos com modelos de linguagem para trabalhos de marketing.",
              ],
            },
          ],
        },
        {
          id: "approach",
          heading: "Como eu construo",
          blocks: [
            {
              kind: "paragraph",
              text: "Prefiro sistemas pequenos e inspecionáveis: um só binário onde um só binário basta, um limite explícito para cada fila e cache, e formatos de dados escolhidos pelo padrão de acesso, não por hábito.",
            },
            {
              kind: "paragraph",
              text: "As medições vêm antes das afirmações; as [notas de desempenho](/about-blog/performance) deste site mostram como elas são feitas.",
            },
          ],
        },
      ],
    },
    background: {
      eyebrow: "Sobre",
      title: "Trajetória",
      lede: "Formação acadêmica, pesquisa, certificações e interesses fora do software.",
      sections: [
        {
          id: "education",
          heading: "Formação acadêmica",
          blocks: [
            {
              kind: "roles",
              roles: [
                {
                  organization: "Universidade Sungkyunkwan",
                  title: "Bacharelado em Engenharia de Software",
                  period: "mar. 2015 - ago. 2023",
                  location: "Suwon, Coreia do Sul",
                  summary: "Departamento de Ciência e Engenharia da Computação. O prazo do curso inclui o serviço militar obrigatório.",
                  highlights: [
                    "Disciplinas de software embarcado, circuitos lógicos, sistemas operacionais, redes, programação de baixo nível e eletrônica, além da prática de engenharia de software.",
                    "A Universidade Sungkyunkwan tem origem em 1398, como a academia real da dinastia Joseon, e é uma das principais universidades de pesquisa da Coreia.",
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "research",
          heading: "Pesquisa",
          blocks: [
            {
              kind: "callout",
              title: "IMCOM 2023 · IEEE Xplore",
              text: "**Análise emocional de comunidades de redes sociais com um modelo de PLN para coreano.** Apliquei o KoBERT à análise de sentimento e à categorização de redes sociais em coreano, examinando a mudança emocional no início do período da COVID-19.",
            },
            {
              kind: "paragraph",
              text: "O projeto recebeu o Prêmio de Bronze nos prêmios de trabalhos de conclusão e projetos da universidade, entre cerca de 60 inscrições.",
            },
          ],
        },
        {
          id: "awards",
          heading: "Prêmios e certificações",
          blocks: [
            {
              kind: "table",
              columns: [
                "Ano",
                "Reconhecimento",
                "Detalhe",
              ],
              rows: [
                [
                  "2023",
                  "**Prêmio de Bronze**",
                  "Prêmios de trabalhos de conclusão e projetos da Universidade Sungkyunkwan",
                ],
                [
                  "2023",
                  "**IELTS 8.5 / 9.0**",
                  "CEFR C2",
                ],
                [
                  "2021",
                  "**7º lugar**",
                  "Hackathon de Projeto Final e Ideias, Sociedade Coreana de Informação na Internet",
                ],
                [
                  "2020",
                  "**3º lugar**",
                  "Hackathon de Fintech, Universidade Nacional de Ciência e Tecnologia de Seul",
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
          heading: "Idiomas",
          blocks: [
            {
              kind: "list",
              items: [
                "**Coreano**: nativo.",
                "**Inglês**: fluência profissional (C2).",
                "**Francês, alemão, espanhol e mandarim**: familiaridade básica.",
              ],
            },
          ],
        },
        {
          id: "beyond",
          heading: "Fora do software",
          blocks: [
            {
              kind: "list",
              items: [
                "**Fotografia**: fotografia de paisagem e retrato desde 2010, com cerca de 30.000 quadros arquivados. Uma seleção está na [galeria](/photographs).",
                "**Jornalismo**: repórter estudantil do Sungkyun Times, a revista em inglês da universidade, cobrindo tecnologia, segurança e assuntos internacionais.",
                "**Ensino e voluntariado**: dei aulas de inglês em um centro comunitário e fui voluntário em um brechó administrado por pessoas com deficiência.",
                "**Tradução**: interpretação e tradução para instituições públicas; veja [Experiência](/about/experience#other).",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
