import type { BrowserContext } from "@playwright/test";
import type { Campaign } from "./model";

/** Only external provider assets are redirected to their synthetic fixture equivalents. */
export async function isolateAssets(context: BrowserContext, campaign: Campaign): Promise<void> {
  await context.route("**/*",async(route)=>{
    const url=new URL(route.request().url());
    if(url.origin===new URL(campaign.base_url).origin || url.origin==="http://127.0.0.1:34901" || ["data:","blob:"].includes(url.protocol)) {await route.continue();return;}
    let provider:string | undefined;
    if(url.hostname==="cyhdev-img.s3.us-west-1.amazonaws.com" && /^\/(images|profile_pictures|thumbnails)\/[A-Za-z0-9_./-]+$/.test(url.pathname))
      provider=`http://127.0.0.1:34901/cyhdev-img${url.pathname}`;
    else if(/^(?:[abc]\.)?tile\.openstreetmap\.org$/.test(url.hostname)) provider="http://127.0.0.1:34901/__fixture/image";
    if(provider) {
      const response=await context.request.get(provider,{timeout:5000});
      try {await route.fulfill({response});} finally {await response.dispose();}
    } else {await route.abort();}
  });
}
