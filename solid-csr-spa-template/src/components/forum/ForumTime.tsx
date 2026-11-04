import { locale } from "../../state/i18n";
import { formatRelativeTime } from "../../utils/relativeTime";

/** Relative time with the exact local timestamp on hover. */
export default function ForumTime(props: { readonly at: string }) {
  return (
    <time datetime={props.at} title={new Date(props.at).toLocaleString(locale())}>
      {formatRelativeTime(props.at, locale())}
    </time>
  );
}
