// Generated from rust-be-template OpenAPI. Do not edit by hand.


export type DeleteWasmModuleResponse = {
  readonly cleanup_deleted_count: number;
  readonly cleanup_failure_count: number;
  readonly cleanup_remaining_count: number;
  readonly deleted_wasm_module_id: string;
  readonly unresolved_cleanup_count: number;
};
