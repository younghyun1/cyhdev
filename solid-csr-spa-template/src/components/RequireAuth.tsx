import { onSettled, Show, type Component, type ParentComponent } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { isAuthenticated } from "../state/auth";

const LoginRedirect: Component = () => {
  const navigate = useNavigate();
  onSettled(() => navigate("/login", { replace: true }));
  return null;
};

const RequireAuth: ParentComponent = (props) => {
  return (
    // isAuthenticated is null until the app bootstrap resolves; wait for non-null
    <Show when={isAuthenticated() !== null} fallback={null}>
      <Show when={isAuthenticated()} fallback={<LoginRedirect />}>
        {props.children}
      </Show>
    </Show>
  );
};

// HOC so routes.ts (a .ts file, no JSX) can wrap a lazy component cleanly.
export function withAuth(Inner: Component): Component {
  return () => (
    <RequireAuth>
      <Inner />
    </RequireAuth>
  );
}

export default RequireAuth;
