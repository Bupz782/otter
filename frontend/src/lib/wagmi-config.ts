import { getDefaultConfig } from "@rainbow-me/rainbowkit";
import { sepolia } from "wagmi/chains";

export const wagmiConfig = getDefaultConfig({
  appName: "Otter",
  projectId: import.meta.env.VITE_WALLET_CONNECT_PROJECT_ID || "otter-local",
  chains: [sepolia],
  ssr: false,
});
