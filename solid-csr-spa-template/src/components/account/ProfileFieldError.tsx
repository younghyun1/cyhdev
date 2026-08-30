import { Show } from "solid-js";

export default function ProfileFieldError(props: {
  readonly id: string;
  readonly message?: string;
}) {
  return (
    <Show when={props.message}>
      <p id={props.id} class="mt-1 text-sm text-danger" role="alert">
        {props.message}
      </p>
    </Show>
  );
}
