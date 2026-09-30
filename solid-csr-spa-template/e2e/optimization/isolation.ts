import { realpathSync } from "node:fs";
import { resolve, sep } from "node:path";

const loopback = new Set(["127.0.0.1", "localhost", "[::1]"]);
/** Fail before fixture reset when inherited configuration can address real services. */
export function validateEnvironment(env: NodeJS.ProcessEnv, runtime: string): void {
  if (env.CYHDEV_OPT_DISPOSABLE !== "1") throw new Error("Set CYHDEV_OPT_DISPOSABLE=1 for a dedicated fixture environment");
  const database = new URL(env.DB_URL ?? "invalid:");
  if (!loopback.has(database.hostname) || !["postgres:", "postgresql:"].includes(database.protocol)
    || !/^\/cyhdev_optimization_[a-z0-9_]+$/.test(database.pathname) || database.search)
    throw new Error("DB_URL must select a loopback cyhdev_optimization_* fixture database without overrides");
  if (env.DB_HOST?.startsWith("/")) throw new Error("DB_HOST socket override would bypass the disposable DB_URL guard");
  if (!loopback.has(env.AWS_SES_SMTP_URL ?? "")) throw new Error("SMTP must terminate in a loopback fixture sink");
  if(env.OIDC_CLIENT_SECRET && !env.OIDC_CLIENT_SECRET.startsWith("optimization-fixture-")) throw new Error("OIDC must use synthetic client credentials");
  if(env.RTC_TURN_URL) throw new Error("RTC training cannot address an external TURN service");
  for (const name of ["AWS_ENDPOINT_URL", "OIDC_ISSUER_URL"]) {
    const url = new URL(env[name] ?? "invalid:");
    if (!loopback.has(url.hostname) || !["http:", "https:"].includes(url.protocol) || url.username || url.password)
      throw new Error(`${name} must select a loopback fixture service`);
  }
  if (env.AWS_ENDPOINT_URL_S3 && env.AWS_ENDPOINT_URL_S3 !== env.AWS_ENDPOINT_URL)
    throw new Error("S3 endpoint overrides must agree with the fixture endpoint");
  for (const name of ["AWS_IMAGE_UPLOAD_KEY", "AWS_IMAGE_UPLOAD_SECRET_KEY", "AWS_SES_SMTP_USERNAME", "AWS_SES_SMTP_ACCESS_KEY"])
    if (!env[name]?.startsWith("optimization-fixture-")) throw new Error(`${name} must use explicitly synthetic credentials`);
  for (const name of ["MINECRAFT_WORLD_SOCKET", "MINECRAFT_MAP_CONTROL_SOCKET", "SQUAREMAP_WEB_DIR", "SEARCH_INDEX_PATH","LOCAL_SMTP_CA_PEM"])
    if (env[name]) {
      const path = resolve(env[name]!);
      if (!path.startsWith(`${resolve(runtime)}${sep}`)) throw new Error(`${name} must remain under the disposable runtime directory`);
    }
}
export function validateFixturePaths(env: NodeJS.ProcessEnv, runtime: string): void {
  const root = realpathSync(runtime);
  for (const name of ["MINECRAFT_WORLD_SOCKET", "MINECRAFT_MAP_CONTROL_SOCKET", "SQUAREMAP_WEB_DIR","LOCAL_SMTP_CA_PEM"])
    if (env[name] && !realpathSync(env[name]!).startsWith(`${root}${sep}`))
      throw new Error(`${name} resolves outside the disposable runtime directory`);
}
