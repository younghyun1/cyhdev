import { describe, expect, it } from "vitest";

import { ApiContractError } from "../generated";
import { LOGIN_EMAIL_NOT_VERIFIED, apiErrorCode } from "../services/api_error_code";

describe("API error codes", () => {
  it("reads the numeric code from a failed contract response", () => {
    const error = new ApiContractError(
      403,
      JSON.stringify({ success: false, error_code: 68, message: "Verify first" }),
    );
    expect(apiErrorCode(error)).toBe(LOGIN_EMAIL_NOT_VERIFIED);
  });

  it("ignores bodies and errors without a numeric code", () => {
    expect(apiErrorCode(new ApiContractError(500, "not json"))).toBeNull();
    expect(apiErrorCode(new ApiContractError(403, JSON.stringify({ error_code: "68" })))).toBeNull();
    expect(apiErrorCode(new Error("network"))).toBeNull();
  });
});
