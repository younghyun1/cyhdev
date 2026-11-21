import { createSignal, For, onCleanup, Show, type Component } from "solid-js";

import type {
  ArchiveDocumentResponse,
  ArchiveLanguage,
} from "../../services/contracts/kcna_archive";
import { locale, t } from "../../state/i18n";
import { languageKey, precisionKey, sourceKey } from "../../utils/kcna_archive/constants";
import { formatDate } from "../../utils/kcna_archive/format";
import { pairParagraphs } from "../../utils/kcna_archive/parallel";
import { plainText } from "../../utils/kcna_archive/segments";
import Segments from "./Segments";

interface ReaderProps {
  readonly document: ArchiveDocumentResponse;
  /** The translation shown beside the document, when one is open. */
  readonly beside: ArchiveDocumentResponse | undefined;
  /** The language being loaded for the side-by-side view. */
  readonly besideLanguage: ArchiveLanguage | undefined;
  /** Link that opens another language edition in place of this one. */
  readonly hrefFor: (id: string) => string;
  readonly onBeside: (language: ArchiveLanguage | undefined) => void;
}

/** How long the copied confirmation stays on the citation button. */
const CITED_MS = 1600;

/**
 * One article. The text is in the article's own language whatever the
 * interface language is, so the body carries that language and reads left to
 * right with the site's script stacks.
 */
const Reader: Component<ReaderProps> = (props) => {
  const [cited, setCited] = createSignal(false);
  let citedTimer: number | undefined;
  onCleanup(() => window.clearTimeout(citedTimer));

  const citation = (): string => {
    const document = props.document;
    return `${plainText(document.title)}. ${t(sourceKey(document.source))}, ${document.publicationDate}. ${document.canonicalUrl}`;
  };

  const cite = async (): Promise<void> => {
    try {
      await navigator.clipboard.writeText(citation());
      setCited(true);
      window.clearTimeout(citedTimer);
      citedTimer = window.setTimeout(() => setCited(false), CITED_MS);
    } catch {
      // Clipboard access can be denied; the citation is then shown to copy by hand.
      window.prompt(t("kcna.reader.cite"), citation());
    }
  };

  return (
    <article class="kcna-document" data-document-id={props.document.id}>
      <p class="kcna-document-meta">
        <time datetime={props.document.publicationDate}>
          {formatDate(props.document.publicationDate, locale())}
        </time>
        <span>{t(languageKey(props.document.language))}</span>
        <span>{t(sourceKey(props.document.source))}</span>
      </p>
      <h2 class="kcna-document-title" lang={props.document.language} dir="ltr">
        <Segments segments={props.document.title} />
      </h2>
      <div class="kcna-document-actions">
        <a
          class="kcna-button kcna-button--link"
          href={props.document.canonicalUrl}
          target="_blank"
          rel="noreferrer noopener"
        >
          {t("kcna.reader.source")}
        </a>
        <button type="button" class="kcna-button" onClick={() => void cite()}>
          {cited() ? t("kcna.reader.cited") : t("kcna.reader.cite")}
        </button>
      </div>
      <Show when={props.document.translations.length > 0}>
        <div class="kcna-translations">
          <p class="kcna-label">{t("kcna.reader.translations")}</p>
          <ul class="kcna-translation-list">
            <For each={props.document.translations}>
              {(translation) => {
                const open = (): boolean => props.besideLanguage === translation.language;
                return (
                  <li class="kcna-translation">
                    <a
                      class="kcna-translation-link"
                      href={props.hrefFor(translation.id)}
                      title={translation.title}
                    >
                      <span class="kcna-translation-language">{t(languageKey(translation.language))}</span>
                      <span class="kcna-translation-title" lang={translation.language} dir="ltr">
                        {translation.title}
                      </span>
                    </a>
                    <button
                      type="button"
                      class="kcna-toggle kcna-toggle--wide"
                      aria-pressed={open() ? "true" : "false"}
                      title={open() ? t("kcna.reader.close_beside") : t("kcna.reader.beside")}
                      onClick={() => props.onBeside(open() ? undefined : translation.language)}
                    >
                      <span class="kcna-toggle-label">
                        {open() ? t("kcna.reader.close_beside") : t("kcna.reader.beside")}
                      </span>
                    </button>
                  </li>
                );
              }}
            </For>
          </ul>
        </div>
      </Show>
      <Show
        when={props.beside}
        fallback={
          <div class="kcna-document-body" lang={props.document.language} dir="ltr">
            <For
              each={props.document.paragraphs}
              fallback={<p class="kcna-empty kcna-empty--inline">{t("kcna.reader.no_body")}</p>}
            >
              {(paragraph) => (
                <p>
                  <Segments segments={paragraph} />
                </p>
              )}
            </For>
          </div>
        }
      >
        {(beside) => (
          <div class="kcna-document-body kcna-document-parallel" dir="ltr">
            <h3 class="kcna-parallel-title" lang={beside().language}>
              {plainText(beside().title)}
            </h3>
            <For each={pairParagraphs(props.document.paragraphs, beside().paragraphs)}>
              {([left, right]) => (
                <div class="kcna-parallel-row">
                  <p lang={props.document.language}>
                    <Segments segments={left ?? []} />
                  </p>
                  <p lang={beside().language}>
                    <Segments segments={right ?? []} />
                  </p>
                </div>
              )}
            </For>
          </div>
        )}
      </Show>
      <Show when={props.document.copies.length > 0}>
        <div class="kcna-copies">
          <p class="kcna-label">{t("kcna.reader.copies")}</p>
          <ul class="kcna-copy-list">
            <For each={props.document.copies}>
              {(copy) => (
                <li>
                  <a href={copy.canonicalUrl} target="_blank" rel="noreferrer noopener">
                    {t(sourceKey(copy.source))}
                  </a>
                </li>
              )}
            </For>
          </ul>
        </div>
      </Show>
      <dl class="kcna-facts">
        <Show when={props.document.category}>
          {(category) => (
            <>
              <dt>{t("kcna.reader.category")}</dt>
              <dd lang={props.document.language}>{category()}</dd>
            </>
          )}
        </Show>
        <dt>{t("kcna.reader.precision")}</dt>
        <dd>{t(precisionKey(props.document.publicationPrecision))}</dd>
        <dt>{t("kcna.reader.record")}</dt>
        <dd dir="ltr">{props.document.sourceRecordId ?? props.document.id}</dd>
        <dt>{t("kcna.reader.url")}</dt>
        <dd dir="ltr">{props.document.canonicalUrl}</dd>
      </dl>
      <details class="kcna-metadata">
        <summary title={t("kcna.reader.metadata")}>
          <span class="ui-clip">{t("kcna.reader.metadata")}</span>
        </summary>
        <pre dir="ltr">{JSON.stringify(props.document.metadata, null, 2)}</pre>
      </details>
    </article>
  );
};

export default Reader;
