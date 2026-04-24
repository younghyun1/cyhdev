import { lazy } from "solid-js";
import type { RouteDefinition } from "@solidjs/router";

import Home from "./pages/home";

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
        component: lazy(() => import("./pages/posts/New")),
      },
      {
        path: "/:post_id/edit",
        component: lazy(() => import("./pages/posts/Edit")),
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
    path: "/photographs",
    component: lazy(() => import("./pages/photographs")),
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
    component: lazy(() => import("./pages/edit_profile")),
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
