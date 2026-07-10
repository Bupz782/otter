import { StrictMode, Suspense, lazy } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { MotionConfig } from "framer-motion";
import { WagmiProvider } from "wagmi";
import { RainbowKitProvider, darkTheme } from "@rainbow-me/rainbowkit";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "./index.css";
import { App } from "./App";
import { wagmiConfig } from "./lib/wagmi-config";
import { AppLayout } from "./components/app/AppLayout";

const DemoPage = lazy(() => import("./pages/DemoPage").then((m) => ({ default: m.DemoPage })));

const DashboardPage = lazy(() =>
  import("./pages/app/DashboardPage").then((m) => ({ default: m.DashboardPage }))
);
const IntentsPage = lazy(() =>
  import("./pages/app/IntentsPage").then((m) => ({ default: m.IntentsPage }))
);
const CreateIntentPage = lazy(() =>
  import("./pages/app/CreateIntentPage").then((m) => ({ default: m.CreateIntentPage }))
);
const IntentDetailPage = lazy(() =>
  import("./pages/app/IntentDetailPage").then((m) => ({ default: m.IntentDetailPage }))
);
const DelegationsPage = lazy(() =>
  import("./pages/app/DelegationsPage").then((m) => ({ default: m.DelegationsPage }))
);
const CreateDelegationPage = lazy(() =>
  import("./pages/app/CreateDelegationPage").then((m) => ({ default: m.CreateDelegationPage }))
);
const AgentsPage = lazy(() =>
  import("./pages/app/AgentsPage").then((m) => ({ default: m.AgentsPage }))
);
const AgentDetailPage = lazy(() =>
  import("./pages/app/AgentDetailPage").then((m) => ({ default: m.AgentDetailPage }))
);
const StrategiesPage = lazy(() =>
  import("./pages/app/StrategiesPage").then((m) => ({ default: m.StrategiesPage }))
);
const CreateStrategyPage = lazy(() =>
  import("./pages/app/CreateStrategyPage").then((m) => ({ default: m.CreateStrategyPage }))
);
const ProofsPage = lazy(() =>
  import("./pages/app/ProofsPage").then((m) => ({ default: m.ProofsPage }))
);
const SettingsPage = lazy(() =>
  import("./pages/app/SettingsPage").then((m) => ({ default: m.SettingsPage }))
);

const queryClient = new QueryClient();

export function AppRoutes() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<App />} />
        <Route
          path="/demo"
          element={
            <Suspense fallback={null}>
              <DemoPage />
            </Suspense>
          }
        />
        <Route path="/app" element={<AppLayout />}>
          <Route index element={<Navigate to="/app/dashboard" replace />} />
          <Route path="dashboard" element={<DashboardPage />} />
          <Route path="intents" element={<IntentsPage />} />
          <Route path="intents/new" element={<CreateIntentPage />} />
          <Route path="intents/:id" element={<IntentDetailPage />} />
          <Route path="delegations" element={<DelegationsPage />} />
          <Route path="delegations/new" element={<CreateDelegationPage />} />
          <Route path="agents" element={<AgentsPage />} />
          <Route path="agents/:agentId" element={<AgentDetailPage />} />
          <Route path="strategies" element={<StrategiesPage />} />
          <Route path="strategies/new" element={<CreateStrategyPage />} />
          <Route path="strategies/:strategyId" element={<StrategiesPage />} />
          <Route path="proofs" element={<ProofsPage />} />
          <Route path="settings" element={<SettingsPage />} />
          <Route path="marketplace" element={<Navigate to="/app/agents" replace />} />
          <Route path="social" element={<Navigate to="/app/strategies" replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <MotionConfig reducedMotion="user">
      <WagmiProvider config={wagmiConfig}>
        <QueryClientProvider client={queryClient}>
          <RainbowKitProvider theme={darkTheme()} modalSize="compact">
            <AppRoutes />
          </RainbowKitProvider>
        </QueryClientProvider>
      </WagmiProvider>
    </MotionConfig>
  </StrictMode>
);
