import type { CapacitorConfig } from "@capacitor/cli";

const config: CapacitorConfig = {
  appId: "com.conduit.intercom",
  appName: "Conduit Intercom",
  webDir: "dist",
  android: {
    allowMixedContent: true,
  },
  plugins: {
    Conduit: {},
  },
};

export default config;
