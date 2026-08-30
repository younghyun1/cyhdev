import { Show, createSignal, onSettled } from "solid-js";
import { useNavigate } from "@solidjs/router";

import ForumTopicForm from "../../components/forum/ForumTopicForm";
import { forumApi } from "../../services/contracts/forum";
import type { ForumCapabilitiesResponse } from "../../services/contracts/forum_types";
import { t } from "../../state/i18n";
import "../../styles/forum.css";

export default function NewForumTopicPage() {
  const navigate = useNavigate();
  const [capabilities, setCapabilities] =
    createSignal<ForumCapabilitiesResponse | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  onSettled(() => {
    void forumApi
      .capabilities()
      .then((response) => setCapabilities(response.data))
      .catch(() =>
        setCapabilities({
          authenticated: false,
          can_post: false,
          can_moderate: false,
        }),
      );
  });

  const createTopic = async (title: string, body: string): Promise<void> => {
    setBusy(true);
    setError(null);
    try {
      const response = await forumApi.createTopic({ title, body });
      navigate(`/forum/${response.data.topic_id}`);
    } catch {
      setError(t("forum.form.create_failed"));
    } finally {
      setBusy(false);
    }
  };

  return (
    <main class="forum-page">
      <div class="forum-shell forum-shell--narrow">
        <header class="forum-header">
          <h1 class="forum-heading">{t("page.forum.new_title")}</h1>
          <a class="forum-link-button" href="/forum">
            {t("forum.topic.back")}
          </a>
        </header>
        <Show when={error()}>
          {(message) => (
            <div class="forum-alert forum-alert--error" role="alert">
              {message()}
            </div>
          )}
        </Show>
        <Show when={capabilities() === null}>
          <p class="forum-alert" role="status">{t("forum.loading")}</p>
        </Show>
        <Show when={capabilities() !== null && !capabilities()?.can_post}>
          <p class="forum-alert forum-alert--error">
            {capabilities()?.authenticated
              ? t("forum.posting_unavailable")
              : t("forum.sign_in_to_post")}
          </p>
        </Show>
        <Show when={capabilities()?.can_post}>
          <ForumTopicForm
            idPrefix="forum-new-topic"
            busy={busy()}
            mode="create"
            onSubmit={createTopic}
            onCancel={() => navigate("/forum")}
          />
        </Show>
      </div>
    </main>
  );
}
