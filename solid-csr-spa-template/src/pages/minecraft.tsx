import { createSignal, lazy, Show, Loading } from "solid-js";
import { isAuthenticated, isSuperuser } from "../state/auth";
import { t } from "../state/i18n";
import "../styles/minecraft.css";

const MinecraftControls = lazy(() => import("../components/minecraft/MinecraftControls"));

export default function Minecraft() {
  const [controlsOpen, setControlsOpen] = createSignal(false);
  return (
    <section class="minecraft-page" aria-label={t("top_bar.nav.minecraft")}>
      <iframe
        class="minecraft-map"
        src="/minecraft/map/"
        title={t("top_bar.nav.minecraft")}
        sandbox="allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox"
      />
      <Show when={isAuthenticated() === true && isSuperuser() === true}>
        <aside class="minecraft-admin">
          <button type="button" aria-expanded={controlsOpen() ? "true" : "false"} onClick={() => setControlsOpen(!controlsOpen())}>{t("minecraft.controls")}</button>
          <Show when={controlsOpen()}>
            <Loading fallback={<p>{t("common.loading")}</p>}><MinecraftControls /></Loading>
          </Show>
        </aside>
      </Show>
    </section>
  );
}
