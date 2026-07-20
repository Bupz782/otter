import { Wallet } from "lucide-react";
import { EmptyState } from "./EmptyState";
import { cn } from "@/lib/utils";

interface ConnectWalletStateProps {
  className?: string;
  /**
   * Compact inline variant used to gate a single action in demo mode
   * (e.g. "Connect wallet to set it live."). Rendered as a small amber
   * note instead of the full empty state.
   */
  message?: string;
}

export function ConnectWalletState({ className, message }: ConnectWalletStateProps) {
  if (message) {
    return (
      <div
        className={cn(
          "flex items-center gap-3 rounded-xl border border-amber-400/30 bg-amber-400/10 px-4 py-3",
          className
        )}
      >
        <Wallet className="h-4 w-4 shrink-0 text-amber-400" aria-hidden="true" />
        <p className="text-sm text-amber-400">{message}</p>
      </div>
    );
  }

  return (
    <EmptyState
      icon={<Wallet className="h-6 w-6" />}
      title="Connect wallet to continue"
      description="Your vault, intents, and delegations appear here once you connect and sign in."
      className={className}
    />
  );
}
