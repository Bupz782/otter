import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { Search, CheckCircle2, Loader2, Clock, ArrowRightCircle, XCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ExecutionStatus, IntentStatus } from "@/types/app";

const stepIcons: Record<IntentStatus, React.ReactNode> = {
  monitoring: <Search className="h-5 w-5" />,
  condition_met: <CheckCircle2 className="h-5 w-5" />,
  proving: <Loader2 className="h-5 w-5 animate-spin" />,
  submitted: <Clock className="h-5 w-5" />,
  confirmed: <CheckCircle2 className="h-5 w-5" />,
  failed: <XCircle className="h-5 w-5" />,
  revoked: <XCircle className="h-5 w-5" />,
};

export function KineticTimeline({ status, isLoading }: { status: ExecutionStatus | null; isLoading?: boolean }) {
  if (isLoading || !status) {
    return (
      <div className="space-y-6 py-4">
        {[1, 2, 3].map((i) => (
          <div key={i} className="flex items-start gap-4">
            <div className="h-8 w-8 rounded-full bg-secondary" />
            <div className="flex-1 space-y-2">
              <div className="h-4 w-32 rounded bg-secondary" />
              <div className="h-3 w-48 rounded bg-secondary" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  return (
    <div className="relative space-y-6 py-4 pl-4">
      <div className="absolute bottom-4 left-[27px] top-4 w-px bg-border" />
      {status.steps.map((step, index) => {
        const isActive = index === status.steps.length - 1;
        return (
          <motion.div
            key={step.status}
            initial={{ opacity: 0, x: -12 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: index * 0.1 }}
            className="relative flex items-start gap-4"
          >
            <div
              className={cn(
                "relative z-10 flex h-8 w-8 shrink-0 items-center justify-center rounded-full border transition-colors duration-300",
                isActive
                  ? "border-accent bg-accent text-accent-foreground"
                  : "border-border bg-secondary text-muted-foreground"
              )}
            >
              {stepIcons[step.status]}
            </div>
            <div className="flex-1">
              <p className={cn("font-medium", isActive && "text-foreground")}>{step.label}</p>
              <p className="text-sm text-muted-foreground">{step.detail}</p>
              {step.timestamp && (
                <p className="mt-1 text-xs text-muted-foreground">
                  {new Date(step.timestamp).toLocaleString()}
                </p>
              )}
            </div>
          </motion.div>
        );
      })}
    </div>
  );
}
