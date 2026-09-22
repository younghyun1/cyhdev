import { Show } from "solid-js";

interface UserBadgeProps {
  userName: string;
  profilePictureUrl?: string;
  countryFlag?: string | null;
  size?: "sm" | "md";
  link?: boolean;
}

export function UserBadge(props: UserBadgeProps) {
  const sizeClasses = () => {
    if (props.size === "md") {
      return {
        img: "w-6 h-6",
        text: "text-sm",
      };
    }
    // Default to "sm"
    return {
      img: "w-5 h-5",
      text: "text-xs",
    };
  };

  const href = () => `/users/${encodeURIComponent(props.userName)}`;
  const content = () => (
    <>
      <Show when={props.profilePictureUrl}>
        <img
          src={props.profilePictureUrl}
          alt={props.userName}
          class={`${sizeClasses().img} shrink-0 rounded-full object-cover border border-line`}
        />
      </Show>
      <Show when={!props.profilePictureUrl}>
        <span
          class={`${sizeClasses().img} inline-flex shrink-0 items-center justify-center rounded-full border border-line bg-surface-2 font-mono font-semibold uppercase text-ink-muted ${sizeClasses().text}`}
          aria-hidden="true"
        >
          {props.userName.slice(0, 1)}
        </span>
      </Show>
      <span
        class={`min-w-0 [overflow-wrap:anywhere] font-medium text-ink ${sizeClasses().text}`}
      >
        {props.userName}
      </span>
      <Show when={props.countryFlag}>
        <span class={`shrink-0 ${sizeClasses().text}`}>{props.countryFlag}</span>
      </Show>
    </>
  );

  return (
    <Show
      when={props.link !== false}
      fallback={<span class="inline-flex min-w-0 max-w-full items-center gap-1">{content()}</span>}
    >
      <a
        href={href()}
        class="relative z-10 inline-flex min-w-0 max-w-full items-center gap-1 rounded-sm no-underline hover:underline"
      >
        {content()}
      </a>
    </Show>
  );
}
