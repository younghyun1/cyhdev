import type {
  ApiResponse,
  UpdateWasmModuleRequest,
  WasmModuleItem,
} from "../../generated";
import { contractApi } from "../account_api";
import { uploadWithProgress } from "../upload_with_progress";

type ProgressOptions = {
  readonly onUploadProgress?: (percent: number) => void;
};

export const i18nApi = {
  getUiTextBundle: (locale: string) =>
    contractApi.getUiTextBundle({ query: { locale } }),
} as const;

export const adminApi = {
  syncI18nCache: () => contractApi.syncI18nCache(),
} as const;

export const wasmModuleApi = {
  getWasmModules: () => contractApi.getWasmModules(),
  uploadWasmModule: (
    formData: FormData,
    options: ProgressOptions = {},
  ): Promise<ApiResponse<WasmModuleItem>> =>
    options.onUploadProgress
      ? uploadWithProgress<WasmModuleItem>({
          url: "/api/wasm-modules",
          formData,
          onProgress: options.onUploadProgress,
        })
      : contractApi.uploadWasmModule({ body: formData }),
  updateWasmModule: (wasmModuleId: string, body: UpdateWasmModuleRequest) =>
    contractApi.updateWasmModule({
      body,
      path: { wasm_module_id: wasmModuleId },
    }),
  updateWasmModuleAssets: (
    wasmModuleId: string,
    formData: FormData,
    options: ProgressOptions = {},
  ): Promise<ApiResponse<WasmModuleItem>> =>
    options.onUploadProgress
      ? uploadWithProgress<WasmModuleItem>({
          url: `/api/wasm-modules/${encodeURIComponent(wasmModuleId)}/assets`,
          formData,
          onProgress: options.onUploadProgress,
        })
      : contractApi.updateWasmModuleAssets({
          body: formData,
          path: { wasm_module_id: wasmModuleId },
        }),
  deleteWasmModule: (wasmModuleId: string) =>
    contractApi.deleteWasmModule({
      path: { wasm_module_id: wasmModuleId },
    }),
} as const;
