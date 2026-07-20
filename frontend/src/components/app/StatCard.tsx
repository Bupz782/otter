import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";

interface StatCardProps {
  icon: LucideIcon;
  label: string;
  value: ReactNode;
  hint?: string;
  className?: string;
}

export function StatCard({ icon: Icon, label, value, hint, className }: StatCardProps) {
  return (
    <div
      className={cn(
        "rounded-2xl border border-border/50 bg-card/60 p-5 backdrop-blur-sm",
        className
      )}
    >
      <div className="mb-3 flex h-9 w-9 items-center justify-center rounded-lg border border-border bg-secondary">
        <Icon className="h-4 w-4 text-accent" />
      </div>
      <p className="text-xs uppercase tracking-wider text-muted-foreground">{label}</p>
      <p className="mt-1 font-heading text-2xl font-bold tabular-nums">{value}</p>
      {hint && <p className="mt-1 text-xs text-muted-foreground">{hint}</p>}
    </div>
  );
}
