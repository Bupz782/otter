import { useEffect } from "react";
import { motion } from "framer-motion";
import { Vault, Pencil, ShieldCheck, ArrowRight, X } from "lucide-react";
import { Button } from "@/components/ui/button";

const cards = [
  {
    icon: <Vault className="h-6 w-6" />,
    title: "Your deposit stays in your vault",
    description:
      "Funds remain in the StrategyVault. Agents can only act within signed limits you control.",
  },
  {
    icon: <Pencil className="h-6 w-6" />,
    title: "Write rules in plain English",
    description:
      "Describe conditions like 'Lend 1000 USDC if yield > 3%'. Otter parses and enforces them.",
  },
  {
    icon: <ShieldCheck className="h-6 w-6" />,
    title: "Every action is proven",
    description:
      "Zero-knowledge proofs verify solvency and limit compliance on-chain, without exposing balances.",
  },
];

export function WelcomeModal({ onStart, onSkip }: { onStart: () => void; onSkip: () => void }) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onSkip();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onSkip]);

  return (
    <div
      className="fixed inset-0 z-[110] flex items-center justify-center bg-background/80 p-4 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-labelledby="welcome-title"
    >
      <motion.div
        initial={{ opacity: 0, scale: 0.95, y: 16 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.95, y: 16 }}
        className="relative w-full max-w-2xl rounded-2xl border border-accent/20 bg-card p-6 shadow-2xl md:p-8"
      >
        <button
          onClick={onSkip}
          className="absolute right-4 top-4 rounded p-1 text-muted-foreground hover:bg-secondary hover:text-foreground"
          aria-label="Close"
        >
          <X className="h-5 w-5" />
        </button>

        <div className="text-center">
          <h2 id="welcome-title" className="font-heading text-3xl font-bold">
            Welcome to Otter
          </h2>
          <p className="mt-2 text-muted-foreground">Three things to know before you start.</p>
        </div>

        <div className="mt-8 grid gap-4 sm:grid-cols-3">
          {cards.map((card, i) => (
            <motion.div
              key={card.title}
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: i * 0.1 }}
              className="rounded-xl border border-border/60 bg-secondary/50 p-4"
            >
              <div className="flex h-10 w-10 items-center justify-center rounded-full bg-accent-subtle text-accent">
                {card.icon}
              </div>
              <h3 className="mt-3 font-heading text-sm font-bold">{card.title}</h3>
              <p className="mt-1 text-xs text-muted-foreground">{card.description}</p>
            </motion.div>
          ))}
        </div>

        <div className="mt-8 flex flex-col-reverse gap-3 sm:flex-row sm:justify-end">
          <Button variant="ghost" onClick={onSkip}>
            Skip for now
          </Button>
          <Button onClick={onStart} className="rounded-full">
            Start the tour
            <ArrowRight className="ml-2 h-4 w-4" />
          </Button>
        </div>
      </motion.div>
    </div>
  );
}
