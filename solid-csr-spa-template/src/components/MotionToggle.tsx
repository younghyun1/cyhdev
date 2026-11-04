import type { JSX } from "@solidjs/web";
import { t } from "../state/i18n";
import { motionEnabled, toggleMotion } from "../state/motion";

/** On/off switch for every decorative animation on the site. */
export default function MotionToggle(): JSX.Element {
  return (
    <button
      type="button"
      class="motion-switch"
      role="switch"
      aria-checked={motionEnabled() ? "true" : "false"}
      aria-label={t("motion.label")}
      onClick={toggleMotion}
    >
      <span class="motion-switch-thumb" aria-hidden="true" />
    </button>
  );
}
