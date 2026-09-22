export function formatBuildTime(value: string | undefined): string {
  if (!value) return "…";
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) return "…";
  return date.toISOString().replace("T", " ").replace(/\.\d{3}Z$/, " UTC");
}

export function postgresVersion(value: string): string {
  return `PostgreSQL ${value.replace(/^PostgreSQL\s*/i, "")}`;
}

export function visitorPopupText(value: string): string {
  // Legacy catalogs contain bold tags; keep all content as text, never HTML.
  return value.replace(/<\/?b>/g, "");
}
