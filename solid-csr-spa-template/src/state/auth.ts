import { createSignal } from "solid-js";
import type { MeResponse } from "../generated";

export const [isAuthenticated, setAuthenticated] = createSignal<boolean | null>(
  null,
);
export const [user, setUser] = createSignal<MeResponse | null>(null);
export const [isSuperuser, setSuperuser] = createSignal<boolean | null>(null);
