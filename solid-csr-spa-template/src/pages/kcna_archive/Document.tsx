import { createMemo, Errored, Loading, refresh, Show, type Component } from "solid-js";
import { useParams, useSearchParams } from "@solidjs/router";

import { DocumentSkeleton, Failure } from "../../components/kcna_archive/Notice";
import Reader from "../../components/kcna_archive/Reader";
import { useArchiveShortcuts } from "../../components/kcna_archive/useArchiveShortcuts";
import { kcnaArchiveApi } from "../../services/contracts/kcna_archive";
import { t } from "../../state/i18n";
import { parseQuery } from "../../utils/kcna_archive/query";
import { besideLanguageOf, documentHref, searchHref } from "../../utils/kcna_archive/routes";
import KcnaArchiveLayout from "./Layout";

/** One document on its own page: the link people cite, and the reader on narrow screens. */
const KcnaArchiveDocument: Component = () => {
  const route = useParams<{ documentId: string }>();
  const [params, setParams] = useSearchParams();

  // The search this page was opened from; the back link and the links across
  // translations keep it, and its text marks the matches.
  const query = createMemo(() => parseQuery(params));
  const besideLanguage = createMemo(() => besideLanguageOf(params.with));

  const opened = createMemo(async () => {
    const id = route.documentId;
    const text = query().text;
    return kcnaArchiveApi.document(id, text);
  });
  const beside = createMemo(async () => {
    const language = besideLanguage();
    const text = query().text;
    const link = opened().translations.find((translation) => translation.language === language);
    return link === undefined ? null : kcnaArchiveApi.document(link.id, text);
  });

  useArchiveShortcuts((action) => {
    if (action !== "toggle-translation") return false;
    const translations = opened().translations;
    const target = translations.find((entry) => entry.language === "eng") ?? translations[0];
    if (target === undefined) return false;
    setParams({ with: besideLanguage() === undefined ? target.language : undefined });
    return true;
  });

  return (
    <section class="kcna-document-page" aria-label={t("kcna.reader.label")}>
      <p class="kcna-back">
        <a href={searchHref(query(), { doc: route.documentId })}>
          <span class="kcna-back-arrow" aria-hidden="true">
            ←
          </span>{" "}
          {t("kcna.reader.back")}
        </a>
      </p>
      <Errored
        fallback={(error, reset) => (
          <Failure
            error={error()}
            onRetry={() => {
              void refresh(opened);
              reset();
            }}
          />
        )}
      >
        <Loading fallback={<DocumentSkeleton />}>
          <Show when={opened()}>
            {(current) => (
              <Reader
                document={current()}
                beside={beside() ?? undefined}
                besideLanguage={besideLanguage()}
                hrefFor={(id) => documentHref(id, query())}
                onBeside={(language) => setParams({ with: language })}
              />
            )}
          </Show>
        </Loading>
      </Errored>
    </section>
  );
};

/** The route component: the section frame around this tab's page. */
const KcnaArchiveDocumentPage: Component = () => (
  <KcnaArchiveLayout>
    <KcnaArchiveDocument />
  </KcnaArchiveLayout>
);

export default KcnaArchiveDocumentPage;
