import {
  type Accessor,
  createEffect,
  createSignal,
  onSettled,
} from "solid-js";

import type {
  ForumReply,
  ForumReplyCursor,
} from "../../services/contracts/forum_types";

interface ReplyFragmentFollowerOptions {
  readonly replies: Accessor<ReadonlyArray<ForumReply>>;
  readonly cursor: Accessor<ForumReplyCursor | null>;
  readonly error: Accessor<string | null>;
  readonly loading: Accessor<boolean>;
  readonly maximumReplies: number;
  readonly loadPage: (cursor: ForumReplyCursor) => Promise<void>;
}

const REPLY_FRAGMENT_PATTERN =
  /^#forum-reply-[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/iu;

/** Follow stable notification fragments through bounded reply keysets. */
export function createReplyFragmentFollower(
  options: ReplyFragmentFollowerOptions,
): { reset: () => void } {
  let scrolledHash = "";
  let failedHash = "";
  const [locationHash, setLocationHash] = createSignal(window.location.hash);

  const captureHash = () => setLocationHash(window.location.hash);
  onSettled(() => {
    window.addEventListener("hashchange", captureHash);
    return () => window.removeEventListener("hashchange", captureHash);
  });

  createEffect(
    () => ({
      count: options.replies().length,
      cursor: options.cursor(),
      error: options.error(),
      hash: locationHash(),
      loading: options.loading(),
    }),
    ({ count, cursor, error, hash, loading }) => {
      if (hash === scrolledHash || !REPLY_FRAGMENT_PATTERN.test(hash)) return;
      if (hash === failedHash) {
        if (error !== null) return;
        failedHash = "";
      }
      if (error !== null) {
        failedHash = hash;
        return;
      }
      if (loading) return;
      const target = document.getElementById(hash.slice(1));
      if (target) {
        target.scrollIntoView({ block: "center" });
        scrolledHash = hash;
      } else if (cursor !== null && count < options.maximumReplies) {
        void options.loadPage(cursor);
      } else if (cursor === null || count >= options.maximumReplies) {
        scrolledHash = hash;
      }
    },
  );

  return {
    reset: () => {
      scrolledHash = "";
      failedHash = "";
    },
  };
}
