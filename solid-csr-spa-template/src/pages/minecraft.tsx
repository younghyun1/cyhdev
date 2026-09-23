import { t } from "../state/i18n";
import "../styles/minecraft.css";

export default function Minecraft() {
  return (
    <section class="minecraft-page" aria-label={t("top_bar.nav.minecraft")}>
      {/* The map is written by the game server, so it runs in an opaque origin
          without access to this site's cookies or API. The backend enforces the
          same sandbox on direct navigation; clipboard access keeps its copy-link
          button working across that origin boundary. */}
      <iframe
        class="minecraft-map"
        src="/minecraft/map/"
        title={t("top_bar.nav.minecraft")}
        sandbox="allow-scripts allow-popups allow-popups-to-escape-sandbox"
        allow="clipboard-write *"
      />
    </section>
  );
}
