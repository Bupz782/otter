import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import { Bot, Lock, ArrowRight, BookOpen, Trophy } from "lucide-react";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { motion } from "framer-motion";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { DataRow } from "@/components/app/DataRow";
import { EmptyState } from "@/components/app/EmptyState";
import { ErrorState } from "@/components/app/ErrorState";
import { useAgents } from "@/hooks/useAgents";
import { useStrategies } from "@/hooks/useStrategies";
import { useLeaderboard } from "@/hooks/useLeaderboard";
import type { Agent } from "@/types/app";

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

const riskVariant: Record<string, "default" | "secondary" | "outline" | "destructive"> = {
  Conservative: "default",
  Balanced: "secondary",
  Advanced: "outline",
};

function agentStats(agent: Agent): { label: string; value: string }[] {
  return [
    { label: "Yield routed", value: `$${(agent.yieldGenerated / 1_000_000).toFixed(1)}M` },
    { label: "Proofs", value: agent.proofsSubmitted.toLocaleString() },
    { label: "Uptime", value: `${agent.uptime}%` },
    { label: "Rebates", value: `$${agent.mevCaptured.toLocaleString()}` },
  ];
}

export function AgentsPage() {
  useDocumentTitle("Agents");
  const { data: agents, isLoading, error, refetch } = useAgents();
  const {
    data: strategies,
    isLoading: strategiesLoading,
    error: strategiesError,
    refetch: refetchStrategies,
  } = useStrategies();
  const {
    data: leaderboard,
    isLoading: leaderboardLoading,
    error: leaderboardError,
    refetch: refetchLeaderboard,
  } = useLeaderboard();

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <FadeIn>
        <PageHeader
          title="Otter Agents"
          subtitle="Vetted agents that execute inside your signed limits."
        />
      </FadeIn>

      <FadeIn delay={0.05}>
        <SectionCard className="py-4">
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-border bg-secondary">
              <Lock className="h-4 w-4 text-accent" />
            </div>
            <p className="text-sm text-muted-foreground">
              <span className="font-medium text-foreground">Vetted agents only.</span> Every agent
              is protocol-operated, bonded, and audited. Signed limits, not API keys.
            </p>
          </div>
        </SectionCard>
      </FadeIn>

      <FadeIn delay={0.1}>
        <div className="grid gap-6 sm:grid-cols-2">
          {isLoading ? (
            <>
              <Skeleton className="h-56 w-full" />
              <Skeleton className="h-56 w-full" />
            </>
          ) : error ? (
            <div className="sm:col-span-2">
              <ErrorState subject="agents" onRetry={refetch} />
            </div>
          ) : agents.length === 0 ? (
            <div className="sm:col-span-2">
              <EmptyState
                icon={<Bot className="h-6 w-6" />}
                title="No agents available"
                description="Every Otter agent is bonded and audited. Check back soon."
              />
            </div>
          ) : (
            agents.map((agent) => (
              <Link key={agent.id} to={`/app/agents/${agent.id}`} className="group block h-full">
                <SectionCard className="flex h-full flex-col transition-colors hover:border-accent/40">
                  <div className="flex items-start justify-between gap-3">
                    <p className="font-heading text-lg font-bold transition-colors group-hover:text-accent">
                      {agent.name}
                    </p>
                    <Badge
                      variant={riskVariant[agent.riskProfile]}
                      className="shrink-0 text-[10px]"
                    >
                      {agent.riskProfile}
                    </Badge>
                  </div>
                  <p className="mt-1 truncate text-sm text-muted-foreground">{agent.description}</p>
                  <div className="mt-4 grid flex-1 grid-cols-2 content-start gap-3 sm:grid-cols-4">
                    {agentStats(agent).map((stat) => (
                      <div key={stat.label}>
                        <p className="text-xs uppercase tracking-wider text-muted-foreground">
                          {stat.label}
                        </p>
                        <p className="mt-0.5 text-sm font-medium tabular-nums">{stat.value}</p>
                      </div>
                    ))}
                  </div>
                  <span className="mt-4 inline-flex items-center gap-1 text-sm font-medium text-accent">
                    View agent
                    <ArrowRight className="h-3.5 w-3.5 transition-transform group-hover:translate-x-0.5" />
                  </span>
                </SectionCard>
              </Link>
            ))
          )}
        </div>
      </FadeIn>

      <FadeIn delay={0.15}>
        <div className="grid gap-6 lg:grid-cols-3">
          <SectionCard
            title="Official strategies"
            subtitle="Curated, audited strategies from Otter agents."
            className="lg:col-span-2"
          >
            {strategiesLoading ? (
              <div className="space-y-3">
                <Skeleton className="h-16 w-full" />
                <Skeleton className="h-16 w-full" />
              </div>
            ) : strategiesError ? (
              <ErrorState subject="strategies" onRetry={refetchStrategies} />
            ) : strategies.length === 0 ? (
              <EmptyState
                icon={<BookOpen className="h-6 w-6" />}
                title="No strategies yet"
                description="Otter will publish new strategies as agents are added."
              />
            ) : (
              <div className="space-y-3">
                {strategies.map((strategy) => (
                  <DataRow key={strategy.id}>
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <p className="truncate text-sm font-medium">{strategy.title}</p>
                        <Badge variant="secondary" className="shrink-0">
                          {strategy.agentName}
                        </Badge>
                      </div>
                      <p className="truncate text-xs text-muted-foreground">
                        {strategy.description}
                      </p>
                      <p className="mt-1 text-xs text-muted-foreground tabular-nums">
                        {strategy.copies.toLocaleString()} users · $
                        {strategy.totalVolume.toLocaleString()} volume
                        {strategy.apy > 0 && (
                          <span className="text-emerald-400"> · +{strategy.apy}% APY</span>
                        )}
                      </p>
                    </div>
                    <Button asChild variant="ghost" size="sm" className="shrink-0">
                      <Link to={`/app/intents/new?strategy=${strategy.id}`}>Use strategy</Link>
                    </Button>
                  </DataRow>
                ))}
              </div>
            )}
          </SectionCard>

          <SectionCard title="Agent Leaderboard" subtitle="Ranked by proof count.">
            {leaderboardLoading ? (
              <div className="space-y-3">
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
              </div>
            ) : leaderboardError ? (
              <ErrorState subject="the leaderboard" onRetry={refetchLeaderboard} />
            ) : leaderboard.length === 0 ? (
              <EmptyState
                icon={<Trophy className="h-6 w-6" />}
                title="No rankings yet"
                description="Agents climb the board as they submit proofs."
              />
            ) : (
              <div className="space-y-3">
                {leaderboard.slice(0, 5).map((entry) => (
                  <Link key={entry.agentId} to={`/app/agents/${entry.agentId}`} className="block">
                    <DataRow>
                      <span className="w-5 shrink-0 font-heading text-lg font-bold text-accent tabular-nums">
                        {entry.rank}
                      </span>
                      <span className="min-w-0 flex-1 truncate text-sm font-medium">
                        {entry.agentName}
                      </span>
                      <span className="shrink-0 text-xs text-muted-foreground tabular-nums">
                        {entry.proofsSubmitted.toLocaleString()} proofs
                      </span>
                    </DataRow>
                  </Link>
                ))}
              </div>
            )}
          </SectionCard>
        </div>
      </FadeIn>
    </div>
  );
}
