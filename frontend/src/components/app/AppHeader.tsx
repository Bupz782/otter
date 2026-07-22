import { useState, useRef, useEffect } from "react";
import { Menu, HelpCircle, Lightbulb, RotateCcw, ChevronDown, Play, Home } from "lucide-react";
import { useLocation, Link } from "react-router-dom";
import { AppConnectButton } from "./AppConnectButton";
import { Button } from "@/components/ui/button";
import { useOnboardingContext } from "./OnboardingProvider";
import { useAuthToken } from "@/hooks/useAuthToken";
import { useFocusTrap } from "@/hooks/useFocusTrap";

const breadcrumbMap: Record<string, string> = {
  "/app/dashboard": "Dashboard",
  "/app/intents": "Intents",
  "/app/intents/new": "Create Intent",
  "/app/delegations": "Delegations",
  "/app/delegations/new": "New Delegation",
  "/app/agents": "Otter Agents",
  "/app/proofs": "Proofs",
  "/app/settings": "Settings",
};

// Detail routes resolve to "<Section> / Detail" instead of falling back to "App".
// Checked after the exact map, so /app/intents/new still wins over /app/intents/:id.
const detailRoutes: { pattern: RegExp; section: { label: string; to: string } }[] = [
  { pattern: /^\/app\/intents\/[^/]+$/, section: { label: "Intents", to: "/app/intents" } },
  { pattern: /^\/app\/agents\/[^/]+$/, section: { label: "Agents", to: "/app/agents" } },
];

interface Crumb {
  label: string;
  to?: string;
}

function getCrumbs(pathname: string): Crumb[] {
  const root: Crumb = { label: "App", to: "/app/dashboard" };
  const exact = breadcrumbMap[pathname];
  if (exact) return [root, { label: exact }];
  const detail = detailRoutes.find((route) => route.pattern.test(pathname));
  if (detail)
    return [root, { label: detail.section.label, to: detail.section.to }, { label: "Detail" }];
  return [root, { label: "App" }];
}

export function AppHeader({ onMenuClick }: { onMenuClick?: () => void }) {
  const location = useLocation();
  const crumbs = getCrumbs(location.pathname);
  const { isAuthenticated } = useAuthToken();
  const [helpOpen, setHelpOpen] = useState(false);
  const [intentHelpOpen, setIntentHelpOpen] = useState(false);
  const helpRef = useRef<HTMLDivElement>(null);
  const helpButtonRef = useRef<HTMLButtonElement>(null);
  const intentHelpRef = useFocusTrap<HTMLDivElement>(intentHelpOpen);
  const { restart } = useOnboardingContext();

  useEffect(() => {
    if (!helpOpen) return;
    const handleClick = (e: MouseEvent) => {
      if (helpRef.current && !helpRef.current.contains(e.target as Node)) {
        setHelpOpen(false);
      }
    };
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setHelpOpen(false);
        helpButtonRef.current?.focus();
      }
    };
    document.addEventListener("mousedown", handleClick);
    document.addEventListener("keydown", handleKey);
    return () => {
      document.removeEventListener("mousedown", handleClick);
      document.removeEventListener("keydown", handleKey);
    };
  }, [helpOpen]);

  useEffect(() => {
    if (!intentHelpOpen) return;
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setIntentHelpOpen(false);
    };
    document.addEventListener("keydown", handleKey);
    return () => document.removeEventListener("keydown", handleKey);
  }, [intentHelpOpen]);

  return (
    <>
      <header className="glass-strong sticky top-0 z-40 flex h-14 items-center justify-between px-4 md:px-8">
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
              {crumbs.map((crumb, index) => {
                const isCurrent = index === crumbs.length - 1;
                return (
                  <li key={`${crumb.label}-${index}`} className="flex items-center gap-2">
                    {index > 0 && (
                      <span aria-hidden="true" className="text-muted-foreground">
                        /
                      </span>
                    )}
                    {isCurrent || !crumb.to ? (
                      <span
                        aria-current={isCurrent ? "page" : undefined}
                        className="font-medium text-foreground"
                      >
                        {crumb.label}
                      </span>
                    ) : (
                      <Link
                        to={crumb.to}
                        className="rounded text-muted-foreground transition-colors hover:text-accent focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent"
                      >
                        {crumb.label}
                      </Link>
                    )}
                  </li>
                );
              })}
            </ol>
          </nav>
        </div>

        <div className="flex items-center gap-3">
          {!isAuthenticated && (
            <span
              className="flex items-center gap-1.5 rounded-full border border-amber-400/30 bg-amber-400/10 px-3 py-1 text-xs font-medium text-amber-400"
              title="Showing demo data. Connect your wallet to go live."
              aria-label="Showing demo data. Connect your wallet to go live."
            >
              <span className="h-1.5 w-1.5 rounded-full bg-amber-400" aria-hidden="true" />
              Demo data
            </span>
          )}
          <div className="relative" ref={helpRef}>
            <Button
              ref={helpButtonRef}
              variant="ghost"
              size="sm"
              onClick={() => setHelpOpen((v) => !v)}
              className="text-muted-foreground hover:text-foreground"
              aria-haspopup="menu"
              aria-expanded={helpOpen}
            >
              <HelpCircle className="mr-2 h-4 w-4" />
              Help
              <ChevronDown className="ml-1 h-3 w-3" />
            </Button>
            {helpOpen && (
              <div
                role="menu"
                aria-label="Help"
                className="absolute right-0 top-full mt-2 w-56 rounded-xl border border-border/60 bg-card p-1 shadow-lg"
              >
                <button
                  role="menuitem"
                  onClick={() => {
                    setHelpOpen(false);
                    restart();
                  }}
                  className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-foreground hover:bg-secondary focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent"
                >
                  <RotateCcw className="h-4 w-4 text-accent" />
                  Take the tour
                </button>
                <button
                  role="menuitem"
                  onClick={() => {
                    setHelpOpen(false);
                    setIntentHelpOpen(true);
                  }}
                  className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-foreground hover:bg-secondary focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent"
                >
                  <Lightbulb className="h-4 w-4 text-accent" />
                  What's an intent?
                </button>
                <Link
                  role="menuitem"
                  to="/#demo"
                  onClick={() => setHelpOpen(false)}
                  className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-foreground hover:bg-secondary focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent"
                >
                  <Play className="h-4 w-4 text-accent" />
                  Open demo
                </Link>
                <Link
                  role="menuitem"
                  to="/"
                  onClick={() => setHelpOpen(false)}
                  className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-foreground hover:bg-secondary focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent"
                >
                  <Home className="h-4 w-4 text-accent" />
                  Back to landing
                </Link>
              </div>
            )}
          </div>
          <AppConnectButton />
        </div>
      </header>

      {intentHelpOpen && (
        <div
          className="fixed inset-0 z-[110] flex items-center justify-center bg-background/80 p-4 backdrop-blur-sm"
          onClick={() => setIntentHelpOpen(false)}
        >
          <div
            ref={intentHelpRef}
            role="dialog"
            aria-modal="true"
            aria-labelledby="intent-help-title"
            className="w-full max-w-md rounded-2xl border border-accent/20 bg-card p-6 shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center gap-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-full bg-accent-subtle text-accent">
                <Lightbulb className="h-5 w-5" />
              </div>
              <h3 id="intent-help-title" className="font-heading text-xl font-bold">
                What's an intent?
              </h3>
            </div>
            <p className="mt-4 text-sm text-muted-foreground">
              An intent is a conditional rule you write in plain English. Instead of signing every
              transaction, you describe what should happen and when. Otter monitors the condition,
              generates a proof that the action respects your limits, and executes it.
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
