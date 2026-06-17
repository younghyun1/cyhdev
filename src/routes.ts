import { lazy } from "solid-js";
import type { RouteDefinition } from "@solidjs/router";

import Home from "./pages/home";
import { withAuth } from "./components/RequireAuth";

export const routes: RouteDefinition[] = [
  {
    path: "/",
    component: Home,
  },
  {
    path: "/about",
    component: lazy(() => import("./pages/about")),
  },
  {
    path: "/about-blog",
    component: lazy(() => import("./pages/about_blog")),
  },
  {
    path: "/find-password",
    component: lazy(() => import("./pages/find_password")),
  },
  {
    path: "/reset-password",
    component: lazy(() => import("./pages/reset_password")),
  },
  {
    path: "/blog",
    children: [
      {
        path: "/",
        component: lazy(() => import("./pages/posts/List")),
      },
      {
        path: "/new",
        component: withAuth(lazy(() => import("./pages/posts/New"))),
      },
      {
        path: "/:post_id/edit",
        component: withAuth(lazy(() => import("./pages/posts/Edit"))),
      },
      {
        path: "/:post_id",
        component: lazy(() => import("./pages/posts/View")),
      },
    ],
  },
  {
    path: "/visitor-board",
    component: lazy(() => import("./pages/visitor_board")),
  },
  {
    path: "/live-chat",
    component: lazy(() => import("./pages/live_chat")),
  },
  {
    path: "/users/:userName",
    component: lazy(() => import("./pages/user_info")),
  },
  {
    path: "/photographs",
    component: lazy(() => import("./pages/photographs")),
    // The detail view is a URL-synced modal: the param route keeps the gallery
    // page mounted (no remount) while /photographs/:photograph_id is active, so
    // infinite-scroll state and the open modal persist. The page reads the id
    // from the location; the child renders nothing.
    children: [
      { path: "/", component: () => null },
      { path: "/:photograph_id", component: () => null },
    ],
  },
  {
    path: "/projects",
    component: lazy(() => import("./pages/projects")),
  },
  {
    path: "/geo-ip-db",
    component: lazy(() => import("./pages/geo_ip_info")),
  },
  {
    path: "/backend-stats",
    component: lazy(() => import("./pages/backend_stats")),
  },
  {
    path: "/login",
    component: lazy(() => import("./pages/login")),
  },
  {
    path: "/register",
    component: lazy(() => import("./pages/signup")),
  },
  {
    path: "/edit-profile",
    component: withAuth(lazy(() => import("./pages/edit_profile"))),
  },
  {
    path: "/under-construction",
    component: lazy(() => import("./errors/404")),
  },
  {
    path: "/404",
    component: lazy(() => import("./errors/404")),
  },
  {
    path: "**",
    component: lazy(() => import("./errors/404")),
  },
];
