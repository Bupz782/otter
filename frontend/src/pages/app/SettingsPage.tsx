import { motion } from "framer-motion";
import { RotateCcw, Wallet } from "lucide-react";
import { useAccount } from "wagmi";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useOnboardingContext } from "@/components/app/OnboardingProvider";

export function SettingsPage() {
  const { address, isConnected } = useAccount();
  const { restart } = useOnboardingContext();

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        <h1 className="font-heading text-3xl font-bold tracking-tight">Settings</h1>
        <p className="text-muted-foreground">App preferences and mock configuration.</p>
      </motion.div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Wallet className="h-5 w-5 text-accent" />
            Wallet
          </CardTitle>
          <CardDescription>Connected address and network.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="rounded-lg border border-border/60 bg-secondary p-4">
            <p className="text-xs text-muted-foreground">Status</p>
            <p className="font-medium">{isConnected ? "Connected (mock)" : "Disconnected"}</p>
          </div>
          {address && (
            <div className="rounded-lg border border-border/60 bg-secondary p-4">
              <p className="text-xs text-muted-foreground">Address</p>
              <code className="text-sm">{address}</code>
            </div>
          )}
          <p className="text-sm text-muted-foreground">
            This is a mocked wallet connection. No real signatures or transactions are broadcast.
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Onboarding</CardTitle>
          <CardDescription>Replay the guided tour.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between rounded-lg border border-border/60 bg-secondary p-4">
            <div>
              <p className="font-medium">Guided tour</p>
              <p className="text-xs text-muted-foreground">Restart the contextual tooltip tour from the Dashboard.</p>
            </div>
            <Button variant="outline" size="sm" onClick={restart} className="rounded-full">
              <RotateCcw className="mr-2 h-4 w-4" />
              Restart tour
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Mock mode</CardTitle>
          <CardDescription>All data and transactions are simulated.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between rounded-lg border border-border/60 bg-secondary p-4">
            <div>
              <p className="font-medium">Simulate latency</p>
              <p className="text-xs text-muted-foreground">Adds realistic network delays to mock API calls.</p>
            </div>
            <Button variant="outline" size="sm" disabled>On</Button>
          </div>
          <div className="flex items-center justify-between rounded-lg border border-border/60 bg-secondary p-4">
            <div>
              <p className="font-medium">Network</p>
              <p className="text-xs text-muted-foreground">Mocked Ethereum mainnet.</p>
            </div>
            <Button variant="outline" size="sm" disabled>Ethereum</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
