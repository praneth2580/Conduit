import { registerPlugin } from "@capacitor/core";
import type { ConduitPlugin } from "./definitions";

const Conduit = registerPlugin<ConduitPlugin>("Conduit", {
  web: () => import("./web").then((m) => new m.ConduitWeb()),
});

export * from "./definitions";
export { Conduit };
