// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  DeleteWasmModuleResponse,
  GetWasmModulesResponse,
  UpdateWasmModuleRequest,
  WasmModuleItem,
} from "../api-types";
import {
  interpolatePath,
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createWasmClient(transport: ApiTransport) {
  return {
    deleteWasmModule: async (input: {
      readonly path: {
        readonly wasm_module_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/wasm-modules/{wasm_module_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<DeleteWasmModuleResponse>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    getWasmModules: async (options: ApiRequestOptions = {}) => {
      const path = "/api/wasm-modules";
      const url = path;
      return requestJson<ApiResponse<GetWasmModulesResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    updateWasmModule: async (input: {
      readonly body: UpdateWasmModuleRequest;
      readonly path: {
        readonly wasm_module_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/wasm-modules/{wasm_module_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<WasmModuleItem>>(transport, url, {
        method: "PATCH",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    updateWasmModuleAssets: async (input: {
      readonly body: FormData;
      readonly path: {
        readonly wasm_module_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/wasm-modules/{wasm_module_id}/assets", input.path);
      const url = path;
      return requestJson<ApiResponse<WasmModuleItem>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
        body: input.body,
      });
    },
    uploadWasmModule: async (input: {
      readonly body: FormData;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/wasm-modules";
      const url = path;
      return requestJson<ApiResponse<WasmModuleItem>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
        body: input.body,
      });
    },
  } as const;
}
