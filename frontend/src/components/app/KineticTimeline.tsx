import { motion } from "framer-motion";
import { cn } from "@/lib/utils";
import { STATUS_PRESENTATION } from "@/lib/status";
import type { ExecutionStatus } from "@/types/app";

export function KineticTimeline({
  status,
  isLoading,
}: {
  status: ExecutionStatus | null;
  isLoading?: boolean;
}) {
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

  // The API tells us which step is current; fall back to the last step.
  const currentIndex = status.steps.findIndex((step) => step.status === status.currentStep);
  const activeIndex = currentIndex === -1 ? status.steps.length - 1 : currentIndex;

  return (
    <div role="list" className="relative space-y-6 py-4 pl-4">
      <div aria-hidden="true" className="absolute bottom-4 left-[27px] top-4 w-px bg-border" />
      {status.steps.map((step, index) => {
        const isActive = index === activeIndex;
        const Icon = STATUS_PRESENTATION[step.status].icon;
        return (
          <motion.div
            role="listitem"
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
              <Icon className="h-5 w-5" />
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
