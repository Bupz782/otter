import { X, Vault, Bot, ScanLine, Coins, ShieldCheck } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import type { ComparisonPoint } from "@/types/app";

const comparisons: ComparisonPoint[] = [
  {
    id: "vaults",
    title: "vs. vaults & yield aggregators",
    icon: "vault",
    comparison: "You define the condition",
    description:
      "Vaults ask you to trust a strategy manager. Otter lets you write the rule in plain English and enforce it cryptographically.",
  },
  {
    id: "bots",
    title: "vs. automation bots",
    icon: "bot",
    comparison: "Cryptographic limits, not API keys",
    description:
      "Bots need your private keys or API access. Otter uses signed delegations with hard limits that even Otter cannot exceed.",
  },
  {
    id: "intents",
    title: "vs. one-shot intents",
    icon: "scan",
    comparison: "Persistent, monitored delegation",
    description:
      "Most intents execute once. Otter keeps watching until your condition is met, then proves every action respects your limits.",
  },
  {
    id: "mev",
    title: "vs. MEV searchers",
    icon: "coins",
    comparison: "Rebates go to you",
    description:
      "Searchers keep MEV profits. Otter captures MEV through protected channels and rebates the majority back to depositors.",
  },
  {
    id: "custody",
    title: "vs. opaque custodians",
    icon: "shield",
    comparison: "Proof-of-solvency on-chain",
    description:
      "Custodians ask for trust. Otter publishes zero-knowledge proofs that the vault is solvent without revealing balances.",
  },
];

const iconMap: Record<string, React.ReactNode> = {
  vault: <Vault className="h-5 w-5" />,
  bot: <Bot className="h-5 w-5" />,
  scan: <ScanLine className="h-5 w-5" />,
  coins: <Coins className="h-5 w-5" />,
  shield: <ShieldCheck className="h-5 w-5" />,
};

export function WhyOtterPanel({ open, onClose }: { open: boolean; onClose: () => void }) {
  return (
    <AnimatePresence>
      {open && (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm"
            onClick={onClose}
          />
          <motion.aside
            initial={{ x: "100%" }}
            animate={{ x: 0 }}
            exit={{ x: "100%" }}
            transition={{ type: "spring", damping: 25, stiffness: 200 }}
            className="fixed right-0 top-0 z-50 h-screen w-full max-w-md border-l border-border/50 bg-card/95 backdrop-blur-xl p-6 shadow-2xl"
          >
            <div className="flex items-center justify-between">
              <div>
                <h2 className="font-heading text-2xl font-bold">Why Otter?</h2>
                <p className="text-sm text-muted-foreground">What makes us different.</p>
              </div>
              <Button variant="ghost" size="icon" onClick={onClose} aria-label="Close">
                <X className="h-5 w-5" />
              </Button>
            </div>

            <div className="mt-8 space-y-4">
              {comparisons.map((item, index) => (
                <motion.div
                  key={item.id}
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ delay: index * 0.08 }}
                  className="rounded-xl border border-border/60 bg-secondary/50 p-4 transition-colors hover:border-accent/30"
                >
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-full bg-accent-subtle text-accent">
                      {iconMap[item.icon]}
                    </div>
                    <div>
                      <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                        {item.title}
                      </p>
                      <p className="font-heading text-lg font-bold">{item.comparison}</p>
                    </div>
                  </div>
                  <p className="mt-3 text-sm text-muted-foreground">{item.description}</p>
                </motion.div>
              ))}
            </div>

            <div className="absolute bottom-0 left-0 right-0 border-t border-border/50 p-6">
              <p className="text-xs text-muted-foreground">
                All execution is mock in this preview. No real transactions are broadcast.
              </p>
            </div>
          </motion.aside>
        </>
      )}
    </AnimatePresence>
  );
}
