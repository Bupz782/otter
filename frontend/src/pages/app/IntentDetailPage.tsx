import { useParams, Link } from "react-router-dom";
import { ArrowLeft, ShieldCheck, Clock } from "lucide-react";
import { motion } from "framer-motion";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { useIntent } from "@/hooks/useIntent";
import { useExecutionStatus } from "@/hooks/useExecutionStatus";
import { IntentStatusBadge } from "@/components/app/IntentStatusBadge";
import { KineticTimeline } from "@/components/app/KineticTimeline";

export function IntentDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data: intent, isLoading: intentLoading } = useIntent(id);
  const { data: status, isLoading: statusLoading } = useExecutionStatus(id, true);

  if (intentLoading) {
    return (
      <div className="mx-auto max-w-3xl space-y-6">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-40 w-full" />
        <Skeleton className="h-64 w-full" />
      </div>
    );
  }

  if (!intent) {
    return (
      <div className="mx-auto max-w-3xl py-20 text-center">
        <h1 className="font-heading text-2xl font-bold">Intent not found</h1>
        <Button asChild className="mt-6 rounded-full">
          <Link to="/app/intents">Back to intents</Link>
        </Button>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <div className="flex items-center gap-4">
        <Button asChild variant="ghost" size="sm">
          <Link to="/app/intents">
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to intents
          </Link>
        </Button>
      </div>

      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        <h1 className="font-heading text-3xl font-bold tracking-tight">Intent Details</h1>
        <p className="text-muted-foreground">{intent.rawText}</p>
      </motion.div>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>Status</CardTitle>
            <CardDescription>Current execution state.</CardDescription>
          </div>
          <IntentStatusBadge status={intent.status} />
        </CardHeader>
        <CardContent>
          <KineticTimeline status={status} isLoading={statusLoading} />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Parameters</CardTitle>
          <CardDescription>Parsed intent parameters.</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-3 sm:grid-cols-2">
          <div className="rounded-lg border border-border/60 bg-secondary p-3">
            <p className="text-xs text-muted-foreground">Action</p>
            <p className="font-heading text-lg font-bold capitalize">{intent.parsed.type}</p>
          </div>
          <div className="rounded-lg border border-border/60 bg-secondary p-3">
            <p className="text-xs text-muted-foreground">Amount</p>
            <p className="font-heading text-lg font-bold">{intent.parsed.amount} {intent.parsed.asset}</p>
          </div>
          <div className="rounded-lg border border-border/60 bg-secondary p-3">
            <p className="text-xs text-muted-foreground">Protocol</p>
            <p className="font-heading text-lg font-bold">{intent.parsed.protocol}</p>
          </div>
          <div className="rounded-lg border border-border/60 bg-secondary p-3">
            <p className="text-xs text-muted-foreground">Chain</p>
            <p className="font-heading text-lg font-bold">{intent.parsed.chain}</p>
          </div>
          {intent.parsed.condition && (
            <div className="rounded-lg border border-border/60 bg-secondary p-3 sm:col-span-2">
              <p className="text-xs text-muted-foreground">Condition</p>
              <p className="font-heading text-lg font-bold">{intent.parsed.condition}</p>
            </div>
          )}
        </CardContent>
      </Card>

      {intent.txHash && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <ShieldCheck className="h-5 w-5 text-accent" />
              Transaction
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
              <code className="rounded-lg bg-secondary px-3 py-2 text-xs">{intent.txHash}</code>
              <Badge variant="secondary" className="w-fit">
                {intent.mevRebate ? `+${intent.mevRebate} USDC MEV rebate` : "No rebate"}
              </Badge>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
