import { render, screen } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";

import ForumTopicCard from "../components/forum/ForumTopicCard";
import ForumTopicDetailCard from "../components/forum/ForumTopicDetail";
import type { ForumTopic } from "../services/contracts/forum_types";

const topicFixture = (): ForumTopic => ({
  topic_id: "0198f4d0-aaaa-7000-8000-000000000001",
  author: {
    public_user_id: "0198f4d0-cccc-7000-8000-000000000001",
    display_name: "Forum user",
    country_code: 840,
    profile_picture_url: null,
    is_deleted: false,
  },
  title: "<script>bad()</script>",
  body: '<img src=x onerror="bad()">',
  content_state: "visible",
  access_state: "open",
  is_pinned: false,
  revision: 1,
  reply_count: 0,
  created_at: "2026-08-30T12:00:00Z",
  updated_at: "2026-08-30T12:00:00Z",
  last_activity_at: "2026-08-30T12:00:00Z",
  edited_at: null,
});

describe("forum rendering", () => {
  it("renders server content as plain text", () => {
    const result = render(() => <ForumTopicCard topic={topicFixture()} />);

    expect(screen.getByText("<script>bad()</script>")).toBeTruthy();
    expect(screen.getByText('<img src=x onerror="bad()">')).toBeTruthy();
    expect(result.container.querySelector("script")).toBeNull();
    expect(result.container.querySelector("img")).toBeNull();
  });

  it("does not render moderation controls without the backend capability", () => {
    render(() => (
      <ForumTopicDetailCard
        topic={topicFixture()}
        own={false}
        authenticated={true}
        canModerate={false}
        subscribed={false}
        onUpdate={() => Promise.resolve()}
        onDelete={() => Promise.resolve()}
        onSubscription={() => Promise.resolve()}
        onModerate={() => Promise.resolve()}
      />
    ));

    expect(screen.queryByText("Moderation")).toBeNull();
  });
});
