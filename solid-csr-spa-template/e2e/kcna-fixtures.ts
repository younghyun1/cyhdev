/**
 * Responses of the KCNA archive endpoints for browser tests. The text covers
 * all six archive languages, a very long title, and a long category list, so
 * layout checks meet every script the archive holds.
 */

const NOW = "2026-11-21T08:00:00.000Z";

const uuid = (number: number): string => `00000000-0000-4000-8000-${String(number).padStart(12, "0")}`;

export const KCNA_DOCUMENT_ID = uuid(1);
export const KCNA_ENGLISH_ID = uuid(2);
export const KCNA_CHINESE_ID = uuid(3);

const LONG_TITLE =
  "A deliberately long unbroken headline identifier 1234567890123456789012345678901234567890 that must wrap inside a narrow result row without moving the page sideways";

const LONG_CATEGORY =
  "A category label that is much longer than the filter control can show on one line at a phone width";

type Language = "kor" | "eng" | "zho" | "rus" | "spa" | "jpn";
type Source = "kcna_kp" | "kcna_co_jp" | "stiles_archive";

const SAMPLES: ReadonlyArray<{
  readonly language: Language;
  readonly title: string;
  readonly snippet: string;
  readonly sources: ReadonlyArray<Source>;
}> = [
  { language: "kor", title: "새 다리 준공식 진행", snippet: "여러 단위의 일꾼들이 참가한 가운데 준공식이 진행되였다.", sources: ["kcna_kp", "kcna_co_jp"] },
  { language: "eng", title: "Bridge opening ceremony held", snippet: "The ceremony took place with workers from many units attending.", sources: ["kcna_kp"] },
  { language: "zho", title: "新桥竣工仪式举行", snippet: "许多单位的工作人员参加了竣工仪式。", sources: ["kcna_kp", "stiles_archive"] },
  { language: "rus", title: "Состоялась церемония открытия моста", snippet: "В церемонии приняли участие работники многих предприятий.", sources: ["kcna_kp"] },
  { language: "spa", title: "Celebrada la ceremonia de apertura del puente", snippet: "A la ceremonia asistieron trabajadores de muchas unidades.", sources: ["kcna_kp"] },
  { language: "jpn", title: "新しい橋の竣工式が行われた", snippet: "多くの部門の職員が参加して竣工式が行われた。", sources: ["kcna_kp", "kcna_co_jp"] },
  { language: "eng", title: LONG_TITLE, snippet: LONG_TITLE, sources: ["stiles_archive"] },
];

const segments = (text: string, query: string): ReadonlyArray<{ text: string; hit: boolean }> => {
  const at = query === "" ? -1 : text.toLowerCase().indexOf(query.toLowerCase());
  if (at < 0) return [{ text, hit: false }];
  return [
    { text: text.slice(0, at), hit: false },
    { text: text.slice(at, at + query.length), hit: true },
    { text: text.slice(at + query.length), hit: false },
  ].filter((part) => part.text !== "");
};

const searchResponse = (url: URL) => {
  const query = url.searchParams.get("q") ?? "";
  const offset = Number(url.searchParams.get("offset") ?? 0);
  const items = SAMPLES.map((sample, index) => ({
    id: uuid(offset + index + 1),
    title: segments(sample.title, query),
    snippet: segments(sample.snippet, query),
    publicationDate: `2026-${String(11 - (index % 9)).padStart(2, "0")}-${String(21 - index).padStart(2, "0")}`,
    language: sample.language,
    sources: sample.sources,
    category: index % 2 === 0 ? (index === 6 ? LONG_CATEGORY : "Economy") : null,
    hasFullText: true,
  }));
  return {
    total: 4210,
    offset,
    limit: 50,
    items,
    facets: {
      languages: [
        { value: "kor", count: 1800 },
        { value: "eng", count: 1500 },
        { value: "zho", count: 400 },
        { value: "rus", count: 300 },
        { value: "spa", count: 120 },
        { value: "jpn", count: 90 },
      ],
      sources: [
        { value: "kcna_kp", count: 3900 },
        { value: "kcna_co_jp", count: 800 },
        { value: "stiles_archive", count: 300 },
      ],
      categories: [
        { value: "Economy", count: 900 },
        { value: LONG_CATEGORY, count: 40 },
        ...Array.from({ length: 24 }, (_, index) => ({ value: `Category ${index + 1}`, count: 24 - index })),
      ],
    },
  };
};

const histogramResponse = () => ({
  bucket: "month",
  buckets: Array.from({ length: 96 }, (_, index) => {
    const month = index % 12;
    const year = 2018 + Math.floor(index / 12);
    return {
      start: `${year}-${String(month + 1).padStart(2, "0")}-01`,
      count: index % 17 === 0 ? 0 : 40 + ((index * 37) % 320),
    };
  }).filter((bucket) => bucket.count > 0),
});

const paragraph = (text: string, query: string) => segments(text, query);

const documentResponse = (id: string, query: string) => {
  const sample =
    id === KCNA_ENGLISH_ID ? SAMPLES[1] : id === KCNA_CHINESE_ID ? SAMPLES[2] : SAMPLES[0];
  const language = sample?.language ?? "kor";
  const body: Record<Language, ReadonlyArray<string>> = {
    kor: ["(평양 11월 21일발 조선중앙통신)", "새 다리 준공식이 21일 진행되였다.", "준공식에는 여러 단위의 일꾼들이 참가하였다."],
    eng: ["Pyongyang, November 21 (KCNA) -- The new bridge opening ceremony was held on Friday.", "Workers from many units attended the ceremony."],
    zho: ["【朝中社平壤11月21日电】新桥竣工仪式21日举行。", "许多单位的工作人员参加了仪式。"],
    rus: ["Пхеньян, 21 ноября (ЦТАК) -- Состоялась церемония открытия моста.", "В церемонии приняли участие работники многих предприятий."],
    spa: ["Pyongyang, 21 de noviembre (KCNA) -- Se celebró la ceremonia de apertura del puente."],
    jpn: ["【平壌11月21日発朝鮮中央通信】新しい橋の竣工式が21日に行われた。"],
  };
  return {
    id,
    title: segments(sample?.title ?? "", query),
    paragraphs: (body[language] ?? []).map((text) => paragraph(text, query)),
    publicationDate: "2026-11-21",
    publicationPrecision: "inferred_day",
    language,
    source: "kcna_kp",
    sources: sample?.sources ?? ["kcna_kp"],
    canonicalUrl: `http://www.kcna.kp/kp/article/q/${id}.kcmsf`,
    category: "Economy",
    sourceRecordId: "KCNA-2026-11-21-0001",
    metadata: { source: "fixture", page: 1, nested: { key: "value" } },
    translations: [
      { id: KCNA_ENGLISH_ID, language: "eng", title: SAMPLES[1]?.title ?? "" },
      { id: KCNA_CHINESE_ID, language: "zho", title: SAMPLES[2]?.title ?? "" },
    ].filter((translation) => translation.id !== id),
    copies: [
      { id: uuid(9), source: "kcna_co_jp", canonicalUrl: "http://www.kcna.co.jp/item/2026/202611/news21/20261121-01k.html" },
    ],
  };
};

const trendsResponse = (url: URL) => {
  const bucket = url.searchParams.get("bucket") === "month" ? "month" : "year";
  const terms = url.searchParams.getAll("term");
  const starts =
    bucket === "month"
      ? Array.from({ length: 36 }, (_, index) => `${2024 + Math.floor(index / 12)}-${String((index % 12) + 1).padStart(2, "0")}-01`)
      : Array.from({ length: 12 }, (_, index) => `${2015 + index}-01-01`);
  const totals = starts.map((_, index) => 800 + ((index * 53) % 400));
  return {
    bucket,
    starts,
    totals,
    series: terms.map((term, seriesIndex) => ({
      term,
      counts: totals.map((total, index) =>
        Math.round((total * (0.04 + ((index * (seriesIndex + 2)) % 9) / 100)) / (seriesIndex + 1)),
      ),
    })),
  };
};

const statusResponse = () => ({
  state: "ready",
  reloading: false,
  lastReloadFailed: false,
  snapshot: {
    documents: 412345,
    copies: 38210,
    indexed: 374135,
    translationGroups: 61200,
    oldestDate: "1996-03-11",
    newestDate: "2026-11-20",
    lastRetrievedAt: NOW,
    snapshotBuiltAt: NOW,
    memoryBytes: 1_610_612_736,
    languages: [
      { value: "kor", count: 180000 },
      { value: "eng", count: 150000 },
      { value: "zho", count: 20000 },
      { value: "rus", count: 12000 },
      { value: "spa", count: 7000 },
      { value: "jpn", count: 5135 },
    ],
    sources: [
      { value: "kcna_kp", count: 300000 },
      { value: "kcna_co_jp", count: 60000 },
      { value: "stiles_archive", count: 14135 },
    ],
  },
});

const envelope = (data: unknown) => ({ success: true, data, meta: { time_to_process: "1ms" } });

/**
 * The JSON body for an archive endpoint, or `undefined` when the path is not
 * one. Tests override single endpoints with their own routes for error states.
 */
export function kcnaArchiveBody(url: URL): unknown {
  const path = url.pathname;
  if (path === "/api/admin/kcna-archive/search") return envelope(searchResponse(url));
  if (path === "/api/admin/kcna-archive/histogram") return envelope(histogramResponse());
  if (path === "/api/admin/kcna-archive/trends") return envelope(trendsResponse(url));
  if (path === "/api/admin/kcna-archive/status" || path === "/api/admin/kcna-archive/reload") {
    return envelope(statusResponse());
  }
  const document = /^\/api\/admin\/kcna-archive\/documents\/([^/]+)$/.exec(path);
  if (document?.[1] !== undefined) {
    return envelope(documentResponse(decodeURIComponent(document[1]), url.searchParams.get("q") ?? ""));
  }
  return undefined;
}
