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
  show: () => void;
  startTour: () => void;
}

const OnboardingContext = createContext<OnboardingContextValue | null>(null);

export function useOnboardingContext() {
  const ctx = useContext(OnboardingContext);
  if (!ctx) throw new Error("useOnboardingContext must be used within OnboardingProvider");
  return ctx;
}

export function OnboardingProvider({ children }: { children: ReactNode }) {
  const onboarding = useOnboarding();
  const { step, isOpen, welcomeOpen, hasHydrated, stepMeta, totalSteps, advance, back, skip, restart, dismiss, startTour } = onboarding;
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
      navigate({ pathname: location.pathname, search: params.toString() }, { replace: true });
    }
  }, [location.search, location.pathname, navigate, restart]);

  // Navigate automatically when the current step is on another page.
  useEffect(() => {
    if (!isOpen || !stepMeta || !hasHydrated) return;
    if (location.pathname !== stepMeta.page) {
      navigate(stepMeta.page);
    }
  }, [isOpen, stepMeta, location.pathname, navigate, hasHydrated]);

  // Resolve target id once we are on the right page.
  useEffect(() => {
    if (!isOpen || !stepMeta || !hasHydrated) {
      setTargetId(null);
      setTargetReady(false);
      return;
    }
    if (location.pathname === stepMeta.page) {
      setTargetId(`onboarding-${stepMeta.id}`);
    } else {
      setTargetId(null);
      setTargetReady(false);
    }
  }, [isOpen, stepMeta, location.pathname, hasHydrated]);

  // Wait for the target element to appear in the DOM.
  useEffect(() => {
    if (!targetId) {
      setTargetReady(false);
      return;
    }

    let raf: number;
    let timeout: ReturnType<typeof setTimeout>;
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
        // ~2 seconds; give up and show floating tooltip anyway.
        setTargetReady(true);
        return;
      }
      raf = requestAnimationFrame(check);
    };

    // Give React a tick to render before scanning the DOM.
    timeout = setTimeout(() => {
      check();
    }, 50);

    return () => {
      clearTimeout(timeout);
      cancelAnimationFrame(raf);
    };
  }, [targetId]);

  const currentStepIndex = step ? Math.max(ONBOARDING_STEPS.findIndex((s) => s.id === step), 0) + 1 : 0;

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
        show: onboarding.show,
        startTour,
      }}
    >
      {children}
      <AnimatePresence>
        {welcomeOpen && <WelcomeModal onStart={startTour} onSkip={skip} />}
      </AnimatePresence>
      {isOpen && stepMeta && targetReady && (
        <>
          {targetId && document.getElementById(targetId) && <Spotlight targetId={targetId} />}
          <OnboardingTooltip
            targetId={targetId ?? "onboarding-fallback"}
            title={stepMeta.title}
            description={stepMeta.description}
            onNext={advance}
            onBack={back}
            onSkip={skip}
            onDismiss={dismiss}
            isLast={step === "proofs-list"}
            isFirst={currentStepIndex === 1}
            stepNumber={currentStepIndex}
            totalSteps={totalSteps}
          />
        </>
      )}
    </OnboardingContext.Provider>
  );
}
