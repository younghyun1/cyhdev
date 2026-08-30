import { For } from "solid-js";
import {
  repositorySourceBaseUrl,
  repositorySourcePaths,
  repositorySourceUrl,
  type RepositorySourcePath,
} from "../config/sourceLinks";
import { pageStyles } from "../styles/pageStyles";

interface SourceReference {
  readonly label: string;
  readonly path: RepositorySourcePath;
}

interface SourceGroup {
  readonly title: string;
  readonly description: string;
  readonly references: readonly SourceReference[];
}

const sourceGroups = [
  {
    title: "Authentication and accounts",
    description:
      "The backend owns authentication use cases and persistence; the browser owns session presentation and navigation only.",
    references: [
      { label: "HTTP API", path: repositorySourcePaths.accountApi },
      { label: "Application service", path: repositorySourcePaths.accountService },
      { label: "Database repository", path: repositorySourcePaths.accountRepository },
      { label: "Domain model", path: repositorySourcePaths.accountDomain },
      { label: "Browser auth state", path: repositorySourcePaths.frontendAuthState },
      { label: "Login page", path: repositorySourcePaths.frontendLogin },
    ],
  },
  {
    title: "HTTP and API contracts",
    description:
      "Routing and static delivery remain in the backend; browser request and response types are generated from OpenAPI.",
    references: [
      { label: "Main router", path: repositorySourcePaths.serverRouter },
      { label: "Static asset server", path: repositorySourcePaths.staticAssetServer },
      { label: "Generated API client", path: repositorySourcePaths.generatedApiClient },
      { label: "Generated API types", path: repositorySourcePaths.generatedApiTypes },
    ],
  },
  {
    title: "Data and publishing",
    description:
      "The database schema, blog, and photography implementations cover the main persistent content paths.",
    references: [
      { label: "Database schema", path: repositorySourcePaths.databaseSchema },
      { label: "Blog backend", path: repositorySourcePaths.blogBackend },
      { label: "Blog frontend", path: repositorySourcePaths.blogFrontend },
      { label: "Photography backend", path: repositorySourcePaths.photographyBackend },
      { label: "Photography frontend", path: repositorySourcePaths.photographyFrontend },
    ],
  },
  {
    title: "Realtime and localization",
    description:
      "Live chat, calling, and localized interface text each have explicit backend and frontend entry points.",
    references: [
      { label: "Live chat backend", path: repositorySourcePaths.liveChatBackend },
      { label: "Live chat frontend", path: repositorySourcePaths.liveChatFrontend },
      { label: "Localization backend", path: repositorySourcePaths.backendI18n },
      { label: "Localization frontend", path: repositorySourcePaths.frontendI18n },
    ],
  },
  {
    title: "Build and runtime",
    description:
      "The workspace toolchain, root commands, and container definition are versioned with the application source.",
    references: [
      { label: "Container build", path: repositorySourcePaths.containerBuild },
      { label: "Root development commands", path: repositorySourcePaths.rootCommands },
      { label: "Workspace manifest", path: repositorySourcePaths.workspaceManifest },
      { label: "Rust toolchain", path: repositorySourcePaths.rustToolchain },
    ],
  },
] as const satisfies readonly SourceGroup[];

export function RepositorySourceMap() {
  return (
    <section class={pageStyles.cardPadded}>
      <h2 class={pageStyles.sourceMapTitle}>5) Source map</h2>
      <p class={pageStyles.sourceMapIntro}>
        {repositorySourceBaseUrl
          ? "These links resolve from paths relative to the monorepo root. They do not use line anchors, so ordinary edits do not invalidate them."
          : "These paths are relative to the monorepo root. Links become available after a published repository URL is configured."}
      </p>
      <div class={pageStyles.sourceGroupGrid}>
        <For each={sourceGroups}>
          {(group) => (
            <section class={pageStyles.sourceGroup}>
              <h3 class={pageStyles.sourceGroupTitle}>{group.title}</h3>
              <p class={pageStyles.sourceGroupDescription}>{group.description}</p>
              <ul class={pageStyles.sourceLinkList}>
                <For each={group.references}>
                  {(reference) => {
                    const sourceUrl = repositorySourceUrl(reference.path);
                    const content = (
                      <>
                        <span class={pageStyles.sourceLinkLabel}>
                          {reference.label}
                        </span>
                        <code class={pageStyles.sourceLinkPath}>
                          {reference.path}
                        </code>
                      </>
                    );
                    return (
                      <li>
                        {sourceUrl ? (
                          <a
                            href={sourceUrl}
                            class={pageStyles.sourceLink}
                            target="_blank"
                            rel="noopener noreferrer"
                          >
                            {content}
                          </a>
                        ) : (
                          <span class={pageStyles.sourceLink}>{content}</span>
                        )}
                      </li>
                    );
                  }}
                </For>
              </ul>
            </section>
          )}
        </For>
      </div>
    </section>
  );
}
