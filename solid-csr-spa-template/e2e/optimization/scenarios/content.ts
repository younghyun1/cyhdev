import { expect } from "@playwright/test";
import { call, value, another, login, type Scene } from "./support";

export async function content(name: string, scene: Scene): Promise<void> {
  const post = scene.variables.post_id;
  if (name === "blog.crud-publishing-search") {
    const created = await call(scene, "POST", "/api/blog/posts", { post_id: null, post_title: "Training draft", post_content: "# Fixture post\n\nTraining markdown **content**.", post_tags: ["fixture"], post_is_published: false });
    const id = value(created, "/data/post_id");
    await call(scene, "PATCH", "/api/blog/{post_id}", { post_title: "Training publication", post_content: "# Published fixture", post_tags: ["fixture", "training"], post_is_published: true }, { path: `/api/blog/${id}` });
    await call(scene, "GET", "/api/blog/posts");
    await call(scene, "GET", "/api/blog/posts/{post_id}", undefined, { path: `/api/blog/posts/${id}` });
    await call(scene, "GET", "/api/blog/search", undefined, { path: "/api/blog/search?q=fixture&page=1&limit=10" });
    await call(scene, "DELETE", "/api/blog/{post_id}", undefined, { path: `/api/blog/${id}` });
  } else if (name === "blog.comments-votes-shares") {
    const created = await call(scene, "POST", "/api/blog/{post_id}/comment", { is_guest: false, guest_id: null, guest_password: null, parent_comment_id: null, comment_content: "Synthetic comment" }, { path: `/api/blog/${post}/comment` });
    const id = value(created, "/data/comment_id");
    const path = `/api/blog/${post}/${id}`;
    const nested=await call(scene,"POST","/api/blog/{post_id}/comment",{is_guest:false,guest_id:null,guest_password:null,parent_comment_id:id,comment_content:"Synthetic nested reply"},{path:`/api/blog/${post}/comment`});
    const nestedId=value(nested,"/data/comment_id");
    await call(scene, "GET", "/api/blog/posts/{post_id}/comments", undefined, { path: `/api/blog/posts/${post}/comments` });
    await call(scene, "PATCH", "/api/blog/{post_id}/{comment_id}", { comment_content: "Updated synthetic comment" }, { path });
    for (const is_upvote of [true, false]) {
      await call(scene, "POST", "/api/blog/{post_id}/vote", { is_upvote }, { path: `/api/blog/${post}/vote` });
      await call(scene, "POST", "/api/blog/{post_id}/{comment_id}/vote", { is_upvote }, { path: `${path}/vote` });
    }
    await call(scene, "DELETE", "/api/blog/{post_id}/vote", undefined, { path: `/api/blog/${post}/vote` });
    await call(scene, "DELETE", "/api/blog/{post_id}/{comment_id}/vote", undefined, { path: `${path}/vote` });
    await call(scene,"DELETE","/api/blog/{post_id}/{comment_id}",undefined,{path:`/api/blog/${post}/${nestedId}`});
    await call(scene, "DELETE", "/api/blog/{post_id}/{comment_id}", undefined, { path });
    await call(scene, "GET", "/api/blog/posts/{post_id}", undefined, { path: `/api/blog/posts/${post}?shared=true` });
  } else if (name === "forum.topics-replies-revisions") {
    await call(scene, "GET", "/api/forum/capabilities");
    await call(scene, "GET", "/api/forum/topics", undefined, { path: "/api/forum/topics?search=synthetic&limit=10" });
    const created = await call(scene, "POST", "/api/forum/topics", { title: "Training discussion", body: "Synthetic body" });
    const id = value(created, "/data/topic_id");
    const path = `/api/forum/topics/${id}`;
    await call(scene, "GET", "/api/forum/topics/{topic_id}", undefined, { path });
    await call(scene, "PATCH", "/api/forum/topics/{topic_id}", { title: "Updated training discussion", body: "Updated body", expected_revision: 1 }, { path });
    await call(scene, "PATCH", "/api/forum/topics/{topic_id}", { title: "Stale training discussion", body: "Stale body", expected_revision: 1 }, { path, status: 409, json_pointer: undefined, minimum_bytes: 1 });
    const reply = await call(scene, "POST", "/api/forum/topics/{topic_id}/replies", { body: "Synthetic reply" }, { path: `${path}/replies` });
    const replyPath = `/api/forum/replies/${value(reply, "/data/reply_id")}`;
    await call(scene, "PATCH", "/api/forum/replies/{reply_id}", { body: "Updated reply", expected_revision: 1 }, { path: replyPath });
    await call(scene, "DELETE", "/api/forum/replies/{reply_id}", { expected_revision: 2 }, { path: replyPath });
    const latest=await call(scene,"GET","/api/forum/topics/{topic_id}",undefined,{path});
    await call(scene, "DELETE", "/api/forum/topics/{topic_id}", { expected_revision: Number(value(latest,"/data/topic/revision")) }, { path });
  } else if (name === "forum.moderation-subscriptions-notifications") {
    const path = `/api/forum/topics/${scene.variables.topic_id}`;
    await another(scene, "member", async (member) => {
      await call(member, "POST", "/api/forum/topics/{topic_id}/subscription", undefined, { path: `${path}/subscription` });
      await another(scene, "anonymous", async (other) => {
        await login(other, "fixture-other@example.test");
        const reply = await call(other, "POST", "/api/forum/topics/{topic_id}/replies", { body: "Notification fixture reply" }, { path: `${path}/replies` });
        scene.variables.moderated_reply_id = value(reply, "/data/reply_id");
      });
      const notifications = await call(member, "GET", "/api/forum/notifications");
      const id = value(notifications, "/data/notifications/0/notification_id");
      await call(member, "POST", "/api/forum/notifications/{notification_id}/read", undefined, { path: `/api/forum/notifications/${id}/read` });
      await call(member, "DELETE", "/api/forum/topics/{topic_id}/subscription", undefined, { path: `${path}/subscription` });
    });
    const latest=await call(scene,"GET","/api/forum/topics/{topic_id}",undefined,{path});
    let revision = Number(value(latest,"/data/topic/revision"));
    for (const action of ["hide", "restore", "lock", "unlock", "pin", "unpin"]) {
      const mutation = await call(scene, "POST", "/api/forum/topics/{topic_id}/moderation", { action, reason: "Synthetic training moderation", expected_revision: revision }, { path: `${path}/moderation` });
      revision = Number(value(mutation, "/data/revision"));
    }
    revision = 1;
    for (const action of ["hide", "restore"]) {
      const mutation = await call(scene, "POST", "/api/forum/replies/{reply_id}/moderation", { action, reason: "Synthetic training moderation", expected_revision: revision }, { path: `/api/forum/replies/${scene.variables.moderated_reply_id}/moderation` });
      revision = Number(value(mutation, "/data/revision"));
    }
    const audit = await call(scene, "GET", "/api/forum/moderation/audit");
    expect(audit).toBeTruthy();
  } else { throw new Error(`Unknown content scenario ${name}`); }
}
