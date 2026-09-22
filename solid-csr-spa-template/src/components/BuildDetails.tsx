import { t } from "../state/i18n";
import { serverBuildInfo } from "../state/server_info";
import { formatBuildTime } from "../utils/statusMetadata";

declare const __BUILD_TIMESTAMP__: string;
declare const __SOLID_VERSION__: string;
declare const __VITE_VERSION__: string;
declare const __TYPESCRIPT_VERSION__: string;

export default function BuildDetails() {
  return <>
    <div>{t("bottom_bar.fe")} · {t("bottom_bar.built")} {formatBuildTime(__BUILD_TIMESTAMP__)} · SolidJS {__SOLID_VERSION__} · TypeScript {__TYPESCRIPT_VERSION__} · Vite {__VITE_VERSION__}</div>
    <div>{t("bottom_bar.be")} · {t("bottom_bar.built")} {formatBuildTime(serverBuildInfo().built_time)} · Axum {serverBuildInfo().name?.replace(/^axum\s*/i, "") ?? "…"} · Rust {serverBuildInfo().rust_version?.replace(/^rustc\s*/i, "") ?? "…"}</div>
  </>;
}
