import { createSignal } from "solid-js";

export type ServerBuildInfo = {
  built_time?: string;
  name?: string;
  rust_version?: string;
};

const STORAGE_KEY = "server_build_info_v1";

const readCachedServerBuildInfo = (): ServerBuildInfo => {
  if (typeof window === "undefined") return {};
  try {
    const raw = window.sessionStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as ServerBuildInfo;
    return parsed ?? {};
  } catch {
    return {};
  }
};

const sanitize = (value?: string | null): string | undefined => {
  if (typeof value !== "string") return undefined;
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
};

export const [serverBuildInfo, setServerBuildInfo] =
  createSignal<ServerBuildInfo>(readCachedServerBuildInfo());

export const updateServerBuildInfo = (partial: Partial<ServerBuildInfo>) => {
  setServerBuildInfo((prev) => {
    const next: ServerBuildInfo = {
      built_time: sanitize(partial.built_time) ?? prev.built_time,
      name: sanitize(partial.name) ?? prev.name,
      rust_version: sanitize(partial.rust_version) ?? prev.rust_version,
    };
    try {
      window.sessionStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {
      // ignore storage errors
    }
    return next;
  });
};
