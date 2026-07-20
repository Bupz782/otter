import type { ReactNode } from "react";
import { useParams, Link } from "react-router-dom";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { ArrowLeft, TrendingUp, ShieldCheck, Coins, Activity, Plus } from "lucide-react";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { StatCard } from "@/components/app/StatCard";
import { ErrorState } from "@/components/app/ErrorState";
import { useAgent } from "@/hooks/useAgent";

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

export function AgentDetailPage() {
  useDocumentTitle("Agent");
  const { agentId } = useParams<{ agentId: string }>();
  const { data: agent, isLoading, error, refetch } = useAgent(agentId);

  if (isLoading) {
    return (
      <div className="mx-auto max-w-6xl space-y-6">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-24 w-full" />
        <Skeleton className="h-64 w-full" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="mx-auto max-w-3xl space-y-6">
        <ErrorState subject="this agent" onRetry={refetch} />
        <div className="flex justify-center">
          <Button asChild variant="ghost" size="sm">
            <Link to="/app/agents">
              <ArrowLeft className="mr-2 h-4 w-4" />
              Back to agents
            </Link>
          </Button>
        </div>
      </div>
    );
  }

  if (!agent) {
    return (
      <div className="mx-auto max-w-3xl">
        <SectionCard className="py-16 text-center">
          <h1 className="font-heading text-2xl font-bold">Agent not found</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            It may have been removed, or the link is wrong.
          </p>
          <Button asChild className="mt-6 rounded-full">
            <Link to="/app/agents">Back to agents</Link>
          </Button>
        </SectionCard>
      </div>
    );
  }

  const about = [
    { label: "Operated by", value: agent.operatedBy },
    { label: "Risk profile", value: agent.riskProfile },
    { label: "Bond", value: `$${agent.bond.toLocaleString()}` },
    { label: "Reputation", value: `${agent.reputation} / 5` },
    { label: "Delegators", value: agent.followers.toLocaleString() },
    { label: "Strategies", value: String(agent.strategies) },
  ];

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <FadeIn>
        <Button asChild variant="ghost" size="sm">
          <Link to="/app/agents">
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to agents
          </Link>
        </Button>
      </FadeIn>

      <FadeIn delay={0.05}>
        <PageHeader
          title={agent.name}
          subtitle={agent.description}
          action={
            <Button asChild className="rounded-full">
              <Link to={`/app/delegations/new?agent=${agent.id}`}>
                <Plus className="mr-2 h-4 w-4" />
                Create delegation
              </Link>
            </Button>
          }
        />
      </FadeIn>

      <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
        <FadeIn delay={0.1}>
          <StatCard
            icon={TrendingUp}
            label="Yield generated"
            value={`$${agent.yieldGenerated.toLocaleString()}`}
            className="h-full"
          />
        </FadeIn>
        <FadeIn delay={0.15}>
          <StatCard
            icon={ShieldCheck}
            label="Proofs submitted"
            value={agent.proofsSubmitted.toLocaleString()}
            className="h-full"
          />
        </FadeIn>
        <FadeIn delay={0.2}>
          <StatCard
            icon={Coins}
            label="Rebates captured"
            value={`$${agent.mevCaptured.toLocaleString()}`}
            className="h-full"
          />
        </FadeIn>
        <FadeIn delay={0.25}>
          <StatCard icon={Activity} label="Uptime" value={`${agent.uptime}%`} className="h-full" />
        </FadeIn>
      </div>

      <FadeIn delay={0.3}>
        <SectionCard title="About" subtitle="What stands behind this agent.">
          <dl className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {about.map((item) => (
              <div key={item.label} className="rounded-lg border border-border/60 bg-secondary p-3">
                <dt className="text-xs text-muted-foreground">{item.label}</dt>
                <dd className="mt-1 font-mono text-sm">{item.value}</dd>
              </div>
            ))}
          </dl>
        </SectionCard>
      </FadeIn>
    </div>
  );
}
