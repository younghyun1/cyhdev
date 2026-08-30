import type { RouteSectionProps } from "@solidjs/router";
import type { JSX } from "@solidjs/web";
import TopBar from "./TopBar";

export default function Layout(props: RouteSectionProps): JSX.Element {
  return (
    <>
      <TopBar />
      <main class="transition-colors duration-90 bg-transparent text-slate-900 dark:text-slate-100 min-h-screen">
        {props.children}
      </main>
    </>
  );
}
