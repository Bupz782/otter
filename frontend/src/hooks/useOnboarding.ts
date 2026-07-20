import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";

const STEP_KEY = "otter-onboarding-step";
const WELCOME_KEY = "otter-onboarding-welcome";

export type OnboardingStep =
  | "dashboard-balance"
  | "dashboard-create"
  | "dashboard-intents"
  | "dashboard-activity"
  | "dashboard-positions"
  | "completed";

export interface StepMeta {
  id: OnboardingStep;
  page: string;
  title: string;
  description: string;
}

export const ONBOARDING_STEPS: StepMeta[] = [
  {
    id: "dashboard-balance",
    page: "/app/dashboard",
    title: "Your vault at a glance",
    description: "Everything Otter manages for you, in one place. No custody, ever.",
  },
  {
    id: "dashboard-create",
    page: "/app/dashboard",
    title: "Create an intent",
    description:
      "Tell Otter what to do and when, in plain English. It watches the market and acts.",
  },
  {
    id: "dashboard-intents",
    page: "/app/dashboard",
    title: "Rules in play",
    description:
      "Every live intent and its status. Otter monitors each condition around the clock.",
  },
  {
    id: "dashboard-activity",
    page: "/app/dashboard",
    title: "Recent activity",
    description: "Deposits, executions, and MEV rebates as they happen. Nothing moves silently.",
  },
  {
    id: "dashboard-positions",
    page: "/app/dashboard",
    title: "Where capital sits",
    description:
      "Your allocation across protocols and chains, down to the dollar. Proven on-chain.",
  },
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
    // Guard against stale step ids persisted by older versions of the tour.
    const stored =
      rawStep === "completed" || ONBOARDING_STEPS.some((s) => s.id === rawStep)
        ? (rawStep as OnboardingStep)
        : null;

    if (!rawWelcome) {
      setWelcomeOpen(true);
      setStep(stored ?? "dashboard-balance");
    } else {
      setWelcomeOpen(false);
      setStep(stored ?? "completed");
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
    navigate("/app/dashboard");
  }, [goToStep, navigate, markWelcomeSeen]);

  return {
    step,
    isOpen,
    welcomeOpen,
    hasHydrated,
    stepMeta: step ? (ONBOARDING_STEPS.find((s) => s.id === step) ?? null) : null,
    totalSteps: ONBOARDING_STEPS.length,
    advance,
    back,
    skip,
    restart,
    dismiss,
    startTour,
  };
}
