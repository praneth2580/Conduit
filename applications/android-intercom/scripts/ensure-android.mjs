import { existsSync } from "node:fs";
import { execSync } from "node:child_process";

if (!existsSync("android")) {
  console.log("Android platform not found — running: npx cap add android");
  execSync("npx cap add android", { stdio: "inherit" });
}
