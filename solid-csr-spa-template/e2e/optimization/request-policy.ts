import { resolve } from "node:path";
import { readJson } from "./model";

interface RequestLimitPolicy {
  schema_version: number;
  profile: string;
  refill_interval_micros: number;
  burst_size: number;
}

/** The backend embeds this same fixed policy; its startup guards admit only disposable loopback fixtures. */
export function requestLimitPolicy(): Readonly<RequestLimitPolicy> {
  return Object.freeze(readJson<RequestLimitPolicy>(resolve(import.meta.dirname, "../../../tools/optimization/request-limits.json")));
}
