import { motion } from "framer-motion";
import { cn } from "@/lib/utils";
import type { IntentStatus } from "@/types/app";

const statusConfig: Record<
  IntentStatus,
  { color: string; pulse: boolean; spin: boolean }
> = {
  monitoring: { color: "bg-amber-400", pulse: true, spin: false },
  condition_met: { color: "bg-amber-300", pulse: true, spin: false },
  proving: { color: "bg-accent", pulse: true, spin: true },
  submitted: { color: "bg-blue-400", pulse: true, spin: false },
  confirmed: { color: "bg-emerald-400", pulse: false, spin: false },
  failed: { color: "bg-rose-400", pulse: false, spin: false },
  revoked: { color: "bg-muted-foreground", pulse: false, spin: false },
};

export function StatusOrb({
  status,
  size = "md",
  className,
}: {
  status: IntentStatus;
  size?: "sm" | "md" | "lg";
  className?: string;
}) {
  const config = statusConfig[status];
  const sizeClass = {
    sm: "h-2 w-2",
    md: "h-3 w-3",
    lg: "h-4 w-4",
  }[size];

  return (
    <span className={cn("relative inline-flex", className)}>
      <span
        className={cn(
          "inline-flex rounded-full",
          sizeClass,
          config.color,
          config.spin && "animate-spin",
          config.pulse && "opacity-90"
        )}
      />
      {config.pulse && (
        <span
          className={cn(
            "absolute inline-flex rounded-full opacity-60",
            sizeClass,
            config.color,
            "animate-ping"
          )}
        />
      )}
    </span>
  );
}
