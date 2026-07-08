import { useState } from "react";
import { Link } from "react-router-dom";
import { Plus, Filter, Lightbulb } from "lucide-react";
import { motion } from "framer-motion";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { useIntents } from "@/hooks/useIntents";
import { IntentStatusBadge } from "@/components/app/IntentStatusBadge";
import { EmptyState } from "@/components/app/EmptyState";
import { StatusOrb } from "@/components/app/StatusOrb";
import type { IntentStatus } from "@/types/app";

const filters: { label: string; value: IntentStatus | "all" }[] = [
  { label: "All", value: "all" },
  { label: "Monitoring", value: "monitoring" },
  { label: "Proving", value: "proving" },
  { label: "Confirmed", value: "confirmed" },
  { label: "Failed", value: "failed" },
];

export function IntentsPage() {
  const [statusFilter, setStatusFilter] = useState<IntentStatus | undefined>(undefined);
  const { data: intents, isLoading } = useIntents(
    statusFilter ? { status: statusFilter } : undefined
  );

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
      >
        <div>
          <h1 className="font-heading text-3xl font-bold tracking-tight">Intents</h1>
          <p className="text-muted-foreground">Manage your conditional DeFi intents.</p>
        </div>
        <Button asChild id="onboarding-intents-create" className="rounded-full">
          <Link to="/app/intents/new">
            <Plus className="mr-2 h-4 w-4" />
            Create intent
          </Link>
        </Button>
      </motion.div>

      <div id="onboarding-intents-filters" className="flex flex-wrap items-center gap-2">
        <Filter className="h-4 w-4 text-muted-foreground" />
        {filters.map((f) => (
          <Button
            key={f.value}
            variant={
              statusFilter === (f.value === "all" ? undefined : f.value) ? "default" : "outline"
            }
            size="sm"
            onClick={() => setStatusFilter(f.value === "all" ? undefined : f.value)}
          >
            {f.value !== "all" && <StatusOrb status={f.value} size="sm" />}
            {f.label}
          </Button>
        ))}
      </div>

      <Card id="onboarding-intents-list">
        <CardHeader>
          <CardTitle>Your intents</CardTitle>
          <CardDescription>Click an intent to view execution details.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {isLoading ? (
            <div className="space-y-3">
              <Skeleton className="h-24 w-full" />
              <Skeleton className="h-24 w-full" />
              <Skeleton className="h-24 w-full" />
            </div>
          ) : intents.length === 0 ? (
            <EmptyState
              icon={<Lightbulb className="h-6 w-6" />}
              title="No intents match this filter"
              description="Try a different filter or create a new intent."
              action={
                <Button asChild variant="outline" className="rounded-full">
                  <Link to="/app/intents/new">Create intent</Link>
                </Button>
              }
            />
          ) : (
            intents.map((intent) => (
              <Link
                key={intent.id}
                to={`/app/intents/${intent.id}`}
                className="group flex flex-col gap-3 rounded-xl border border-border/60 bg-card p-4 transition-colors hover:border-accent/40 sm:flex-row sm:items-center sm:justify-between"
              >
                <div className="space-y-1">
                  <p className="font-medium transition-colors group-hover:text-accent">
                    {intent.rawText}
                  </p>
                  <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                    <Badge variant="outline">{intent.parsed.type}</Badge>
                    <span>
                      {intent.parsed.amount} {intent.parsed.asset}
                    </span>
                    <span>·</span>
                    <span>{intent.parsed.protocol}</span>
                    <span>·</span>
                    <span>{intent.parsed.chain}</span>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <IntentStatusBadge status={intent.status} />
                  <span className="text-xs text-muted-foreground">
                    {new Date(intent.createdAt).toLocaleDateString()}
                  </span>
                </div>
              </Link>
            ))
          )}
        </CardContent>
      </Card>
    </div>
  );
}
