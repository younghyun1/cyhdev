import LiveChatPanel from "../components/LiveChatPanel";
import { pageStyles } from "../styles/pageStyles";

export default function LiveChatPage() {
  return (
    <main class={pageStyles.page}>
      <div class={`${pageStyles.pageInner} max-w-5xl`}>
        <div class="mb-6">
          <h1 class={pageStyles.title}>Live Chat</h1>
          <p class={pageStyles.subtitle}>
            Text chat for signed-in users and guests.
          </p>
        </div>
        <LiveChatPanel mode="full" />
      </div>
    </main>
  );
}
