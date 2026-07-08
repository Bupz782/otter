import { useState, useRef, useEffect } from "react";
import { Menu, HelpCircle, Lightbulb, RotateCcw, Sparkles, ChevronDown } from "lucide-react";
import { useLocation, Link } from "react-router-dom";
import { AppConnectButton } from "./AppConnectButton";
import { WhyOtterButton } from "./WhyOtterButton";
import { WhyOtterPanel } from "./WhyOtterPanel";
import { Button } from "@/components/ui/button";
import { useOnboardingContext } from "./OnboardingProvider";
import { cn } from "@/lib/utils";

const breadcrumbMap: Record<string, string> = {
  "/app/dashboard": "Dashboard",
  "/app/intents": "Intents",
  "/app/intents/new": "Create Intent",
  "/app/delegations": "Delegations",
  "/app/delegations/new": "New Delegation",
  "/app/agents": "Otter Agents",
  "/app/strategies": "Strategies",
  "/app/proofs": "Proofs",
  "/app/settings": "Settings",
};

export function AppHeader({ onMenuClick }: { onMenuClick?: () => void }) {
  const location = useLocation();
  const label = breadcrumbMap[location.pathname] ?? "App";
  const [whyOpen, setWhyOpen] = useState(false);
  const [helpOpen, setHelpOpen] = useState(false);
  const [intentHelpOpen, setIntentHelpOpen] = useState(false);
  const helpRef = useRef<HTMLDivElement>(null);
  const { restart } = useOnboardingContext();

  useEffect(() => {
    if (!helpOpen) return;
    const handleClick = (e: MouseEvent) => {
      if (helpRef.current && !helpRef.current.contains(e.target as Node)) {
        setHelpOpen(false);
      }
    };
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setHelpOpen(false);
    };
    document.addEventListener("mousedown", handleClick);
    document.addEventListener("keydown", handleKey);
    return () => {
      document.removeEventListener("mousedown", handleClick);
      document.removeEventListener("keydown", handleKey);
    };
  }, [helpOpen]);

  return (
    <>
      <header className="glass-strong sticky top-0 z-40 flex h-16 items-center justify-between px-4 md:px-8">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            className="md:hidden"
            onClick={onMenuClick}
            aria-label="Open navigation"
          >
            <Menu className="h-5 w-5" />
          </Button>
          <nav aria-label="Breadcrumb">
            <ol className="flex items-center gap-2 text-sm">
              <li>
                <Link to="/app/dashboard" className="text-muted-foreground transition-colors hover:text-accent">
                  App
                </Link>
              </li>
              <li className="text-muted-foreground">/</li>
              <li
                className={cn(
                  "font-medium",
                  location.pathname === "/app/dashboard" ? "text-foreground" : "text-muted-foreground"
                )}
              >
                {label}
              </li>
            </ol>
          </nav>
        </div>

        <div className="flex items-center gap-3">
          <div className="relative hidden sm:block" ref={helpRef}>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setHelpOpen((v) => !v)}
              className="text-muted-foreground hover:text-foreground"
            >
              <HelpCircle className="mr-2 h-4 w-4" />
              Help
              <ChevronDown className="ml-1 h-3 w-3" />
            </Button>
            {helpOpen && (
              <div className="absolute right-0 top-full mt-2 w-56 rounded-xl border border-border/60 bg-card p-1 shadow-lg">
                <button
                  onClick={() => {
                    setHelpOpen(false);
                    restart();
                  }}
                  className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-foreground hover:bg-secondary"
                >
                  <RotateCcw className="h-4 w-4 text-accent" />
                  Restart tour
                </button>
                <button
                  onClick={() => {
                    setHelpOpen(false);
                    setIntentHelpOpen(true);
                  }}
                  className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-foreground hover:bg-secondary"
                >
                  <Lightbulb className="h-4 w-4 text-accent" />
                  What's an intent?
                </button>
                <button
                  onClick={() => {
                    setHelpOpen(false);
                    setWhyOpen(true);
                  }}
                  className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-foreground hover:bg-secondary"
                >
                  <Sparkles className="h-4 w-4 text-accent" />
                  Why Otter?
                </button>
              </div>
            )}
          </div>
          <WhyOtterButton onClick={() => setWhyOpen(true)} />
          <AppConnectButton />
        </div>
      </header>

      <WhyOtterPanel open={whyOpen} onClose={() => setWhyOpen(false)} />

      {intentHelpOpen && (
        <div
          className="fixed inset-0 z-[110] flex items-center justify-center bg-background/80 p-4 backdrop-blur-sm"
          onClick={() => setIntentHelpOpen(false)}
        >
          <div
            className="w-full max-w-md rounded-2xl border border-accent/20 bg-card p-6 shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center gap-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-full bg-accent-subtle text-accent">
                <Lightbulb className="h-5 w-5" />
              </div>
              <h3 className="font-heading text-xl font-bold">What's an intent?</h3>
            </div>
            <p className="mt-4 text-sm text-muted-foreground">
              An intent is a conditional rule you write in plain English. Instead of signing every transaction, you
              describe what should happen and when. Otter monitors the condition, generates a proof that the action
              respects your limits, and executes it.
            </p>
            <div className="mt-6 flex justify-end">
              <Button onClick={() => setIntentHelpOpen(false)} className="rounded-full">
                Got it
              </Button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
