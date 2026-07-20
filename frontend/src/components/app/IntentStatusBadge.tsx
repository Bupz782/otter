import { Badge } from "@/components/ui/badge";
import { StatusOrb } from "@/components/app/StatusOrb";
import { getStatusPresentation } from "@/lib/status";
import { cn } from "@/lib/utils";
import type { IntentStatus } from "@/types/app";

export function IntentStatusBadge({
  status,
  className,
}: {
  status: IntentStatus;
  className?: string;
}) {
  const config = getStatusPresentation(status);
  return (
    <Badge variant="outline" className={cn("gap-1.5", config.badgeClass, className)}>
      <StatusOrb status={status} size="sm" />
      {config.label}
    </Badge>
  );
}
