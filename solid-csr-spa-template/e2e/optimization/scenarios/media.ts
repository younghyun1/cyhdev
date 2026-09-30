import { expect } from "@playwright/test";
import { setTimeout as delay } from "node:timers/promises";
import { call, value, uploadFile, type Scene } from "./support";
import { pointer } from "../model";

export async function media(name: string, scene: Scene): Promise<void> {
  if (name === "photographs.upload-batch-processing") {
    const multipart = { file: uploadFile("image.png", "image/png"), comments: "Synthetic upload", lat: "0", lon: "0", context: "photography" };
    const image = await call(scene, "POST", "/api/photographs/upload", undefined, { multipart });
    scene.variables.uploaded_photo_id = value(image, "/data/photograph_id");
    const batch = await call(scene, "POST", "/api/photographs/batch-upload", undefined, { status: 202,
      multipart: { file: uploadFile("image.png", "image/png"), meta: JSON.stringify([{ comment: "Synthetic batch", lat: 0, lon: 0 }]), context: "photography" } });
    const id = value(batch, "/data/batch_id");
    let complete = false;
    for (let n = 0; n < 100; n += 1) {
      const status = await call(scene, "GET", "/api/photographs/batch/{batch_id}", undefined, { path: `/api/photographs/batch/${id}` });
      expect(pointer(status, "/data/failed")).toBe(0);
      if (pointer(status, "/data/done") === true) { expect(pointer(status,"/data/completed")).toBe(1); complete = true; break; }
      await delay(200);
    }
    expect(complete).toBe(true);
    await call(scene, "GET", "/api/photographs/batches");
  } else if (name === "photographs.filters-details-metadata") {
    for (const path of ["/api/photographs/get?page=1&page_size=10", "/api/photographs/get?context=photography&page=1&page_size=2"]) await call(scene, "GET", "/api/photographs/get", undefined, { path });
    const detail = await call(scene, "GET", "/api/photographs/{photograph_id}", undefined, { path: `/api/photographs/${scene.variables.uploaded_photo_id}` });
    expect(detail).toBeTruthy();
    await scene.page.goto(`${scene.campaign.base_url}/photographs/${scene.variables.photograph_id}`);
    await expect(scene.page.locator(".details-info")).toBeVisible();
  } else if (name === "photographs.comments-votes-deletion") {
    const id = scene.variables.uploaded_photo_id;
    const created = await call(scene, "POST", "/api/photographs/{photograph_id}/comment", { parent_comment_id: null, comment_content: "Synthetic photograph comment" }, { path: `/api/photographs/${id}/comment` });
    const comment = value(created, "/data/photograph_comment_id");
    const path = `/api/photographs/${id}/${comment}`;
    await call(scene, "GET", "/api/photographs/{photograph_id}/comments", undefined, { path: `/api/photographs/${id}/comments` });
    await call(scene, "PATCH", "/api/photographs/{photograph_id}/{comment_id}", { comment_content: "Updated photograph comment" }, { path });
    for (const is_upvote of [true, false]) {
      await call(scene, "POST", "/api/photographs/{photograph_id}/vote", { is_upvote }, { path: `/api/photographs/${id}/vote` });
      await call(scene, "POST", "/api/photographs/{photograph_id}/{comment_id}/vote", { is_upvote }, { path: `${path}/vote` });
    }
    await call(scene, "DELETE", "/api/photographs/{photograph_id}/vote", undefined, { path: `/api/photographs/${id}/vote` });
    await call(scene, "DELETE", "/api/photographs/{photograph_id}/{comment_id}/vote", undefined, { path: `${path}/vote` });
    await call(scene, "DELETE", "/api/photographs/{photograph_id}/{comment_id}", undefined, { path });
    await call(scene, "DELETE", "/api/photographs/delete", { photograph_ids: [id ?? ""] });
  } else if (name === "wasm.upload-edit-serve-delete") {
    const uploaded = await call(scene, "POST", "/api/wasm-modules", undefined, { multipart: { bundle_file: uploadFile("demo.html", "text/html"), thumbnail: uploadFile("image.png", "image/png"), title: "Synthetic demo", description: "Native training fixture" } });
    const id = value(uploaded, "/data/wasm_module_id");
    scene.variables.wasm_module_id = id;
    await call(scene, "GET", "/api/wasm-modules");
    await call(scene, "PATCH", "/api/wasm-modules/{wasm_module_id}", { wasm_module_title: "Updated synthetic demo", wasm_module_description: "Updated native fixture" }, { path: `/api/wasm-modules/${id}` });
    await call(scene, "POST", "/api/wasm-modules/{wasm_module_id}/assets", undefined, { path: `/api/wasm-modules/${id}/assets`, multipart: { bundle_file: uploadFile("demo.html", "text/html") } });
    await call(scene, "GET", "/api/wasm-modules/{wasm_module_id}/wasm", undefined, { path: `/api/wasm-modules/${id}/wasm`, json_pointer: undefined, minimum_bytes: 16, content_type: "text/html" });
    await scene.page.goto(`${scene.campaign.base_url}/api/wasm-modules/${id}/wasm`);
    await expect(scene.page.locator("#fixture-demo")).toBeVisible();
    await call(scene, "POST", "/api/wasm-modules/{wasm_module_id}/assets", undefined, { path: `/api/wasm-modules/${id}/assets`, multipart: { wasm_file: uploadFile("demo.wasm", "application/wasm") } });
    const response = await scene.context.request.get(`${scene.campaign.base_url}/api/wasm-modules/${id}/wasm`);
    try { await WebAssembly.instantiate(await response.body()); } finally { await response.dispose(); }
    expect(pointer(uploaded, "/success")).toBe(true);
    await call(scene, "DELETE", "/api/wasm-modules/{wasm_module_id}", undefined, { path: `/api/wasm-modules/${id}` });
  } else { throw new Error(`Unknown media scenario ${name}`); }
}
