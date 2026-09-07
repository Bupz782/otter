import { useEffect, useState, type ReactNode } from "react";
import { Link } from "react-router-dom";
import { Bot, Lock, ArrowRight, BookOpen, ShieldCheck, KeyRound } from "lucide-react";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { StatCard } from "@/components/app/StatCard";
import { DataRow } from "@/components/app/DataRow";
import { EmptyState } from "@/components/app/EmptyState";
import { ErrorState } from "@/components/app/ErrorState";
import { DemoDataNotice } from "@/components/app/DemoDataNotice";
import { useAgents } from "@/hooks/useAgents";
import { useStrategies } from "@/hooks/useStrategies";
import { useAuthToken } from "@/hooks/useAuthToken";
import { api } from "@/lib/api";

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

function truncateHex(value: string): string {
  return value.length > 18 ? `${value.slice(0, 10)}…${value.slice(-8)}` : value;
}

/** Fetches the agent's real public key (protected endpoint; hidden when it fails). */
function useAgentPubkey() {
  const { isAuthenticated } = useAuthToken();
  const [pubkey, setPubkey] = useState<{ x: string; y: string } | null>(null);

  useEffect(() => {
    if (!isAuthenticated) {
      setPubkey(null);
      return;
    }
    let cancelled = false;
    api.agents
      .pubkey()
      .then((res) => {
        if (!cancelled) setPubkey({ x: res.pubkey_x, y: res.pubkey_y });
      })
      .catch(() => {
        if (!cancelled) setPubkey(null);
      });
    return () => {
      cancelled = true;
    };
  }, [isAuthenticated]);

  return pubkey;
}

export function AgentsPage() {
  useDocumentTitle("The Agent");
  const { data: agents, isLoading, error, refetch, isBackendDemo } = useAgents();
  const {
    data: strategies,
    isLoading: strategiesLoading,
    error: strategiesError,
    refetch: refetchStrategies,
  } = useStrategies();
  const pubkey = useAgentPubkey();

  const agent = agents[0];

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <FadeIn>
        <PageHeader
          title="The Otter Agent"
          subtitle="One protocol-operated agent. You set the limits and sign them — it executes your intents inside them, and nowhere else."
        />
      </FadeIn>

      {isBackendDemo && (
        <FadeIn delay={0.05}>
          <DemoDataNotice />
        </FadeIn>
      )}

      <FadeIn delay={0.05}>
        <SectionCard className="py-4">
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-border bg-secondary">
              <Lock className="h-4 w-4 text-accent" />
            </div>
            <p className="text-sm text-muted-foreground">
              <span className="font-medium text-foreground">Signed limits, not API keys.</span>{" "}
              Every execution carries a ZK proof that the action respects your delegation —
              verified on-chain before funds move.
            </p>
          </div>
        </SectionCard>
      </FadeIn>

      <FadeIn delay={0.1}>
        {isLoading ? (
          <Skeleton className="h-40 w-full" />
        ) : error ? (
          <ErrorState subject="the agent" onRetry={refetch} />
        ) : !agent ? (
          <EmptyState
            icon={<Bot className="h-6 w-6" />}
            title="Agent unavailable"
            description="The protocol agent is not responding. Check back soon."
          />
        ) : (
          <SectionCard>
            <div className="flex items-start gap-4">
              <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-border bg-secondary">
                <Bot className="h-6 w-6 text-accent" />
              </div>
              <div className="min-w-0">
                <p className="font-heading text-xl font-bold">{agent.name}</p>
                <p className="mt-1 text-sm text-muted-foreground">
                  Operated by {agent.operatedBy} · bonded ${agent.bond.toLocaleString()}
                </p>
                <p className="mt-2 text-sm text-muted-foreground">{agent.description}</p>
              </div>
            </div>
            {pubkey && (
              <div className="mt-4 rounded-lg border border-border/60 bg-secondary p-3">
                <p className="flex items-center gap-2 text-xs text-muted-foreground">
                  <KeyRound className="h-3.5 w-3.5 text-accent" />
                  Agent public key (BabyJubJub — signs nothing, only proves inside your limits)
                </p>
                <p className="mt-1 font-mono text-xs">x: {truncateHex(pubkey.x)}</p>
                <p className="font-mono text-xs">y: {truncateHex(pubkey.y)}</p>
              </div>
            )}
          </SectionCard>
        )}
      </FadeIn>

      {agent && !isLoading && !error && (
        <FadeIn delay={0.15}>
          <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
            <StatCard
              icon={ShieldCheck}
              label="Proofs submitted"
              value={agent.proofsSubmitted.toLocaleString()}
              className="h-full"
            />
            <StatCard
              icon={BookOpen}
              label="Yield routed"
              value={`$${(agent.yieldGenerated / 1_000_000).toFixed(1)}M`}
              className="h-full"
            />
            <StatCard
              icon={Lock}
              label="MEV rebated"
              value={`$${agent.mevCaptured.toLocaleString()}`}
              className="h-full"
            />
            <StatCard
              icon={Bot}
              label="Uptime"
              value={`${agent.uptime}%`}
              className="h-full"
            />
          </div>
        </FadeIn>
      )}

      <FadeIn delay={0.2}>
        <SectionCard
          title="Official strategies"
          subtitle="Ready-to-use intent templates published by the protocol, executed by the agent inside your own signed limits."
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
              description="The protocol will publish new intent templates over time."
            />
          ) : (
            <div className="space-y-3">
              {strategies.map((strategy) => (
                <DataRow key={strategy.id}>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium">{strategy.title}</p>
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
                    <Link to={`/app/intents/new?strategy=${strategy.id}`}>
                      Use strategy
                      <ArrowRight className="ml-1 h-3.5 w-3.5" />
                    </Link>
                  </Button>
                </DataRow>
              ))}
            </div>
          )}
        </SectionCard>
      </FadeIn>
    </div>
  );
}
