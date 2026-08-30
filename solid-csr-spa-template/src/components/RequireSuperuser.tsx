import { onSettled, Show, type Component, type ParentComponent } from "solid-js";
import { useNavigate } from "@solidjs/router";

import { isSuperuser } from "../state/auth";

const AccessDeniedRedirect: Component = () => {
  const navigate = useNavigate();
  onSettled(() => navigate("/404", { replace: true }));
  return null;
};

const RequireSuperuser: ParentComponent = (props) => (
  <Show when={isSuperuser() !== null} fallback={null}>
    <Show when={isSuperuser() === true} fallback={<AccessDeniedRedirect />}>
      {props.children}
    </Show>
  </Show>
);

export function withSuperuser(Inner: Component): Component {
  return () => (
    <RequireSuperuser>
      <Inner />
    </RequireSuperuser>
  );
}

export default RequireSuperuser;
