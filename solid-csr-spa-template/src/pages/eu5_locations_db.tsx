import { type Component, onSettled } from "solid-js";

import { t } from "../state/i18n";
import "../styles/eu5-locations-db.css";

export const calculateViewportOffsets = (
  topBarBottom: number,
  bottomBarTop: number,
  viewportHeight: number,
): Readonly<{ top: number; bottom: number }> => ({
  top: Math.max(0, Math.ceil(topBarBottom)),
  bottom: Math.max(0, Math.ceil(viewportHeight - bottomBarTop)),
});

const Eu5LocationsDb: Component = () => {
  let page: HTMLElement | undefined;

  onSettled(() => {
    const topBar = document.querySelector<HTMLElement>('[data-site-bar="top"]');
    const bottomBar = document.querySelector<HTMLElement>(
      '[data-site-bar="bottom"]',
    );
    if (!page || !topBar || !bottomBar) return;

    const updateOffsets = () => {
      if (!page) return;
      const offsets = calculateViewportOffsets(
        topBar.getBoundingClientRect().bottom,
        bottomBar.getBoundingClientRect().top,
        window.innerHeight,
      );
      page.style.setProperty("--eu5-top-bar-offset", `${offsets.top}px`);
      page.style.setProperty(
        "--eu5-bottom-bar-offset",
        `${offsets.bottom}px`,
      );
    };
    updateOffsets();

    const observer =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(updateOffsets);
    observer?.observe(topBar);
    observer?.observe(bottomBar);
    window.addEventListener("resize", updateOffsets);
    return () => {
      observer?.disconnect();
      window.removeEventListener("resize", updateOffsets);
    };
  });

  return (
    <section
      ref={(element) => {
        page = element;
      }}
      class="eu5-locations-db-page"
    >
      <iframe
        class="eu5-locations-db-frame"
        src="/eu5-locations-db/app/index.html"
        title={t("top_bar.nav.eu5_locations_db")}
        loading="eager"
        sandbox="allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox allow-top-navigation-by-user-activation"
      />
    </section>
  );
};

export default Eu5LocationsDb;
