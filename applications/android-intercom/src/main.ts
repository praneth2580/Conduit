import { IntercomApp } from "./app";
import { conduitBridge } from "./conduit/bridge";

async function boot(): Promise<void> {
  const root = document.querySelector<HTMLElement>("#app");
  if (!root) return;

  try {
    const version = await conduitBridge.version();
    console.info(`Conduit native: ${version}`);
  } catch (err) {
    console.warn("Running without native plugin", err);
  }

  new IntercomApp(root);
}

void boot();
