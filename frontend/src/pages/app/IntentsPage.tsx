import { useState, type ReactNode } from "react";
import { Link } from "react-router-dom";
import { Plus, Lightbulb, ChevronRight } from "lucide-react";
import { motion } from "framer-motion";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { DataRow } from "@/components/app/DataRow";
import { IntentStatusBadge } from "@/components/app/IntentStatusBadge";
import { EmptyState } from "@/components/app/EmptyState";
import { ErrorState } from "@/components/app/ErrorState";
import { StatusOrb } from "@/components/app/StatusOrb";
import { useIntents } from "@/hooks/useIntents";
import { cn } from "@/lib/utils";
import type { IntentStatus } from "@/types/app";

const EASE: [number, number, number, number] = [0.22, 1, 0.36, 1];

/** Mount-only fade/slide used to stagger the page blocks. */
function FadeIn({
  children,
  delay = 0,
  className,
}: {
  children: ReactNode;
  delay?: number;
  className?: string;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4, ease: EASE, delay }}
      className={className}
    >
      {children}
    </motion.div>
  );
}

const filters: { label: string; value: IntentStatus | "all" }[] = [
  { label: "All", value: "all" },
  { label: "Monitoring", value: "monitoring" },
  { label: "Condition met", value: "condition_met" },
  { label: "Proving", value: "proving" },
  { label: "Confirmed", value: "confirmed" },
  { label: "Failed", value: "failed" },
];

export function IntentsPage() {
  const [statusFilter, setStatusFilter] = useState<IntentStatus | undefined>(undefined);
  const {
    data: intents,
    isLoading,
    error,
    refetch,
  } = useIntents(statusFilter ? { status: statusFilter } : undefined);

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <FadeIn>
        <PageHeader
          title="Intents"
          subtitle="The rules Otter watches for you."
          action={
            <Button asChild className="rounded-full">
              <Link to="/app/intents/new">
                <Plus className="mr-2 h-4 w-4" />
                Create intent
              </Link>
            </Button>
          }
        />
      </FadeIn>

      <FadeIn delay={0.05}>
        <div className="flex flex-wrap items-center gap-2">
          {filters.map((f) => {
            const isActive = statusFilter === (f.value === "all" ? undefined : f.value);
            return (
              <button
                key={f.value}
                type="button"
                aria-pressed={isActive}
                onClick={() => setStatusFilter(f.value === "all" ? undefined : f.value)}
                className={cn(
                  "inline-flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs transition-colors",
                  isActive
                    ? "border-accent/40 bg-accent-subtle text-accent"
                    : "border-border/60 text-muted-foreground hover:text-foreground"
                )}
              >
                {f.value !== "all" && <StatusOrb status={f.value} size="sm" />}
                {f.label}
              </button>
            );
          })}
        </div>
      </FadeIn>

      <FadeIn delay={0.1}>
        <SectionCard title="Your intents" subtitle="Open one to follow its execution.">
          {isLoading ? (
            <div className="space-y-3">
              <Skeleton className="h-16 w-full" />
              <Skeleton className="h-16 w-full" />
              <Skeleton className="h-16 w-full" />
            </div>
          ) : error ? (
            <ErrorState subject="intents" onRetry={refetch} />
          ) : intents.length === 0 ? (
            <EmptyState
              icon={<Lightbulb className="h-6 w-6" />}
              title="No intents to show"
              description="Try another filter, or set a new intent and Otter starts watching."
              action={
                <Button asChild variant="outline" className="rounded-full">
                  <Link to="/app/intents/new">Create intent</Link>
                </Button>
              }
            />
          ) : (
            <div className="space-y-3">
              {intents.map((intent) => (
                <Link key={intent.id} to={`/app/intents/${intent.id}`} className="block">
                  <DataRow>
                    <IntentStatusBadge status={intent.status} className="shrink-0" />
                    <p className="min-w-0 flex-1 truncate text-sm font-medium">{intent.rawText}</p>
                    <div className="hidden shrink-0 items-center gap-3 sm:flex">
                      <span className="font-mono text-xs text-muted-foreground">
                        {intent.parsed.condition ?? "Not specified"}
                      </span>
                      <Badge variant="outline">{intent.parsed.chain ?? "Any chain"}</Badge>
                    </div>
                    <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
                  </DataRow>
                </Link>
              ))}
            </div>
          )}
        </SectionCard>
      </FadeIn>
    </div>
  );
}
