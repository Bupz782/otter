import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";

const STEP_KEY = "otter-onboarding-step";
const WELCOME_KEY = "otter-onboarding-welcome";

export type OnboardingStep =
  | "dashboard-balance"
  | "dashboard-intents"
  | "dashboard-activity"
  | "dashboard-positions"
  | "intents-list"
  | "intents-filters"
  | "intents-create"
  | "create-intent-stepper"
  | "create-intent-input"
  | "create-intent-review"
  | "create-intent-delegate"
  | "create-intent-confirm"
  | "agents-list"
  | "agents-risk"
  | "agents-delegate"
  | "strategies-list"
  | "strategies-leaderboard"
  | "strategies-use"
  | "proofs-solvency"
  | "proofs-list"
  | "completed";

export interface StepMeta {
  id: OnboardingStep;
  page: string;
  title: string;
  description: string;
}

export const ONBOARDING_STEPS: StepMeta[] = [
  { id: "dashboard-balance", page: "/app/dashboard", title: "Your vault balance", description: "This is your total deposited balance. Everything stays in the StrategyVault and can only move when your intent conditions are met." },
  { id: "dashboard-intents", page: "/app/dashboard", title: "Active intents", description: "Intents are plain-English rules you set. Otter monitors the market and executes them when conditions are met." },
  { id: "dashboard-activity", page: "/app/dashboard", title: "Recent activity", description: "Track deposits, executed intents, MEV rebates, and delegation changes in real time." },
  { id: "dashboard-positions", page: "/app/dashboard", title: "Positions", description: "See exactly where your capital is allocated across protocols and chains." },
  { id: "intents-filters", page: "/app/intents", title: "Filter by status", description: "Quickly find intents by their current state: monitoring, proving, confirmed, or failed." },
  { id: "intents-list", page: "/app/intents", title: "Your intents", description: "This list shows every intent you've created, from monitoring to confirmed execution." },
  { id: "intents-create", page: "/app/intents", title: "Create a new intent", description: "Click here to describe a new conditional DeFi action in plain English." },
  { id: "create-intent-stepper", page: "/app/intents/new", title: "Guided intent creation", description: "The wizard walks you through describing, reviewing, delegating, and confirming your intent." },
  { id: "create-intent-input", page: "/app/intents/new", title: "Describe your intent", description: "Type what you want, like 'Lend 1000 USDC on Aave if yield > 3%'. Otter parses it automatically." },
  { id: "create-intent-review", page: "/app/intents/new", title: "Review parsing", description: "Check the detected action, amount, protocol, chain, and condition before continuing." },
  { id: "create-intent-delegate", page: "/app/intents/new", title: "Pick an Otter agent", description: "Choose an Otter-operated agent and a delegation. The agent can only act within the limits you sign." },
  { id: "create-intent-confirm", page: "/app/intents/new", title: "Confirm and monitor", description: "Otter generates a zero-knowledge proof before executing, guaranteeing the action respects your limits." },
  { id: "agents-list", page: "/app/agents", title: "Otter Agents", description: "These are protocol-operated, bonded, and audited execution agents. You never delegate to a user-run agent." },
  { id: "agents-risk", page: "/app/agents", title: "Risk profiles", description: "Each agent is tagged Conservative, Balanced, or Advanced so you can match it to your risk appetite." },
  { id: "agents-delegate", page: "/app/agents", title: "Delegate with limits", description: "Open any agent to create a signed delegation. You control max amounts, allowed protocols, chains, and expiry." },
  { id: "strategies-list", page: "/app/strategies", title: "Official strategies", description: "These are pre-built, audited Otter strategies. Use one to prefill an intent instantly." },
  { id: "strategies-leaderboard", page: "/app/strategies", title: "Agent leaderboard", description: "See which Otter agents have submitted the most proofs." },
  { id: "strategies-use", page: "/app/strategies", title: "Use a strategy", description: "Click 'Use strategy' to copy the rule into the intent wizard, then customize limits before confirming." },
  { id: "proofs-solvency", page: "/app/proofs", title: "Proof-of-solvency", description: "Otter periodically proves the vault holds enough assets without revealing individual balances." },
  { id: "proofs-list", page: "/app/proofs", title: "Every action proven", description: "Delegation, execution, and solvency proofs are all verified on-chain. Click any proof to inspect its details." },
];

const STEP_ORDER: OnboardingStep[] = [...ONBOARDING_STEPS.map((s) => s.id), "completed"];

export function useOnboarding() {
  const [step, setStep] = useState<OnboardingStep | null>(null);
  const [welcomeOpen, setWelcomeOpen] = useState(false);
  const [hasHydrated, setHasHydrated] = useState(false);
  const navigate = useNavigate();

  // Hydrate from localStorage once on mount.
  useEffect(() => {
    const rawStep = localStorage.getItem(STEP_KEY);
    const rawWelcome = localStorage.getItem(WELCOME_KEY);

    if (!rawWelcome) {
      setWelcomeOpen(true);
      setStep((rawStep as OnboardingStep) ?? "dashboard-balance");
    } else {
      setWelcomeOpen(false);
      setStep((rawStep as OnboardingStep) ?? "completed");
    }
    setHasHydrated(true);
  }, []);

  // Persist step changes.
  useEffect(() => {
    if (step) localStorage.setItem(STEP_KEY, step);
  }, [step]);

  const isOpen = useMemo(
    () => hasHydrated && step !== null && step !== "completed" && !welcomeOpen,
    [hasHydrated, step, welcomeOpen]
  );

  const goToStep = useCallback((next: OnboardingStep) => {
    setStep(next);
  }, []);

  const advance = useCallback(() => {
    setStep((current) => {
      if (!current) return "dashboard-balance";
      const index = STEP_ORDER.indexOf(current);
      const next = STEP_ORDER[Math.min(index + 1, STEP_ORDER.length - 1)];
      return next;
    });
  }, []);

  const back = useCallback(() => {
    setStep((current) => {
      if (!current) return current;
      const index = STEP_ORDER.indexOf(current);
      if (index <= 0) return current;
      return STEP_ORDER[index - 1];
    });
  }, []);

  const markWelcomeSeen = useCallback(() => {
    localStorage.setItem(WELCOME_KEY, "seen");
    setWelcomeOpen(false);
  }, []);

  const skip = useCallback(() => {
    markWelcomeSeen();
    setStep("completed");
  }, [markWelcomeSeen]);

  const restart = useCallback(() => {
    markWelcomeSeen();
    goToStep("dashboard-balance");
    navigate("/app/dashboard");
  }, [goToStep, navigate, markWelcomeSeen]);

  const dismiss = useCallback(() => {
    setStep("completed");
  }, []);

  const startTour = useCallback(() => {
    markWelcomeSeen();
    goToStep("dashboard-balance");
  }, [goToStep, markWelcomeSeen]);

  return {
    step,
    isOpen,
    welcomeOpen,
    hasHydrated,
    stepMeta: step ? ONBOARDING_STEPS.find((s) => s.id === step) ?? null : null,
    totalSteps: ONBOARDING_STEPS.length,
    advance,
    back,
    skip,
    restart,
    dismiss,
    startTour,
  };
}
