import type { ParentComponent } from "solid-js";
import { t } from "../state/i18n";
import { serverBuildInfo } from "../state/server_info";
import { formatBuildTime } from "../utils/statusMetadata";

declare const __BUILD_TIMESTAMP__: string;
declare const __SOLID_VERSION__: string;
declare const __VITE_VERSION__: string;
declare const __TYPESCRIPT_VERSION__: string;

/**
 * Left-to-right island for a value inside a translated status line: version
 * numbers, timestamps, and latencies keep their order, while the translated
 * labels around them follow the interface direction. Isolated, so a value never
 * reorders its neighbours.
 */
export const Ltr: ParentComponent = (props) => <bdi dir="ltr">{props.children}</bdi>;

export default function BuildDetails() {
  return <>
    <div>{t("bottom_bar.fe")} · {t("bottom_bar.built")} <Ltr>{formatBuildTime(__BUILD_TIMESTAMP__)}</Ltr> · <Ltr>SolidJS {__SOLID_VERSION__}</Ltr> · <Ltr>TypeScript {__TYPESCRIPT_VERSION__}</Ltr> · <Ltr>Vite {__VITE_VERSION__}</Ltr></div>
    <div>{t("bottom_bar.be")} · {t("bottom_bar.built")} <Ltr>{formatBuildTime(serverBuildInfo().built_time)}</Ltr> · <Ltr>Axum {serverBuildInfo().name?.replace(/^axum\s*/i, "") ?? "…"}</Ltr> · <Ltr>Rust {serverBuildInfo().rust_version?.replace(/^rustc\s*/i, "") ?? "…"}</Ltr></div>
  </>;
}
