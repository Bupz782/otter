import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { motion, AnimatePresence } from "framer-motion";
import { X, ArrowRight, ArrowLeft, SkipForward } from "lucide-react";
import { Button } from "@/components/ui/button";

interface OnboardingTooltipProps {
  targetId: string;
  title: string;
  description: string;
  onNext: () => void;
  onBack: () => void;
  onSkip: () => void;
  onDismiss: () => void;
  isLast?: boolean;
  isFirst?: boolean;
  stepNumber: number;
  totalSteps: number;
}

export function OnboardingTooltip({
  targetId,
  title,
  description,
  onNext,
  onBack,
  onSkip,
  onDismiss,
  isLast,
  isFirst,
  stepNumber,
  totalSteps,
}: OnboardingTooltipProps) {
  const [rect, setRect] = useState<DOMRect | null>(null);
  const [position, setPosition] = useState<"bottom" | "top" | "left" | "right">("bottom");

  useEffect(() => {
    const update = () => {
      const target = document.getElementById(targetId);
      if (!target) return;
      const targetRect = target.getBoundingClientRect();
      setRect(targetRect);

      const tooltipWidth = 340;
      const tooltipHeight = 180;
      const spaceBottom = window.innerHeight - targetRect.bottom;
      const spaceTop = targetRect.top;
      const spaceRight = window.innerWidth - targetRect.right;

      if (spaceBottom > tooltipHeight) {
        setPosition("bottom");
      } else if (spaceTop > tooltipHeight) {
        setPosition("top");
      } else if (spaceRight > tooltipWidth) {
        setPosition("right");
      } else {
        setPosition("left");
      }
    };

    update();
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  }, [targetId]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onDismiss();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onDismiss]);

  if (!rect) return null;

  const style: React.CSSProperties = {
    position: "fixed",
    zIndex: 130,
    width: 340,
  };

  if (position === "bottom") {
    style.top = rect.bottom + 16;
    style.left = Math.min(Math.max(rect.left + rect.width / 2 - 170, 16), window.innerWidth - 356);
  } else if (position === "top") {
    style.top = rect.top - 196;
    style.left = Math.min(Math.max(rect.left + rect.width / 2 - 170, 16), window.innerWidth - 356);
  } else if (position === "right") {
    style.top = rect.top;
    style.left = rect.right + 16;
  } else {
    style.top = rect.top;
    style.left = rect.left - 356;
  }

  return createPortal(
    <>
      <div className="pointer-events-none fixed inset-0 z-[120] bg-black/5" />
      <AnimatePresence mode="wait">
        <motion.div
          key={targetId}
          initial={{ opacity: 0, scale: 0.95, y: 8 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: 8 }}
          transition={{ duration: 0.2 }}
          style={style}
          role="dialog"
          aria-modal="true"
          aria-live="polite"
          className="z-[130] overflow-hidden rounded-2xl border border-white/30 bg-white/[0.13] p-5 shadow-[0_8px_32px_0_rgba(0,0,0,0.36)] backdrop-blur-3xl saturate-150"
        >
          <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-white/[0.18] via-white/[0.08] to-transparent pointer-events-none" />
          <div className="absolute inset-0 rounded-2xl shadow-[inset_0_1px_0_0_rgba(255,255,255,0.25)] pointer-events-none" />
          <div className="relative flex items-start justify-between gap-3">
            <div>
              <p className="text-[10px] font-semibold uppercase tracking-wider text-accent drop-shadow-sm">
                Step {stepNumber} of {totalSteps}
              </p>
              <h3 className="font-heading text-base font-bold text-foreground drop-shadow-sm">{title}</h3>
              <p className="mt-1 text-sm text-white/90 drop-shadow-sm">{description}</p>
            </div>
            <button
              onClick={onDismiss}
              className="rounded-full p-1 text-white/80 hover:bg-white/15 hover:text-foreground"
              aria-label="Dismiss"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
          <div className="relative mt-5 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Button variant="ghost" size="sm" onClick={onBack} disabled={isFirst} className="text-white/90 hover:bg-white/15 hover:text-foreground">
                <ArrowLeft className="mr-1 h-3 w-3" />
                Back
              </Button>
              <Button variant="ghost" size="sm" onClick={onSkip} className="text-white/80 hover:bg-white/15 hover:text-foreground">
                <SkipForward className="mr-2 h-3 w-3" />
                Skip tour
              </Button>
            </div>
            <Button size="sm" onClick={onNext} className="rounded-full bg-white/90 text-accent-foreground shadow hover:bg-white hover:shadow-lg">
              {isLast ? "Finish" : "Next"}
              <ArrowRight className="ml-2 h-3 w-3" />
            </Button>
          </div>
        </motion.div>
      </AnimatePresence>
    </>,
    document.body
  );
}
