import { Show, type Component, type ParentComponent } from "solid-js";
import { Navigate } from "@solidjs/router";
import { isAuthenticated } from "../state/auth";

const RequireAuth: ParentComponent = (props) => (
  // isAuthenticated is null until App.onMount me() resolves; wait for non-null
  <Show when={isAuthenticated() !== null} fallback={null}>
    <Show when={isAuthenticated()} fallback={<Navigate href="/login" />}>
      {props.children}
    </Show>
  </Show>
);

// HOC so routes.ts (a .ts file, no JSX) can wrap a lazy component cleanly.
export function withAuth(Inner: Component): Component {
  return () => (
    <RequireAuth>
      <Inner />
    </RequireAuth>
  );
}

export default RequireAuth;
