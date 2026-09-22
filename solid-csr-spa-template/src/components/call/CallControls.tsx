import { Show } from "solid-js";
import { pageStyles } from "../../styles/pageStyles";
import { useRtc } from "../../state/rtc";
import { t } from "../../state/i18n";

/// Join/leave plus mic and camera toggles for the active call.
export function CallControls() {
  const rtc = useRtc();
  const active = () =>
    rtc.callState() === "joining" || rtc.callState() === "in_call";

  return (
    <div class={pageStyles.callControls}>
      <Show
        when={active()}
        fallback={
          <button
            class={pageStyles.buttonPrimary}
            type="button"
            disabled={rtc.callState() === "joining"}
            onClick={() => void rtc.joinCall()}
          >
            {t("call.join")}
          </button>
        }
      >
        <button
          class={pageStyles.buttonSecondary}
          type="button"
          onClick={rtc.toggleMic}
        >
          {t(rtc.micOn() ? "call.mute" : "call.unmute")}
        </button>
        <button
          class={pageStyles.buttonSecondary}
          type="button"
          onClick={rtc.toggleCamera}
        >
          {t(rtc.camOn() ? "call.stop_video" : "call.start_video")}
        </button>
        <button
          class={pageStyles.buttonDanger}
          type="button"
          onClick={rtc.leaveCall}
        >
          {t("call.leave")}
        </button>
        <Show when={rtc.callState() === "joining"}>
          <span class={pageStyles.subtitle}>{t("call.joining")}</span>
        </Show>
      </Show>
    </div>
  );
}
