import { type Component, createEffect, onSettled } from "solid-js";

import { t } from "../state/i18n";
import { theme } from "../state/theme";
import "../styles/eu5-locations-db.css";

export const EU5_THEME_MESSAGE_PREFIX = "cyhdev:eu5-theme:";
export const EU5_THEME_READY_MESSAGE = "cyhdev:eu5-theme-ready";

export const serializeEu5Theme = (mode: "light" | "dark"): string =>
  `${EU5_THEME_MESSAGE_PREFIX}${mode}`;

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
  let frame: HTMLIFrameElement | undefined;

  const sendTheme = (mode: "light" | "dark"): void => {
    frame?.contentWindow?.postMessage(
      serializeEu5Theme(mode),
      window.location.origin,
    );
  };

  const handleThemeReady = (event: MessageEvent<unknown>): void => {
    const target = frame?.contentWindow;
    if (
      !target ||
      event.source !== target ||
      event.origin !== window.location.origin ||
      event.data !== EU5_THEME_READY_MESSAGE
    ) {
      return;
    }
    sendTheme(theme());
  };

  createEffect(
    () => theme(),
    (mode) => sendTheme(mode),
  );

  onSettled(() => {
    const topBar = document.querySelector<HTMLElement>('[data-site-bar="top"]');
    const bottomBar = document.querySelector<HTMLElement>(
      '[data-site-bar="bottom"]',
    );
    window.addEventListener("message", handleThemeReady);
    if (!page || !topBar || !bottomBar) {
      return () => window.removeEventListener("message", handleThemeReady);
    }

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
      window.removeEventListener("message", handleThemeReady);
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
        ref={(element) => {
          frame = element;
        }}
        class="eu5-locations-db-frame"
        src="/eu5-locations-db/app/index.html"
        title={t("top_bar.nav.eu5_locations_db")}
        loading="eager"
        sandbox="allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox allow-top-navigation-by-user-activation"
        onLoad={() => sendTheme(theme())}
      />
    </section>
  );
};

export default Eu5LocationsDb;
