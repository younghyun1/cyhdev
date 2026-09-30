import { expect } from "@playwright/test";
import { setTimeout as delay } from "node:timers/promises";
import { pointer } from "../model";
import { call, value, another, login, FIXTURE, type Scene } from "./support";

export async function system(name: string, scene: Scene): Promise<void> {
  if (name === "authorization.roles-permissions-audit") {
    const roles = await call(scene,"GET","/api/admin/authorization/roles");
    const permissions = await call(scene,"GET","/api/admin/authorization/permissions");
    await call(scene,"GET","/api/admin/authorization/users");
    await call(scene,"GET","/api/admin/authorization/role-permissions");
    const rows = pointer(roles,"/data/roles") as {role_id:string;role_name:string}[];
    const role = rows.find((row)=>row.role_name === "moderator");
    const ordinary = rows.find((row)=>row.role_name === "user");
    if (!role || !ordinary) throw new Error("Synthetic roles are missing");
    scene.variables.ordinary_role_id = ordinary.role_id;
    const permission = value(permissions,"/data/permissions/0/permission_id");
    for (const enabled of [true,false]) await call(scene,"PATCH","/api/admin/authorization/roles/{role_id}/permissions/{permission_id}",
      {enabled,reason:"Synthetic authorization exercise",confirmed:true,confirmed_role_id:role.role_id,confirmed_permission_id:permission},
      {path:`/api/admin/authorization/roles/${role.role_id}/permissions/${permission}`});
    for (const role_id of [role.role_id,ordinary.role_id]) await call(scene,"PATCH","/api/admin/authorization/users/{user_id}/role",
      {role_id,reason:"Synthetic role transition",confirmed:true,confirmed_user_id:scene.variables.other_id ?? ""},
      {path:`/api/admin/authorization/users/${scene.variables.other_id}/role`});
    const audit=await call(scene,"GET","/api/admin/authorization/audit");
    expect((pointer(audit,"/data/events") as unknown[]).length).toBeGreaterThan(0);
  } else if (name === "authorization.denied-and-session-refresh") {
    await another(scene,"member",async(member)=>{
      await call(member,"GET","/api/admin/authorization/users",undefined,{status:403,json_pointer:undefined,minimum_bytes:1});
      await call(member,"POST","/api/admin/minecraft/map/waypoints",{world:"minecraft:overworld",name:"Denied",description:"",x:0,y:64,z:0},{status:403,json_pointer:undefined,minimum_bytes:1});
    });
    await another(scene,"anonymous",async(other)=>{
      await login(other,"fixture-other@example.test");
      await call(other,"GET","/api/auth/is-superuser",undefined,{json_pointer:"/data/is_superuser",equals:false});
      const roles=await call(scene,"GET","/api/admin/authorization/roles");
      const rows=pointer(roles,"/data/roles") as {role_id:string;role_name:string}[];
      const owner=rows.find((row)=>row.role_name==="younghyun");
      const ordinary=rows.find((row)=>row.role_name==="user");
      if(!owner || !ordinary) throw new Error("Synthetic authority roles missing");
      for(const [role_id,expected] of [[owner.role_id,true],[ordinary.role_id,false]] as const) {
        await call(scene,"PATCH","/api/admin/authorization/users/{user_id}/role",{role_id,reason:"Synthetic live-session refresh",confirmed:true,confirmed_user_id:scene.variables.other_id ?? ""},{path:`/api/admin/authorization/users/${scene.variables.other_id}/role`});
        await call(other,"GET","/api/auth/is-superuser",undefined,{json_pointer:"/data/is_superuser",equals:expected});
        await call(other,"GET","/api/admin/authorization/users",undefined,expected?{}:{status:403,json_pointer:undefined,minimum_bytes:1});
      }
    });
    await call(scene,"POST","/api/auth/oidc/login/start",undefined,{headers:{Origin:"https://invalid.example.test"},status:403,json_pointer:undefined,minimum_bytes:1});
  } else if (name === "geo.lookup-and-visitor-board") {
    await call(scene,"GET","/api/geo-ip-info/me");
    for(const ip of ["8.8.8.8","2001:4860:4860::8888"]) {
      await call(scene,"GET","/api/geo-ip-info/{ip_address}",undefined,{path:`/api/geo-ip-info/${encodeURIComponent(ip)}`});
      await call(scene,"GET","/api/geolocate/{ip_address}",undefined,{path:`/api/geolocate/${encodeURIComponent(ip)}`});
    }
    await call(scene,"GET","/api/visitor-board");
  } else if(name === "reference.countries-subdivisions-languages") {
    await call(scene,"GET","/api/dropdown/country");
    await call(scene,"GET","/api/dropdown/country/{country_id}",undefined,{path:`/api/dropdown/country/${scene.variables.country_id}`});
    await call(scene,"GET","/api/dropdown/country/{country_id}/subdivision",undefined,{path:`/api/dropdown/country/${scene.variables.country_id}/subdivision`});
    await call(scene,"GET","/api/dropdown/language");
    await call(scene,"GET","/api/dropdown/language/{language_id}",undefined,{path:`/api/dropdown/language/${scene.variables.language_id}`});
  } else if(name === "i18n.locales-refresh") {
    for(const locale of ["en-US","ko-KR"]) await call(scene,"GET","/api/i18n/ui-text",undefined,{path:`/api/i18n/ui-text?locale=${locale}`});
    await call(scene,"POST","/api/admin/sync-i18n-cache");
  } else if(name === "operations.retention-cleanup-purge") {
    await call(scene,"GET","/api/admin/media-cleanup/unresolved");
    await call(scene,"POST","/api/admin/media-cleanup/{cleanup_id}/resolve",{expected_original_url:"s3://cyhdev-img/images/fixture-cleanup.avif",bucket:"cyhdev-img",key:"images/fixture-cleanup.avif"},
      {path:`/api/admin/media-cleanup/${scene.variables.cleanup_id}/resolve`});
    await call(scene,"GET","/api/admin/account-retention-notifications");
    await call(scene,"POST","/api/admin/account-retention-notifications/{notification_id}/retry",undefined,{path:`/api/admin/account-retention-notifications/${scene.variables.notification_id}/retry`});
    let delivered=false;
    for(let attempt=0;attempt<75;attempt+=1) {
      const response=await fetch(`${FIXTURE}/__fixture/mail?recipient=fixture-retained%40example.test`,{signal:AbortSignal.timeout(5000)});
      if(response.ok) { const body:unknown=await response.json(); expect(value(body,"/message")).toContain("retention"); delivered=true;break; }
      await delay(1000);
    }
    expect(delivered).toBe(true);
    await call(scene,"POST","/api/admin/users/{user_id}/hard-purge",undefined,{path:`/api/admin/users/${scene.variables.purge_user_id}/hard-purge`});
  } else if(name === "runtime.startup-shutdown-background-jobs") {
    const state=await call(scene,"GET","/api/healthcheck/state");
    expect(Number(value(state,"/data/responses_handled"))).toBeGreaterThan(0);
    // The enclosing driver asserts successful graceful exit; scheduled delivery was asserted above.
  } else { throw new Error(`Unknown system scenario ${name}`); }
}
