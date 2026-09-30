import { expect } from "@playwright/test";
import { call, uploadFile, value, type Scene } from "./support";

export async function browser(name:string,scene:Scene):Promise<void> {
  if(name==="browser.navigation-auth-guards") {
    await scene.page.goto(`${scene.campaign.base_url}/edit-profile`);
    await expect(scene.page).toHaveURL(/\/login/);
    await scene.page.goto(`${scene.campaign.base_url}/blog`);
    await expect(scene.page.locator(".blog-list-title-row")).toBeVisible();
    await scene.page.getByRole("link",{name:"Synthetic optimization fixture",exact:true}).first().click();
    await expect(scene.page.locator("main h1")).toHaveText("Synthetic optimization fixture");
    await scene.page.goBack();
    await expect(scene.page.locator(".blog-list-title-row")).toBeVisible();
  } else if(name==="browser.compression-cache-errors") {
    for(const encoding of ["identity","gzip","zstd"]) {
      const response=await scene.context.request.get(`${scene.campaign.base_url}/`,{headers:{"Accept-Encoding":encoding}});
      try {expect(response.status()).toBe(200);expect(response.headers()["content-type"]).toContain("text/html");expect((await response.body()).length).toBeGreaterThan(100);} finally{await response.dispose();}
    }
    await call(scene,"POST","/api/auth/login",{user_email:"fixture-member@example.test",user_password:"incorrect-password"},{status:400,json_pointer:"/success",equals:false});
    await call(scene,"POST","/api/auth/login",{user_email:"fixture-member@example.test",user_password:"OptimizationWrong123"},{status:401,json_pointer:"/success",equals:false});
    const missing=await scene.context.request.get(`${scene.campaign.base_url}/api/blog/posts/01990000-0000-7000-8000-000000000099`);
    try{expect(missing.status()).toBe(404);}finally{await missing.dispose();}
    const redirect=await scene.context.request.get(`http://127.0.0.1:${scene.campaign.redirect_port}/blog`,{maxRedirects:0});
    try{expect(redirect.status()).toBe(308);expect(redirect.headers().location).toBe(`${scene.campaign.base_url}/blog`);}finally{await redirect.dispose();}
  } else if(name==="browser.eu5-and-wasm-embeds") {
    await scene.page.goto(`${scene.campaign.base_url}/eu5-locations-db`);
    const frame=scene.page.locator("iframe.eu5-locations-db-frame");
    await expect(frame).toBeVisible();
    expect(await frame.getAttribute("sandbox")).toContain("allow-scripts");
    const canvas=scene.page.frameLocator("iframe.eu5-locations-db-frame").locator("#canvas");
    await expect(canvas).toBeVisible();
    // Slint paints its interface into a canvas; DOM text cannot establish rendered content.
    await expect.poll(async()=> (await canvas.screenshot()).length,{timeout:30000}).toBeGreaterThan(5000);
    const populated=await canvas.screenshot();
    await canvas.click({position:{x:320,y:30}});
    await scene.page.keyboard.type("Stockholm");
    await expect.poll(async()=> (await canvas.screenshot()).length,{timeout:30000}).toBeLessThan(populated.length*0.85);
    if(process.env.CYHDEV_OPT_STAGE==="development") await canvas.screenshot({path:`${process.env.CYHDEV_OPT_RUN}/eu5-canvas.png`});
    const module=await call(scene,"POST","/api/wasm-modules",undefined,{multipart:{bundle_file:uploadFile("demo.html","text/html"),thumbnail:uploadFile("image.png","image/png"),title:"Synthetic embedded demo",description:"Optimization iframe fixture"}});
    const id=value(module,"/data/wasm_module_id");
    await scene.page.goto(`${scene.campaign.base_url}/projects`);
    await scene.page.getByText("Synthetic embedded demo",{exact:true}).click();
    await expect(scene.page.frameLocator(`iframe[src='/api/wasm-modules/${id}/wasm']`).locator("#fixture-demo")).toHaveText("Synthetic optimization demo");
    await call(scene,"DELETE","/api/wasm-modules/{wasm_module_id}",undefined,{path:`/api/wasm-modules/${id}`});
  } else {throw new Error(`Unknown browser scenario ${name}`);}
}
