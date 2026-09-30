import { expect } from "@playwright/test";
import { call, value, another, type Scene } from "./support";

export async function protocols(name:string,scene:Scene):Promise<void> {
  await scene.page.goto(scene.campaign.base_url);
  if(name==="host.health-stats-websocket") {
    await call(scene,"GET","/api/healthcheck/server",undefined,{json_pointer:undefined,minimum_bytes:16,content_type:"application/json"});
    await call(scene,"GET","/api/healthcheck/state");
    await call(scene,"GET","/api/healthcheck/fastfetch");
    const length=await scene.page.evaluate(()=>new Promise<number>((resolve,reject)=>{
      const socket=new WebSocket(`${location.origin.replace("https:","wss:")}/ws/host-stats`);
      socket.binaryType="arraybuffer";
      const timer=setTimeout(()=>{socket.close();reject(new Error("Host sample deadline"));},10000);
      socket.onmessage=(event:MessageEvent<ArrayBuffer>)=>{clearTimeout(timer);socket.close();resolve(event.data.byteLength);};
      socket.onerror=()=>{clearTimeout(timer);socket.close();reject(new Error("Host sample socket failed"));};
    }));
    expect(length).toBe(20);
  } else if(name==="chat.history-send-moderation") {
    const id=await scene.page.evaluate(()=>new Promise<string>((resolve,reject)=>{
      const socket=new WebSocket(`${location.origin.replace("https:","wss:")}/ws/live-chat`);
      const timer=setTimeout(()=>{socket.close();reject(new Error("Chat acknowledgement deadline"));},10000);
      socket.onmessage=(event:MessageEvent<string>)=>{
        const message=JSON.parse(event.data) as {type:string;message?:{live_chat_message_id:string;message_body:string}};
        if(message.type==="hello") {
          socket.send(JSON.stringify({type:"typing",is_typing:true}));
          socket.send(JSON.stringify({type:"heartbeat",nonce:"fixture-heartbeat"}));
          socket.send(JSON.stringify({type:"send_message",client_message_id:"fixture-message",body:"Synthetic optimization chat"}));
        }
        if(message.type==="message_ack" && message.message?.message_body==="Synthetic optimization chat") {clearTimeout(timer);socket.close();resolve(message.message.live_chat_message_id);}
        if(message.type==="error") {clearTimeout(timer);socket.close();reject(new Error("Chat rejected synthetic message"));}
      };
      socket.onerror=()=>{clearTimeout(timer);socket.close();reject(new Error("Chat socket failed"));};
    }));
    const history=await call(scene,"GET","/api/live-chat/messages");
    expect(value(history,"/data/items/0/message_body")).toBe("Synthetic optimization chat");
    await call(scene,"GET","/api/admin/live-chat/cache-stats");
    await call(scene,"DELETE","/api/admin/live-chat/messages/{message_id}",undefined,{path:`/api/admin/live-chat/messages/${id}`});
  } else if(name==="rtc.signaling-call-media") {
    const {rtc}=await import("./rtc");
    await another(scene,"admin",async(other)=>{await rtc(scene,other);});
  } else {throw new Error(`Unknown protocol scenario ${name}`);}
}
