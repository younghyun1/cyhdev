import { Show } from "solid-js";
import { pageStyles } from "../../styles/pageStyles";
import { useRtc } from "../../state/rtc";

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
            Join call
          </button>
        }
      >
        <button
          class={pageStyles.buttonSecondary}
          type="button"
          onClick={rtc.toggleMic}
        >
          {rtc.micOn() ? "Mute" : "Unmute"}
        </button>
        <button
          class={pageStyles.buttonSecondary}
          type="button"
          onClick={rtc.toggleCamera}
        >
          {rtc.camOn() ? "Stop video" : "Start video"}
        </button>
        <button
          class={pageStyles.buttonDanger}
          type="button"
          onClick={rtc.leaveCall}
        >
          Leave
        </button>
        <Show when={rtc.callState() === "joining"}>
          <span class={pageStyles.subtitle}>Joining…</span>
        </Show>
      </Show>
    </div>
  );
}
