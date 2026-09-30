import { expect } from "@playwright/test";
import { setTimeout as delay } from "node:timers/promises";
import { call, value, FIXTURE, type Scene } from "./support";
import { pointer, type Json } from "../model";

export async function minecraft(name: string, scene: Scene): Promise<void> {
  if (name === "minecraft.dimensions-tiles-visibility") {
    for(const world of ["minecraft:overworld","minecraft:the_nether","minecraft:the_end"]) {
      const query={world,tile_x:0,tile_z:0,level:2,y:64};
      for(const format of ["",".bin",".png"]) {
        await call(scene,"POST",`/api/minecraft/map/seed-tile${format}`,query,format ? {json_pointer:undefined,minimum_bytes:64,content_type:format===".png"?"image/png":"application/vnd.cyhdev.biome-tile"}:{});
        await delay(1100);
      }
    }
    await call(scene,"POST","/api/minecraft/map/prediction",{world:"minecraft:overworld",min_x:0,min_z:0,y:64});
    const first=await scene.context.request.get(`${scene.campaign.base_url}/minecraft/map/tiles/fixture.png`);
    try {
      expect(first.status()).toBe(200);
      expect((await first.body()).length).toBeGreaterThan(64);
      const etag=first.headers().etag;
      if(!etag) throw new Error("Squaremap cache validator missing");
      const cached=await scene.context.request.get(`${scene.campaign.base_url}/minecraft/map/tiles/fixture.png`,{headers:{"If-None-Match":etag}});
      try{expect(cached.status()).toBe(304);}finally{await cached.dispose();}
    } finally {await first.dispose();}
    const hidden=await fetch(`${FIXTURE}/__fixture/visibility`,{method:"POST",body:"false"});
    expect(hidden.ok).toBe(true);
    await delay(6000);
    const masked=await call(scene,"POST","/api/minecraft/map/seed-tile",{world:"minecraft:overworld",tile_x:0,tile_z:0,level:2,y:64});
    expect(pointer(masked,"/data/palette")).toEqual([]);
    const cells=pointer(masked,"/data/indices");
    if(!Array.isArray(cells) || cells.length!==4096 || !cells.every(cell=>cell===null)) throw new Error("Hidden tile leaked terrain cells");
    expect((await fetch(`${FIXTURE}/__fixture/visibility`,{method:"POST",body:"true"})).ok).toBe(true);
    await delay(6000);
    const restored=await call(scene,"POST","/api/minecraft/map/seed-tile",{world:"minecraft:overworld",tile_x:0,tile_z:0,level:2,y:64});
    expect((pointer(restored,"/data/palette") as unknown[]).length).toBeGreaterThan(0);
  } else if(name === "minecraft.hover-surveys-blocks") {
    await delay(1100);
    const catalog=await call(scene,"POST","/api/minecraft/map/query",{kind:"catalog"});
    expect(value(catalog,"/data/worlds/2/id")).toBe("minecraft:the_end");
    for(const world of ["minecraft:overworld","minecraft:the_nether","minecraft:the_end"]) {
      await delay(1100);
      const area=await call(scene,"POST","/api/minecraft/map/query",{kind:"area",world,chunk_x:0,chunk_z:0,width:2,height:2,y:64});
      expect(value(area,"/data/cells/0/biome")).toMatch(/^minecraft:/);
      await delay(1100);
      await call(scene,"POST","/api/minecraft/map/query",{kind:"blocks",world,chunk_x:0,chunk_z:0,width:1,height:1,block:"minecraft:diamond_ore",min_y:0,max_y:64});
    }
    await delay(1100);
    await call(scene,"POST","/api/minecraft/map/query",{kind:"area",world:"minecraft:overworld",chunk_x:0,chunk_z:0,width:9,height:9,y:64},{status:400,json_pointer:undefined,minimum_bytes:1});
  } else if(name === "minecraft.waypoint-crud") {
    const input={world:"minecraft:overworld",name:"Synthetic waypoint",description:"Optimization fixture",x:0,y:64,z:0};
    const created=await call(scene,"POST","/api/admin/minecraft/map/waypoints",input);
    const id=value(created,"/data/id");
    await call(scene,"GET","/api/minecraft/map/waypoints",undefined,{path:"/api/minecraft/map/waypoints?world=minecraft%3Aoverworld"});
    await call(scene,"PATCH","/api/admin/minecraft/map/waypoints/{waypoint_id}",{...input,name:"Updated waypoint",x:16},{path:`/api/admin/minecraft/map/waypoints/${id}`});
    await call(scene,"DELETE","/api/admin/minecraft/map/waypoints/{waypoint_id}",undefined,{path:`/api/admin/minecraft/map/waypoints/${id}`});
  } else if(name === "minecraft.server-controls-fixture") {
    const status=await call(scene,"GET","/api/admin/minecraft");
    const id=value(status,"/data/players/0/id");
    const actions:Json[]=[{action:"message",message:"Synthetic optimization message"},{action:"whitelist_add",name:"FixturePlayer"},
      {action:"whitelist_remove",name:"FixturePlayer"},{action:"whitelist_enable",enabled:true},{action:"kick",name:"FixturePlayer"},
      {action:"map_visibility",id,hidden:true},{action:"map_visibility",id,hidden:false},{action:"save"},{action:"restart"}];
    for(const action of actions) {
      await delay(11000);
      await call(scene,"POST","/api/admin/minecraft/actions",action,{json_pointer:"/data/acknowledged",equals:true});
      if(typeof action==="object" && action!==null && !Array.isArray(action) && action.action==="map_visibility") {
        await call(scene,"GET","/api/admin/minecraft",undefined,{json_pointer:"/data/players/0/map_hidden",equals:action.hidden});
      }
    }
  } else {throw new Error(`Unknown Minecraft scenario ${name}`);}
}
