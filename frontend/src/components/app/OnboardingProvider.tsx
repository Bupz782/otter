import { createContext, useContext, useState, useEffect, type ReactNode } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { AnimatePresence } from "framer-motion";
import { OnboardingTooltip } from "./OnboardingTooltip";
import { Spotlight } from "./Spotlight";
import { WelcomeModal } from "./WelcomeModal";
import { useOnboarding, ONBOARDING_STEPS } from "@/hooks/useOnboarding";

interface OnboardingContextValue {
  step: ReturnType<typeof useOnboarding>["step"];
  isOpen: boolean;
  welcomeOpen: boolean;
  hasHydrated: boolean;
  stepMeta: ReturnType<typeof useOnboarding>["stepMeta"];
  totalSteps: number;
  advance: () => void;
  back: () => void;
  skip: () => void;
  restart: () => void;
  dismiss: () => void;
  startTour: () => void;
}

const OnboardingContext = createContext<OnboardingContextValue | null>(null);

// eslint-disable-next-line react-refresh/only-export-components
export function useOnboardingContext() {
  const ctx = useContext(OnboardingContext);
  if (!ctx) throw new Error("useOnboardingContext must be used within OnboardingProvider");
  return ctx;
}

export function OnboardingProvider({ children }: { children: ReactNode }) {
  const onboarding = useOnboarding();
  const {
    step,
    isOpen,
    welcomeOpen,
    hasHydrated,
    stepMeta,
    totalSteps,
    advance,
    back,
    skip,
    restart,
    dismiss,
    startTour,
  } = onboarding;
  const location = useLocation();
  const navigate = useNavigate();
  const [targetId, setTargetId] = useState<string | null>(null);
  const [targetReady, setTargetReady] = useState(false);

  // Allow forcing the tour via URL: ?onboarding=restart or ?onboarding=start
  useEffect(() => {
    const params = new URLSearchParams(location.search);
    const mode = params.get("onboarding");
    if (mode === "restart" || mode === "start") {
      localStorage.removeItem("otter-onboarding-welcome");
      localStorage.removeItem("otter-onboarding-step");
      restart();
      params.delete("onboarding");
      // Land on the dashboard (where every tour step lives) with the param stripped.
      navigate({ pathname: "/app/dashboard", search: params.toString() }, { replace: true });
    }
  }, [location.search, location.pathname, navigate, restart]);

  // Resolve the target id for the current step. The tour never navigates
  // by itself: all steps live on the dashboard, and entry points
  // (restart, startTour, ?onboarding=) already land there.
  useEffect(() => {
    if (!isOpen || !stepMeta || !hasHydrated) {
      setTargetId(null);
      setTargetReady(false);
      return;
    }
    setTargetId(`onboarding-${stepMeta.id}`);
    setTargetReady(false);
  }, [isOpen, stepMeta, hasHydrated]);

  // Wait for the target element to appear in the DOM. If it never does,
  // auto-advance to the next step instead of freezing; once the last step
  // is skipped this way, advance() completes the tour gracefully.
  useEffect(() => {
    if (!targetId) {
      setTargetReady(false);
      return;
    }

    const timeout = setTimeout(() => {
      check();
    }, 50);
    const raf = { current: 0 };
    let attempts = 0;

    const check = () => {
      const el = document.getElementById(targetId);
      attempts += 1;
      if (el) {
        setTargetReady(true);
        el.scrollIntoView({ behavior: "smooth", block: "center" });
        return;
      }
      if (attempts > 120) {
        // ~2 seconds: target is missing, skip to the next step.
        advance();
        return;
      }
      raf.current = requestAnimationFrame(check);
    };

    return () => {
      clearTimeout(timeout);
      cancelAnimationFrame(raf.current);
    };
  }, [targetId, advance]);

  const currentStepIndex = step
    ? Math.max(
        ONBOARDING_STEPS.findIndex((s) => s.id === step),
        0
      ) + 1
    : 0;

  return (
    <OnboardingContext.Provider
      value={{
        step,
        isOpen,
        welcomeOpen,
        hasHydrated,
        stepMeta,
        totalSteps,
        advance,
        back,
        skip,
        restart,
        dismiss,
        startTour,
      }}
    >
      {children}
      <AnimatePresence>
        {welcomeOpen && <WelcomeModal onStart={startTour} onSkip={skip} />}
      </AnimatePresence>
      {isOpen && stepMeta && targetReady && targetId && (
        <>
          <Spotlight targetId={targetId} />
          <OnboardingTooltip
            targetId={targetId}
            title={stepMeta.title}
            description={stepMeta.description}
            onNext={advance}
            onBack={back}
            onSkip={skip}
            onDismiss={dismiss}
            isLast={step === ONBOARDING_STEPS[ONBOARDING_STEPS.length - 1].id}
            isFirst={currentStepIndex === 1}
            stepNumber={currentStepIndex}
            totalSteps={totalSteps}
          />
        </>
      )}
    </OnboardingContext.Provider>
  );
}
