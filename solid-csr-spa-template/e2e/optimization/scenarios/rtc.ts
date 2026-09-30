import { expect, type Page } from "@playwright/test";
import type { Scene } from "./support";

declare global { interface Window { optimizationPeers:RTCPeerConnection[] } }

async function prepare(page:Page,origin:string):Promise<void> {
  await page.addInitScript(()=>{
    const Native=window.RTCPeerConnection;
    window.optimizationPeers=[];
    window.RTCPeerConnection=class extends Native {
      constructor(configuration?:RTCConfiguration) {super(configuration);window.optimizationPeers.push(this);}
    };
    localStorage.setItem("ui_locale","en-US");
  });
  await page.goto(`${origin}/live-chat`);
  await expect(page.getByRole("button",{name:"Join call",exact:true})).toBeEnabled();
}

async function received(page:Page):Promise<{audio:number;video:number}> {
  return page.evaluate(async()=>{
    let audio=0,video=0;
    for(const peer of window.optimizationPeers) {
      const stats=await peer.getStats();
      stats.forEach((report:RTCInboundRtpStreamStats)=>{
        if(report.type!=="inbound-rtp") return;
        if(report.kind==="audio") audio+=report.bytesReceived??0;
        if(report.kind==="video") video+=report.framesDecoded??0;
      });
    }
    return {audio,video};
  });
}

export async function rtc(first:Scene,second:Scene):Promise<void> {
  await prepare(first.page,first.campaign.base_url);
  await prepare(second.page,second.campaign.base_url);
  try {
    await first.page.getByRole("button",{name:"Join call",exact:true}).click();
    await expect(first.page.getByRole("button",{name:"Leave",exact:true})).toBeVisible();
    await second.page.getByRole("button",{name:"Join call",exact:true}).click();
    await expect(second.page.getByRole("button",{name:"Leave",exact:true})).toBeVisible();
    for(const page of [first.page,second.page]) {
      await expect.poll(async()=>{const stats=await received(page);return stats.audio>0&&stats.video>3;},{timeout:30000}).toBe(true);
      await page.getByRole("button",{name:"Mute",exact:true}).click();
      await page.getByRole("button",{name:"Unmute",exact:true}).click();
      await page.getByRole("button",{name:"Stop video",exact:true}).click();
      await page.getByRole("button",{name:"Start video",exact:true}).click();
    }
  } finally {
    for(const page of [first.page,second.page]) {const leave=page.getByRole("button",{name:"Leave",exact:true});if(await leave.isVisible()) await leave.click();}
  }
}
