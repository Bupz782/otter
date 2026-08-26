import type { ReactNode } from "react";
import { LogOut, RotateCcw } from "lucide-react";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { motion } from "framer-motion";
import { useAccount } from "wagmi";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { DataRow } from "@/components/app/DataRow";
import { setAuthTokens } from "@/lib/api";
import { useAuthToken } from "@/hooks/useAuthToken";
import { useOnboardingContext } from "@/components/app/OnboardingProvider";
import { truncateHash } from "@/lib/utils";

const EASE: [number, number, number, number] = [0.22, 1, 0.36, 1];

/** Mount-only fade/slide used to stagger the page blocks. */
function FadeIn({
  children,
  delay = 0,
  className,
}: {
  children: ReactNode;
  delay?: number;
  className?: string;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4, ease: EASE, delay }}
      className={className}
    >
      {children}
    </motion.div>
  );
}

export function SettingsPage() {
  useDocumentTitle("Settings");
  const { address, isConnected, chain } = useAccount();
  const { isAuthenticated } = useAuthToken();
  const { restart } = useOnboardingContext();

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <FadeIn>
        <PageHeader title="Settings" subtitle="Your wallet, session, and the guided tour." />
      </FadeIn>

      <FadeIn delay={0.05}>
        <SectionCard title="Wallet" subtitle="Connected address and network.">
          {isConnected && address ? (
            <div className="space-y-4">
              <DataRow>
                <div className="min-w-0 flex-1">
                  <p className="text-xs text-muted-foreground">Address</p>
                  <p className="font-mono text-sm">{truncateHash(address)}</p>
                </div>
                <Badge variant="outline" className="shrink-0">
                  {chain?.name ?? "Unknown network"}
                </Badge>
              </DataRow>
              <p className="text-sm text-muted-foreground">
                Signatures happen in your wallet. Otter never holds your keys or moves funds without
                a signed delegation.
              </p>
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">
              Not connected. Connect from the header to link your vault.
            </p>
          )}
        </SectionCard>
      </FadeIn>

      <FadeIn delay={0.1}>
        <SectionCard title="Tour" subtitle="Replay the five-step walkthrough of the dashboard.">
          <Button variant="outline" onClick={restart} className="rounded-full">
            <RotateCcw className="mr-2 h-4 w-4" />
            Take the tour
          </Button>
        </SectionCard>
      </FadeIn>

      <FadeIn delay={0.15}>
        <SectionCard
          title="Session"
          subtitle="Sign-in ties your vault, intents, and delegations to this wallet."
        >
          {isAuthenticated ? (
            <Button variant="outline" onClick={() => setAuthTokens(null, null)} className="rounded-full">
              <LogOut className="mr-2 h-4 w-4" />
              Sign out
            </Button>
          ) : (
            <p role="status" className="text-sm text-muted-foreground">
              You are signed out. Connect and sign in from the header to continue.
            </p>
          )}
        </SectionCard>
      </FadeIn>
    </div>
  );
}
