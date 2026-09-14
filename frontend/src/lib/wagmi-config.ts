import { getDefaultConfig } from "@rainbow-me/rainbowkit";
import { anvil, sepolia } from "wagmi/chains";

// Local demo stack (no VITE_API_URL baked in) talks to anvil (chain 31337):
// include it so wallets connected to the local chain don't get RainbowKit's
// "wrong network" state. Deployed builds (VITE_API_URL set) are Sepolia-only.
const chains = import.meta.env.VITE_API_URL ? ([sepolia] as const) : ([sepolia, anvil] as const);

export const wagmiConfig = getDefaultConfig({
  appName: "Otter",
  projectId: import.meta.env.VITE_WALLET_CONNECT_PROJECT_ID || "otter-local",
  chains: chains as unknown as [typeof sepolia, ...typeof chains],
  ssr: false,
});
