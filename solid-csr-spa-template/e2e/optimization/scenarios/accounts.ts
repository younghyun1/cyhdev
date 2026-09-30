import { expect } from "@playwright/test";
import { call, login, mailToken, value, uploadFile, PASSWORD, another, FIXTURE, type Scene } from "./support";

export async function accounts(name: string, scene: Scene): Promise<void> {
  const country = Number(scene.variables.country_id), language = Number(scene.variables.language_id);
  if (name === "accounts.signup-verification") {
    const email = "fixture-created@example.test";
    await call(scene, "POST", "/api/auth/signup", { user_name: "FixtureCreated", user_email: email, user_password: PASSWORD,
      user_country: country, user_language: language, user_subdivision: null }, { status: 202 });
    const token = await mailToken(email);
    await call(scene, "POST", "/api/auth/verify-user-email", { email_verification_token: token });
    await login(scene, email);
    const me = await call(scene, "GET", "/api/auth/me");
    scene.variables.created_user_id = value(me, "/data/user_info/user_id");
    expect(value(me, "/data/user_info/user_name")).toBe("FixtureCreated");
  } else if (name === "accounts.login-logout") {
    await login(scene, "fixture-member@example.test");
    await call(scene, "GET", "/api/auth/is-superuser");
    await call(scene, "GET", "/api/auth/me");
    await call(scene, "POST", "/api/auth/logout");
    await call(scene, "GET", "/api/auth/me", undefined, { json_pointer: "/data/user_info", equals: null });
  } else if (name === "accounts.password-reset") {
    await call(scene, "POST", "/api/auth/reset-password-request", { user_email: "fixture-created@example.test" });
    const token = await mailToken("fixture-created@example.test","reset-password");
    await call(scene, "POST", "/api/auth/reset-password", { password_reset_token: token, new_password: "OptimizationReset456" });
    await login(scene, "fixture-created@example.test", "OptimizationReset456");
    await call(scene, "POST", "/api/auth/reset-password", { password_reset_token: token, new_password: PASSWORD }, { status: 400, json_pointer: undefined, minimum_bytes: 1 });
  } else if (name === "accounts.profile-and-pictures") {
    await call(scene, "PATCH", "/api/auth/profile", { current_password: PASSWORD, user_name: "FixtureMember", user_country: country, user_language: language, user_subdivision: null });
    await call(scene, "GET", "/api/users/{user_name}", undefined, { path: "/api/users/FixtureMember" });
    for (let n = 0; n < 2; n += 1) await call(scene, "POST", "/api/user/upload-profile-picture", undefined, { multipart: { file: uploadFile("image.png", "image/png") } });
    const pictures = await call(scene, "GET", "/api/user/profile-pictures");
    const id = value(pictures, "/data/profile_pictures/0/profile_picture_id");
    await call(scene, "POST", "/api/user/profile-pictures/{profile_picture_id}/select", undefined, { path: `/api/user/profile-pictures/${id}/select` });
    await call(scene, "DELETE", "/api/user/profile-pictures/{profile_picture_id}", undefined, { path: `/api/user/profile-pictures/${id}` });
  } else if (name === "accounts.deletion-retention") {
    await login(scene, "fixture-created@example.test", "OptimizationReset456");
    await call(scene, "DELETE", "/api/auth/account", { current_password: "OptimizationReset456" });
    await call(scene, "GET", "/api/auth/me", undefined, { json_pointer: "/data/user_info", equals: null });
  } else if (name === "oidc.login-link-unlink") {
    const started = await call(scene, "POST", "/api/auth/oidc/link/start", { current_password: PASSWORD });
    const linked=await authorize(scene,value(started,"/data/authorization_url"));
    const token = new URLSearchParams(linked.hash.slice(1)).get("oidc_link_token");
    if (!token) throw new Error("OIDC link did not return a completion capability");
    scene.operations.add("GET /api/auth/oidc/callback");
    await call(scene, "POST", "/api/auth/oidc/link/complete", { completion_token: token });
    await call(scene, "GET", "/api/auth/oidc/status");
    await another(scene, "anonymous", async (guest) => {
      const loginStart = await call(guest, "POST", "/api/auth/oidc/login/start");
      const completed=await authorize(guest,value(loginStart,"/data/authorization_url"));
      expect(completed.hash).toBe("#oidc=success");
      await call(guest, "GET", "/api/auth/me",undefined,{json_pointer:"/data/user_info/user_id",equals:guest.variables.member_id??""});
    });
    await call(scene, "DELETE", "/api/auth/oidc/link", { current_password: PASSWORD });
    await another(scene,"anonymous",async(guest)=>{
      const rejected=await guest.context.request.get(`${scene.campaign.base_url}/api/auth/oidc/callback?code=invalid-fixture&state=invalid-fixture`,{maxRedirects:0});
      try {expect(rejected.status()).toBe(303);expect(new URL(rejected.headers().location??"invalid:").hash).toBe("#oidc=failed");}finally{await rejected.dispose();}
      await call(guest,"GET","/api/auth/me",undefined,{json_pointer:"/data/user_info",equals:null});
    });
  } else { throw new Error(`Unknown account scenario ${name}`); }
}

async function authorize(scene:Scene,target:string):Promise<URL> {
  const authorization=new URL(target);
  expect(authorization.origin).toBe(FIXTURE);
  const response=await scene.context.request.get(authorization.href,{maxRedirects:0});
  let callback:URL;
  try {expect(response.status()).toBe(303);callback=new URL(response.headers().location??"invalid:");}finally{await response.dispose();}
  expect(callback.origin).toBe(scene.campaign.base_url);
  expect(callback.pathname).toBe("/api/auth/oidc/callback");
  const finished=await scene.context.request.get(callback.href,{maxRedirects:0});
  try {expect(finished.status()).toBe(303);return new URL(finished.headers().location??"invalid:");}finally{await finished.dispose();}
}
