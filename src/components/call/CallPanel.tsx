import { For, Show } from "solid-js";
import { pageStyles } from "../../styles/pageStyles";
import { useRtc } from "../../state/rtc";
import { CallControls } from "./CallControls";
import { VideoTile } from "./VideoTile";

/// Video grid + controls, shown above the message list in the full chat panel.
export function CallPanel() {
  const rtc = useRtc();
  const inCall = () =>
    rtc.callState() === "joining" || rtc.callState() === "in_call";

  return (
    <div class={pageStyles.callPanel}>
      <div class={pageStyles.callHeader}>
        <span class="text-sm font-semibold">Call</span>
        <Show when={rtc.participantCount() > 0}>
          <span class={pageStyles.callPill}>
            {rtc.participantCount()} in call
          </span>
        </Show>
      </div>

      <Show when={inCall()}>
        <div class={pageStyles.callGrid}>
          <VideoTile
            stream={rtc.localStream()}
            label="You"
            profilePictureUrl={rtc.selfActor()?.user_profile_picture_url ?? null}
            countryFlag={rtc.selfActor()?.country_flag ?? null}
            micOn={rtc.micOn()}
            camOn={rtc.camOn()}
            muted={true}
          />
          <For each={rtc.remoteTiles()}>
            {(tile) => (
              <VideoTile
                stream={tile.stream}
                label={tile.participant.actor.display_name}
                profilePictureUrl={tile.participant.actor.user_profile_picture_url}
                countryFlag={tile.participant.actor.country_flag}
                micOn={tile.participant.mic_on}
                camOn={tile.participant.cam_on}
              />
            )}
          </For>
        </div>
      </Show>

      <Show when={rtc.callError()}>
        <div class={pageStyles.callError}>{rtc.callError()}</div>
      </Show>

      <CallControls />
    </div>
  );
}
