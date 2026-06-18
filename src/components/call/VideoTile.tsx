import { Show, createEffect } from "solid-js";
import { pageStyles } from "../../styles/pageStyles";

interface VideoTileProps {
  stream: MediaStream | null;
  label: string;
  profilePictureUrl?: string | null;
  countryFlag?: string | null;
  micOn: boolean;
  camOn: boolean;
  /** Mute audio playback (used for the local self tile to avoid echo). */
  muted?: boolean;
}

/// One participant tile. The `<video>` stays mounted whenever a stream exists so
/// audio keeps playing; the avatar overlays it when the camera is off.
export function VideoTile(props: VideoTileProps) {
  let videoEl: HTMLVideoElement | undefined;

  createEffect(() => {
    const stream = props.stream;
    if (videoEl) {
      videoEl.srcObject = stream;
    }
  });

  const showAvatar = () => !props.stream || !props.camOn;

  return (
    <div class={pageStyles.callTile}>
      <Show when={props.stream}>
        <video
          ref={videoEl}
          class={pageStyles.callVideo}
          autoplay
          playsinline
          muted={props.muted ?? false}
        />
      </Show>
      <Show when={showAvatar()}>
        <div class={pageStyles.callTileFallback}>
          <Show
            when={props.profilePictureUrl}
            fallback={
              <span class={pageStyles.callAvatar}>
                {props.label.slice(0, 1)}
              </span>
            }
          >
            <img
              src={props.profilePictureUrl ?? undefined}
              alt={props.label}
              class={`${pageStyles.callAvatar} object-cover`}
            />
          </Show>
        </div>
      </Show>
      <div class={pageStyles.callTileOverlay}>
        <span class="truncate">
          {props.label}
          <Show when={props.countryFlag}> {props.countryFlag}</Show>
        </span>
        <Show when={!props.micOn}>
          <span aria-label="muted" title="muted">
            🔇
          </span>
        </Show>
      </div>
    </div>
  );
}
