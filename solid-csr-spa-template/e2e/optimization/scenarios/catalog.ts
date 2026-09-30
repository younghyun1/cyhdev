import { accounts } from "./accounts";
import { content } from "./content";
import { media } from "./media";
import { system } from "./system";
import { protocols } from "./protocols";
import { minecraft } from "./minecraft";
import { browser } from "./browser";
import type { Scene } from "./support";

export async function runScenario(name:string,scene:Scene):Promise<void> {
  if(name.startsWith("accounts.")||name.startsWith("oidc.")) await accounts(name,scene);
  else if(name.startsWith("blog.")||name.startsWith("forum.")) await content(name,scene);
  else if(name.startsWith("photographs.")||name.startsWith("wasm.")) await media(name,scene);
  else if(name.startsWith("host.")||name.startsWith("chat.")||name.startsWith("rtc.")) await protocols(name,scene);
  else if(name.startsWith("minecraft.")) await minecraft(name,scene);
  else if(name.startsWith("browser.")) await browser(name,scene);
  else await system(name,scene);
}
