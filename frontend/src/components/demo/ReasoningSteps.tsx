import { useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Check } from "lucide-react";
import { reasoningSteps } from "@/data/intents";

interface ReasoningStepsProps {
  isActive: boolean;
  onComplete: () => void;
}

export function ReasoningSteps({ isActive, onComplete }: ReasoningStepsProps) {
  const [visibleCount, setVisibleCount] = useState(0);

  useEffect(() => {
    if (!isActive) {
      setVisibleCount(0);
      return;
    }

    let current = 0;
    const interval = setInterval(() => {
      current += 1;
      setVisibleCount(current);
      if (current >= reasoningSteps.length) {
        clearInterval(interval);
        setTimeout(onComplete, 400);
      }
    }, 650);

    return () => clearInterval(interval);
  }, [isActive, onComplete]);

  if (!isActive && visibleCount === 0) return null;

  return (
    <div className="mx-auto max-w-xl space-y-3 py-8">
      <AnimatePresence>
        {reasoningSteps.slice(0, visibleCount).map((step) => (
          <motion.div
            key={step}
            initial={{ opacity: 0, x: -12 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.3, ease: [0.22, 1, 0.36, 1] }}
            className="flex items-center gap-3 text-sm text-muted-foreground"
          >
            <span className="flex h-5 w-5 items-center justify-center rounded-full border border-accent/40 bg-accent-subtle">
              <Check className="h-3 w-3 text-accent" />
            </span>
            <span>{step}</span>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}
