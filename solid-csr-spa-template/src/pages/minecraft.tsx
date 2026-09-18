import { t } from "../state/i18n";
import "../styles/minecraft.css";

export default function Minecraft() {
  return (
    <section class="minecraft-page" aria-label={t("top_bar.nav.minecraft")}>
      <iframe
        class="minecraft-map"
        src="/minecraft/map/"
        title={t("top_bar.nav.minecraft")}
        sandbox="allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox"
      />
    </section>
  );
}
