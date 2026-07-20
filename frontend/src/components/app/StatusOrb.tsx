import { cn } from "@/lib/utils";
import { getStatusPresentation } from "@/lib/status";
import type { IntentStatus } from "@/types/app";

export function StatusOrb({
  status,
  size = "md",
  className,
}: {
  status: IntentStatus;
  size?: "sm" | "md" | "lg";
  className?: string;
}) {
  const config = getStatusPresentation(status);
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
          config.dotClass,
          config.active && "opacity-90"
        )}
      />
      {config.active && (
        <span
          className={cn(
            "absolute inline-flex animate-ping rounded-full opacity-60",
            sizeClass,
            config.dotClass
          )}
        />
      )}
    </span>
  );
}
