import { Badge } from "@/components/ui/badge";
import { StatusOrb } from "@/components/app/StatusOrb";
import { cn } from "@/lib/utils";
import type { IntentStatus } from "@/types/app";

const statusConfig: Record<
  IntentStatus,
  { label: string; variant: "default" | "secondary" | "destructive" | "outline" }
> = {
  monitoring: { label: "Monitoring", variant: "secondary" },
  condition_met: { label: "Condition Met", variant: "default" },
  proving: { label: "Proving", variant: "default" },
  submitted: { label: "Submitted", variant: "default" },
  confirmed: { label: "Confirmed", variant: "default" },
  failed: { label: "Failed", variant: "destructive" },
  revoked: { label: "Revoked", variant: "outline" },
};

export function IntentStatusBadge({
  status,
  className,
}: {
  status: IntentStatus;
  className?: string;
}) {
  const config = statusConfig[status];
  return (
    <Badge variant={config.variant} className={cn("gap-1.5 capitalize", className)}>
      <StatusOrb status={status} size="sm" />
      {config.label}
    </Badge>
  );
}
