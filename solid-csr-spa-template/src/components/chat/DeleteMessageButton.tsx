import { Show, createSignal } from "solid-js";
import { isSuperuser } from "../../state/auth";
import { t } from "../../state/i18n";
import { liveChatApi } from "../../services/live_chat";
import { pageStyles } from "../../styles/pageStyles";

export default function DeleteMessageButton(props: {
  messageId: string;
  onDeleted: (id: string) => void;
}) {
  const [busy, setBusy] = createSignal(false);
  const [failed, setFailed] = createSignal(false);
  const remove = async () => {
    if (busy() || !isSuperuser() || !window.confirm(t("live_chat.delete_confirm"))) return;
    setBusy(true);
    setFailed(false);
    try {
      await liveChatApi.deleteMessage(props.messageId);
      props.onDeleted(props.messageId);
    } catch {
      setFailed(true);
    } finally {
      setBusy(false);
    }
  };
  return (
    <Show when={isSuperuser()}>
      <button type="button" class={`${pageStyles.buttonGhost} text-danger`} disabled={busy()} onClick={() => void remove()}>
        {t("common.delete")}
      </button>
      <Show when={failed()}><span role="alert" class="text-danger">{t("live_chat.delete_failed")}</span></Show>
    </Show>
  );
}
