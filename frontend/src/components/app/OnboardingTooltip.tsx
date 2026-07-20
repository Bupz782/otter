import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { motion, AnimatePresence } from "framer-motion";
import { ArrowRight, ArrowLeft, SkipForward } from "lucide-react";
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
    <AnimatePresence mode="wait">
      <motion.div
        key={targetId}
        initial={{ opacity: 0, scale: 0.95, y: 8 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.95, y: 8 }}
        transition={{ duration: 0.2 }}
        style={style}
        role="dialog"
        aria-label={title}
        aria-live="polite"
        className="z-[130] rounded-xl border border-border/60 bg-card p-5 backdrop-blur-md"
      >
        <div>
          <p className="text-[10px] font-semibold uppercase tracking-wider text-accent">
            Step {stepNumber} of {totalSteps}
          </p>
          <h3 className="font-heading text-base font-bold text-foreground">{title}</h3>
          <p className="mt-1 text-sm text-muted-foreground">{description}</p>
        </div>
        <div className="mt-5 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="sm" onClick={onBack} disabled={isFirst}>
              <ArrowLeft className="mr-1 h-3 w-3" />
              Back
            </Button>
            <Button variant="ghost" size="sm" onClick={onSkip}>
              <SkipForward className="mr-2 h-3 w-3" />
              Skip tour
            </Button>
          </div>
          <Button size="sm" onClick={onNext} autoFocus className="rounded-full">
            {isLast ? "Finish" : "Next"}
            <ArrowRight className="ml-2 h-3 w-3" />
          </Button>
        </div>
      </motion.div>
    </AnimatePresence>,
    document.body
  );
}
