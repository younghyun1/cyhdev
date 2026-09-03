import LiveChatPanel from "../components/LiveChatPanel";
import { t } from "../state/i18n";
import { pageStyles } from "../styles/pageStyles";

export default function LiveChatPage() {
  return (
    <main class={`${pageStyles.page} live-chat-page`}>
      <div class={`${pageStyles.pageInner} live-chat-shell max-w-5xl`}>
        <div class="live-chat-page-heading mb-6">
          <h1 class={pageStyles.title}>{t("page.live_chat.title")}</h1>
          <p class={pageStyles.subtitle}>{t("page.live_chat.subtitle")}</p>
        </div>
        <LiveChatPanel mode="full" />
      </div>
    </main>
  );
}
