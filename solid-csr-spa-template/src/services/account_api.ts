import { createApiClient } from "../generated";
import { apiFetch } from "./api";

/** Generated API client bound to the application's credentialed transport. */
export const contractApi = createApiClient((path, init) =>
  apiFetch(path, init),
);

/** Compatibility name retained for existing account callers. */
export const accountContractApi = contractApi;
