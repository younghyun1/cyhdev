import { Show } from "solid-js";

interface UserBadgeProps {
  userName: string;
  profilePictureUrl?: string;
  countryFlag?: string;
  size?: "sm" | "md";
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

  return (
    <span class="inline-flex items-center gap-1">
      <Show when={props.profilePictureUrl}>
        <img
          src={props.profilePictureUrl}
          alt={props.userName}
          class={`${sizeClasses().img} rounded-full object-cover border border-slate-200 dark:border-slate-700`}
        />
      </Show>
      <span
        class={`font-medium text-slate-900 dark:text-slate-300 ${sizeClasses().text}`}
      >
        {props.userName}
      </span>
      <Show when={props.countryFlag}>
        <span class={sizeClasses().text}>{props.countryFlag}</span>
      </Show>
    </span>
  );
}
