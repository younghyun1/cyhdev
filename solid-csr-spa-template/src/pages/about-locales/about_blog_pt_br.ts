import type { AboutBlogContent } from "./content_types";

const content: AboutBlogContent = {
  updated: "novembro de 2026",
  chrome: {
    overview: "Visão geral",
    updated: "Última atualização",
    onThisPage: "Nesta página",
    sectionNavigation: "Páginas da stack técnica",
    previous: "Anterior",
    next: "Próxima",
    readingTime: "{minutes} min de leitura",
  },
  pages: {
    overview: {
      eyebrow: "Sobre este site",
      title: "Stack técnica do blog",
      lede: "Um binário Rust, um banco de dados PostgreSQL, uma máquina pequena. Estas páginas descrevem como o site é construído, hospedado, protegido e medido, e por que ele tem este formato.",
      sections: [
        {
          id: "at-a-glance",
          heading: "Resumo",
          blocks: [
            {
              kind: "stats",
              items: [
                {
                  value: "53.889 req/s",
                  label: "Vazão mediana de leitura pública mista com PGO e BOLT em quatro núcleos do servidor",
                },
                {
                  value: "~150 µs",
                  label: "Ida e volta ao banco de dados observada por socket UNIX, contra ~600 µs por TCP em localhost",
                },
                {
                  value: "0",
                  label: "Processos Node.js em produção; o frontend está embutido no binário do servidor",
                },
                {
                  value: "24",
                  count: 24,
                  label: "Idiomas da interface, cada um baixado apenas quando selecionado",
                },
                {
                  value: "4.096",
                  label: "Conexões abertas admitidas antes do TLS, com no máximo 64 por endereço ou IPv6 /64",
                },
                {
                  value: "1",
                  count: 1,
                  label: "Processo que serve TLS, a API, WebSockets, mídia WebRTC e o frontend embutido",
                },
              ],
            },
          ],
        },
        {
          id: "request-path",
          heading: "Caminho da requisição",
          blocks: [
            {
              kind: "pipeline",
              steps: [
                {
                  label: "Navegador",
                  detail: "HTTP/2 ou HTTP/1.1 sobre TLS. O frontend fica em cache sob URLs imutáveis com hash no nome.",
                },
                {
                  label: "Admissão",
                  detail: "As conexões TCP são admitidas antes do TLS: 4.096 no total e 64 por endereço ou IPv6 /64.",
                },
                {
                  label: "rustls",
                  detail: "O TLS termina dentro do processo Rust, com `TCP_NODELAY` em todos os sockets.",
                },
                {
                  label: "Axum",
                  detail: "Cabeçalhos de segurança, sessões, limites de taxa, compressão e limites de corpo; depois, um handler da funcionalidade.",
                },
                {
                  label: "Cache · Diesel",
                  detail: "Caches em memória com limite respondem às leituras frequentes; todo o resto passa pelo Diesel assíncrono.",
                },
                {
                  label: "PostgreSQL 18",
                  detail: "No mesmo host, por socket UNIX, com E/S assíncrona via io_uring.",
                },
              ],
            },
          ],
        },
        {
          id: "chapters",
          heading: "Capítulos",
          blocks: [
            {
              kind: "cards",
              cards: [
                {
                  title: "Infraestrutura",
                  href: "/about-blog/infrastructure",
                  meta: "Host · Gentoo · rede",
                  text: "A máquina, o sistema operacional e o sistema de arquivos, a rede e por que não há proxy reverso.",
                },
                {
                  title: "Backend",
                  href: "/about-blog/backend",
                  meta: "Rust · Axum · PostgreSQL",
                  text: "A stack do serviço, o modelo de dados, os limites de execução, os cabeçalhos de segurança e os contratos da API.",
                },
                {
                  title: "Frontend",
                  href: "/about-blog/frontend",
                  meta: "SolidJS 2 · Vite · Tailwind",
                  text: "A estrutura da aplicação, o carregamento, a localização, o sistema de design e a animação.",
                },
                {
                  title: "Subsistemas",
                  href: "/about-blog/subsystems",
                  meta: "WebRTC · Minecraft · Geo-IP",
                  text: "Chat ao vivo com um SFU no próprio processo, o explorador de Minecraft, consultas de Geo-IP e WebAssembly.",
                },
                {
                  title: "Desempenho",
                  href: "/about-blog/performance",
                  meta: "PGO · BOLT · oha",
                  text: "Builds otimizados, como foram medidos e os resultados.",
                },
              ],
            },
          ],
        },
        {
          id: "verification",
          heading: "Como as mudanças são verificadas",
          blocks: [
            {
              kind: "list",
              items: [
                "Clippy nos alvos nativo e WebAssembly como barreira de implementação.",
                "Verificações locais com cargo xtask para formatação, testes unitários nativos, divergência do contrato OpenAPI e, no frontend, checagem de tipos, lint, testes unitários e build.",
                "Playwright em Chromium e WebKit com verificações de acessibilidade do axe e de reflow em telas pequenas, até 320 pixels CSS.",
                "Testes de integração e de reversão de migrações em uma instância descartável do PostgreSQL, além de uma varredura local de segredos com gitleaks.",
              ],
            },
          ],
        },
      ],
    },
    infrastructure: {
      eyebrow: "Stack técnica",
      title: "Infraestrutura",
      lede: "Uma máquina pequena em casa, uma distribuição Linux baseada em código-fonte e nenhuma camada que o tráfego não precise.",
      sections: [
        {
          id: "host",
          heading: "O host",
          blocks: [
            {
              kind: "paragraph",
              text: "O site roda em um [miniservidor](https://store.minisforum.com/products/minisforum-um690l-slim) na minha residência: um processador móvel Ryzen 9 de oito núcleos que chega a 4,9 GHz, 32 GB de RAM e um SSD NVMe de 1 TB. Por cerca de US$ 400, custa menos que um ano de hardware de nuvem visivelmente mais fraco, e reflete meu entusiasmo por hospedagem própria.",
            },
            {
              kind: "paragraph",
              text: "Nele rodam o servidor combinado de backend e frontend, o PostgreSQL e um servidor de Minecraft. Se quiser jogar, me mande um e-mail.",
            },
          ],
        },
        {
          id: "os",
          heading: "Gentoo e btrfs",
          blocks: [
            {
              kind: "paragraph",
              text: "O [Gentoo](https://www.gentoo.org/) não é a escolha óbvia para um servidor; no trabalho eu escolheria Debian stable com ext4. Aqui eu gosto de compilar os pacotes eu mesmo, com otimizações para o processador exato.",
            },
            {
              kind: "paragraph",
              text: "O [btrfs](https://en.wikipedia.org/wiki/Btrfs) traz snapshots, compressão e copy-on-write, mas o copy-on-write fragmenta muito os arquivos de banco de dados. Por isso, os diretórios de dados do PostgreSQL e do Minecraft ficam fora dele. Um [benchmark de sistemas de arquivos](https://www.enterprisedb.com/blog/postgres-vs-file-systems-performance-comparison) mostra o btrfs padrão atrás do ext4 e do XFS; suspeito que desativar o copy-on-write elimine a maior parte dessa diferença, o que seria uma medição interessante.",
            },
          ],
        },
        {
          id: "network",
          heading: "Rede",
          blocks: [
            {
              kind: "paragraph",
              text: "O tráfego chega por uma conexão Xfinity cabeada de 1 Gbps; o modem da operadora não tem Ethernet de 2,5 Gb, algo que este site não vai notar. O Route 53 hospeda o DNS.",
            },
            {
              kind: "paragraph",
              text: "Dentro da máquina não há proxy reverso, runtime de contêineres nem service mesh. O PostgreSQL roda direto no sistema operacional, e um único binário Rust serve o frontend e a API. As requisições HTTP simples são redirecionadas para HTTPS pelo mesmo processo.",
            },
          ],
        },
        {
          id: "sockets",
          heading: "Sockets UNIX",
          blocks: [
            {
              kind: "paragraph",
              text: "O servidor acessa o PostgreSQL por um socket UNIX, e não por TCP em localhost, o que dispensa por completo a pilha de rede. [Benchmarks](https://www.cybertec-postgresql.com/en/postgresql-performance-advice-unix-sockets-vs-localhost/) mostram a taxa de transações quase dobrando; aqui observei idas e voltas de até **150 µs**, contra cerca de **600 µs** em localhost. É um recurso antigo e irrelevante para a maioria das implantações em nuvem, e é mais rápido.",
            },
            {
              kind: "paragraph",
              text: "O PostgreSQL 18, lançado em setembro de 2025, adicionou [E/S assíncrona](https://pganalyze.com/blog/postgres-18-async-io), e o io_uring está ativado aqui, o que acelera visivelmente as leituras.",
            },
          ],
        },
        {
          id: "deploy",
          heading: "Build e implantação",
          blocks: [
            {
              kind: "list",
              items: [
                "O build do frontend é embutido no binário do backend junto com cópias pré-comprimidas em zstd e gzip, então nenhum processo Node.js roda em produção.",
                "Os builds otimizados podem ser treinados com otimização guiada por perfil e otimizados após o link com BOLT; veja [Desempenho](/about-blog/performance).",
                "Credenciais, certificados e dados de execução ficam fora do Git, e uma varredura com gitleaks verifica a árvore antes de as mudanças saírem da máquina.",
              ],
            },
          ],
        },
      ],
    },
    backend: {
      eyebrow: "Stack técnica",
      title: "Backend",
      lede: "Rust com Axum e Tokio, PostgreSQL por meio do Diesel assíncrono e um limite explícito para tudo que um cliente pode manter aberto.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Execução",
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
                  label: "Dados",
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
                  label: "Integrações",
                  items: [
                    "openidconnect",
                    "lettre",
                    "Amazon S3",
                    "webrtc-rs",
                    "image + AVIF",
                  ],
                },
                {
                  label: "Contratos",
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
              text: "O serviço é um único processo com estado compartilhado: sessões em memória, sincronização de cache na inicialização e tarefas de manutenção agendadas. O mimalloc é o alocador global; se a máquina tem núcleos de verdade, vale levar o comportamento do alocador a sério.",
            },
          ],
        },
        {
          id: "data",
          heading: "Modelo de dados",
          blocks: [
            {
              kind: "paragraph",
              text: "Nos trabalhos com startups e empresas coreanas que vi, MySQL e MariaDB eram tratados como os únicos bancos relacionais sérios. O PostgreSQL já os superou há muito tempo nas áreas que me importam, de UUIDs nativos a JSON binário.",
            },
            {
              kind: "list",
              items: [
                "**Posts** têm slugs, resumos, metadados em JSONB, estado de publicação, contadores desnormalizados e mapeamentos de tags. Os comentários formam encadeamentos por meio de um pai anulável, e os votos de posts e de comentários ficam em tabelas separadas.",
                "**Contas** guardam país e idioma, tabelas de funções e permissões, e fotos de perfil versionadas em uma tabela própria, e não na linha de usuário, que é muito acessada.",
                "**Chaves** são valores UUIDv7 ordenados no tempo: não são enumeráveis como IDs sequenciais e respeitam muito melhor a localidade da árvore B do que UUIDv4 aleatórios.",
                "**Busca** usa um índice tantivy persistido em disco sobre títulos e tags de posts, mantido coerente com o PostgreSQL, que continua sendo a fonte de verdade.",
              ],
            },
          ],
        },
        {
          id: "bounds",
          heading: "Limites de execução",
          blocks: [
            {
              kind: "list",
              items: [
                "As conexões são admitidas antes do TLS por um limitador compartilhado: 4.096 conexões abertas e 64 por endereço IPv4 ou IPv6 /64. Os sockets recusados são fechados sem handshake.",
                "Os cabeçalhos de requisições HTTP/1 devem chegar em até 20 segundos; as conexões HTTP/2 recebem ping a cada 30 segundos e permitem 128 streams simultâneos.",
                "Cada cache tem um limite fixo e uma fonte de verdade da qual pode ler; nenhum cresce com chaves controladas por um cliente. O cache do bundle WebAssembly, por exemplo, guarda no máximo 256 MiB.",
                "Trabalho bloqueante, como a codificação de imagens, roda em pools de workers limitados, e não nos workers do Tokio.",
              ],
            },
          ],
        },
        {
          id: "security",
          heading: "Cabeçalhos de segurança",
          blocks: [
            {
              kind: "paragraph",
              text: "A Content-Security-Policy é montada uma vez na inicialização. Scripts inline só são permitidos por hashes SHA-256 calculados a partir do `index.html` compilado no binário, de modo que o script de tema executado antes da primeira renderização não pode divergir da política. A origem principal nunca instancia WebAssembly: as demos e a aplicação EU5 rodam em frames isolados, com políticas próprias.",
            },
            {
              kind: "paragraph",
              text: "Os cookies de sessão são `Secure` e `HttpOnly`, as operações de superusuário verificam funções no servidor, as contas podem vincular identidades OpenID Connect e o Swagger UI exige autenticação em produção.",
            },
          ],
        },
        {
          id: "contracts",
          heading: "Contratos e assets",
          blocks: [
            {
              kind: "list",
              items: [
                "Os handlers têm anotações OpenAPI, os clientes TypeScript do frontend são gerados a partir delas, e o cargo xtask openapi falha quando os arquivos gerados divergem.",
                "O manifesto do Vite é embutido junto com os assets. Somente os arquivos listados no manifesto e com hash no nome recebem a política de cache `immutable` de um ano; o HTML sempre revalida.",
                "Caminhos de assets inexistentes retornam 404 com `no-store`, e não a estrutura da aplicação, para que uma página desatualizada não coloque HTML em cache sob a URL de um script.",
              ],
            },
          ],
        },
      ],
    },
    frontend: {
      eyebrow: "Stack técnica",
      title: "Frontend",
      lede: "Uma aplicação SolidJS 2 renderizada no cliente, compilada em arquivos estáticos e embutida no binário do servidor.",
      sections: [
        {
          id: "stack",
          heading: "Stack",
          blocks: [
            {
              kind: "tags",
              groups: [
                {
                  label: "Aplicação",
                  items: [
                    "SolidJS 2",
                    "Solid Router 2",
                    "TypeScript",
                    "Vite 8",
                  ],
                },
                {
                  label: "Estilos",
                  items: [
                    "Tailwind CSS 4",
                    "IBM Plex Sans",
                    "IBM Plex Mono",
                    "OKLab",
                  ],
                },
                {
                  label: "Componentes",
                  items: [
                    "Toast UI Editor",
                    "Leaflet",
                    "Chart.js",
                    "highlight.js",
                    "DOMPurify",
                  ],
                },
                {
                  label: "Testes",
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
              text: "Os signals de granularidade fina do Solid atualizam apenas os nós do DOM que dependem dos valores alterados, e as release candidates da versão 2.0 permitem que leituras assíncronas suspendam até o limite de carregamento mais próximo. O roteador carrega cada página sob demanda, e não há servidor JavaScript em produção: o binário Rust entrega os arquivos estáticos.",
            },
          ],
        },
        {
          id: "loading",
          heading: "Carregamento",
          blocks: [
            {
              kind: "list",
              items: [
                "As páginas, o painel de chat da página inicial e o shader de contornos são chunks separados, e o build informa o tamanho em gzip do que carrega primeiro.",
                "Somente o texto da interface em inglês vai no chunk inicial. Os outros idiomas, e as páginas Sobre de cada idioma, são chunks separados carregados ao serem selecionados.",
                "Os testes de desempenho em mobile com limitação de rede têm como meta LCP de no máximo 2,5 s, interações de no máximo 200 ms e CLS de no máximo 0,1.",
              ],
            },
          ],
        },
        {
          id: "localization",
          heading: "Localização",
          blocks: [
            {
              kind: "paragraph",
              text: "A interface tem suporte a 24 idiomas, incluindo layouts da direita para a esquerda para árabe, persa e urdu. Um catálogo de aproximadamente 740 chaves é validado contra o registro do backend, e os placeholders precisam sobreviver à tradução. Estas páginas Sobre são módulos de dados tipados, de modo que uma tradução não consegue omitir em silêncio uma seção que existe em inglês.",
            },
          ],
        },
        {
          id: "design",
          heading: "Sistema de design",
          blocks: [
            {
              kind: "paragraph",
              text: "O visual é caloroso e técnico: papel creme ou preto puro, bordas finas, fonte monoespaçada para tudo que é técnico e uma cor de destaque por vez. As cores são variáveis CSS semânticas, então os componentes nunca precisam de variantes para o modo escuro. As cores de destaque predefinidas são testadas quanto ao contraste com os fundos da página, a cor RGB escolhida por um visitante é ajustada em cada modo para continuar legível, e o modo claro ou escuro e a cor de destaque são aplicados antes da primeira renderização.",
            },
          ],
        },
        {
          id: "motion",
          heading: "Movimento",
          blocks: [
            {
              kind: "list",
              items: [
                "A exibição gradual das seções e a barra de progresso de leitura usam animações CSS guiadas por rolagem, que rodam fora da thread principal nos navegadores compatíveis.",
                "As estatísticas fazem a contagem crescente por meio de um inteiro registrado com `@property`, com o valor final já presente na marcação para tecnologias assistivas.",
                "A troca de tema se expande a partir do botão por meio da View Transitions API.",
                "O campo de contornos da página inicial é um pequeno shader WebGL que carrega depois que a página fica ociosa, roda apenas em navegadores desktop capazes, limita a taxa de quadros e pausa quando sai da tela.",
                "Uma configuração do site desativa tudo isso. As animações vêm ativadas por padrão e não seguem a configuração de movimento reduzido do sistema operacional.",
              ],
            },
          ],
        },
      ],
    },
    subsystems: {
      eyebrow: "Stack técnica",
      title: "Subsistemas",
      lede: "As partes do site que são mais interessantes do que um blog estritamente precisa.",
      sections: [
        {
          id: "live-chat",
          heading: "Chat ao vivo e chamadas",
          blocks: [
            {
              kind: "paragraph",
              text: "O chat funciona sobre um WebSocket com um codec binário compacto, moderação e histórico limitado. As chamadas de voz e vídeo passam por uma Selective Forwarding Unit que roda dentro do processo Axum sobre o webrtc-rs: cada navegador mantém uma conexão peer, e a SFU encaminha a mídia codificada sem transcodificação, por canais limitados por publicação e com um teto de participantes. [Abrir o chat ao vivo](/live-chat).",
            },
          ],
        },
        {
          id: "minecraft",
          heading: "Explorador de terreno do Minecraft",
          blocks: [
            {
              kind: "paragraph",
              text: "Um plugin do Paper expõe um socket UNIX do mesmo usuário para consultas limitadas ao mundo, com uma operação por vez e limites rígidos de tamanho de quadro. O [explorador](/minecraft) desenha os tiles do squaremap com seu próprio cliente Leaflet, em vez de executar o JavaScript do mapa gerado.",
            },
            {
              kind: "paragraph",
              text: "As sobreposições de bioma usam o **CYBM**, um formato binário compacto de tiles servido comprimido com gzip, ou um PNG cujo chunk privado `cyBM` carrega as identidades exatas dos biomas, de modo que o hover nunca adivinha a partir das cores dos pixels. As previsões a partir da semente para terreno inexplorado vêm de um worker separado de geração de mundo construído sobre o Pumpkin.",
            },
          ],
        },
        {
          id: "geo-ip",
          heading: "Consultas de Geo-IP",
          blocks: [
            {
              kind: "paragraph",
              text: "O [painel de visitantes](/visitor-board) e a [consulta de IP](/geo-ip-db) localizam endereços sem chamar uma API externa. Uma ferramenta de pré-processamento transforma os arquivos CSV do IP2Location LITE em `BTreeMap`s indexados pelo início de cada faixa, codifica-os com bitcode e os comprime com zstd no nível 22.",
            },
            {
              kind: "table",
              columns: [
                "Conjunto de dados",
                "CSV",
                "Pacote",
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
              text: "Na inicialização, o servidor decodifica um pacote por vez para limitar o pico de memória e internaliza os nomes de lugares repetidos. Uma consulta encontra a última faixa que começa no endereço ou abaixo dele e verifica o limite final dela.",
            },
          ],
        },
        {
          id: "wasm",
          heading: "WebAssembly",
          blocks: [
            {
              kind: "paragraph",
              text: "Os [pacotes de demo](/projects) enviados são gravados temporariamente em disco, identificados pelos seus magic bytes, limitados a 50 MiB descomprimidos e servidos a partir de um cache limitado dentro de frames em sandbox. O workspace também compila demos em Rust, incluindo um block breaker e um ray tracer.",
            },
            {
              kind: "paragraph",
              text: "O [EU5 Location DB](/eu5-locations-db) é uma aplicação Slint compilada para WebAssembly. A página envia o modo claro ou escuro por um pequeno protocolo de mensagens, de modo que a paleta embutida muda junto com o site.",
            },
          ],
        },
        {
          id: "photography",
          heading: "Pipeline de fotografia",
          blocks: [
            {
              kind: "paragraph",
              text: "Os envios são decodificados dentro de limites fixos de pixels e codificados em AVIF no preset mais lento, trocando tempo de processamento por arquivos menores. As localizações dos dados EXIF podem ser confirmadas em um mapa antes da publicação, e no máximo dois jobs de processamento rodam ao mesmo tempo.",
            },
          ],
        },
      ],
    },
    performance: {
      eyebrow: "Stack técnica",
      title: "Desempenho",
      lede: "Otimização guiada por perfil e BOLT no backend, medidos com um gerador de carga que não é o gargalo.",
      sections: [
        {
          id: "approach",
          heading: "Abordagem",
          blocks: [
            {
              kind: "paragraph",
              text: "Os builds otimizados são treinados com uma carga de trabalho representativa. Um binário instrumentado registra perfis contra uma fixture descartável com endpoints sintéticos de PostgreSQL, SMTP, S3, OpenID Connect e Minecraft. O LLVM recompila o servidor com esses perfis (PGO), e o BOLT então reordena o layout de código do binário final a partir de execução amostrada. Todas as variantes usam LTO, frame pointers e uma biblioteca padrão recompilada.",
            },
          ],
        },
        {
          id: "setup",
          heading: "Configuração",
          blocks: [
            {
              kind: "list",
              items: [
                "Servidor, provedores da fixture e PostgreSQL fixados em quatro núcleos de um AMD Ryzen 7 9800X3D; o cliente de carga fixado nos outros quatro.",
                "HTTPS sobre HTTP/1.1 com pool de 32 conexões, leitura completa das respostas e prazos de dois segundos.",
                "Três rodadas alternadas com cinco amostras por variante, cada uma após 10.000 requisições de aquecimento, a partir de um snapshot novo do banco de dados.",
                "27.000.000 de requisições cronometradas no total; todas retornaram HTTP 200.",
                "A carga vem do oha; o Playwright verifica a resposta de cada rota fora da janela cronometrada.",
              ],
            },
          ],
        },
        {
          id: "results",
          heading: "Resultados",
          blocks: [
            {
              kind: "bars",
              caption: "Mediana de requisições por segundo em 15 amostras por variante.",
              groups: [
                {
                  label: "Leituras públicas em dez rotas",
                  items: [
                    {
                      label: "Referência",
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
                  label: "JSON pequeno de health check",
                  items: [
                    {
                      label: "Referência",
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
              caption: "Latência de cauda para a mistura de dez rotas.",
              columns: [
                "Variante",
                "p95 (ms)",
                "p99 (ms)",
              ],
              numericColumns: [
                1,
                2,
              ],
              rows: [
                [
                  "Referência",
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
              text: "O PGO elevou a vazão mista em **17,9%** e o PGO com BOLT em **20,1%**, e ambos os builds otimizados venceram todas as rodadas alternadas. O BOLT acrescentou 1,8% sobre o PGO nas leituras mistas e ficou 0,8% mais lento na rota trivial de health check. Estes são resultados com quatro núcleos; a capacidade em produção também depende da rede, do conjunto de trabalho do banco de dados e do tráfego real.",
            },
          ],
        },
        {
          id: "corrections",
          heading: "Correções",
          blocks: [
            {
              kind: "list",
              items: [
                "`TCP_NODELAY` não estava definido no início: o acceptor Rustls padrão do axum-server o deixa desligado, então respostas HTTP/2 podiam esperar cerca de 40 ms por uma confirmação atrasada. Agora todo socket aceito o define.",
                "O primeiro benchmark gerava carga a partir do Playwright, cujo cliente ficava mais lento ao longo da execução e mostrou regressões de forma equivocada. Esses números foram descartados.",
              ],
            },
          ],
        },
      ],
    },
  },
};

export default content;
