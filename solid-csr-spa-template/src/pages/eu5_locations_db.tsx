import type { Component } from "solid-js";

import { t } from "../state/i18n";
import "../styles/eu5-locations-db.css";

const Eu5LocationsDb: Component = () => (
  <section class="eu5-locations-db-page">
    <iframe
      class="eu5-locations-db-frame"
      src="/eu5-locations-db/app/index.html"
      title={t("top_bar.nav.eu5_locations_db")}
      loading="eager"
      sandbox="allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox allow-top-navigation-by-user-activation"
    />
  </section>
);

export default Eu5LocationsDb;
