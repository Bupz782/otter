import { Link } from "react-router-dom";
import { Bot, Star, ShieldCheck, TrendingUp, Coins, Activity, Plus, Lock } from "lucide-react";
import { motion } from "framer-motion";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { useAgents } from "@/hooks/useAgents";
import { EmptyState } from "@/components/app/EmptyState";

const riskVariant: Record<string, "default" | "secondary" | "outline" | "destructive"> = {
  Conservative: "default",
  Balanced: "secondary",
  Advanced: "outline",
};

export function AgentsPage() {
  const { data: agents, isLoading } = useAgents();

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="space-y-2"
      >
        <div>
          <h1 className="font-heading text-3xl font-bold tracking-tight">Otter Agents</h1>
          <p className="text-muted-foreground">
            Protocol-operated execution agents you can delegate to within your own limits.
          </p>
        </div>
      </motion.div>

      <Card className="border-accent/20 bg-accent-subtle/30">
        <CardContent className="flex flex-col gap-4 p-5 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-start gap-3">
            <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-accent-subtle text-accent">
              <Lock className="h-5 w-5" />
            </div>
            <div>
              <p className="font-heading text-base font-bold">Otter-curated agents only</p>
              <p className="text-sm text-muted-foreground">
                These agents are protocol-operated, bonded, and audited. You cannot upload or run your own agent.
              </p>
            </div>
          </div>
          <div className="shrink-0 text-xs text-muted-foreground">
            vs. user-run bots: signed limits, not API keys.
          </div>
        </CardContent>
      </Card>

      <Card id="onboarding-agents-list">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Bot className="h-5 w-5 text-accent" />
            Curated agents
          </CardTitle>
          <CardDescription>
            Each agent is bonded, audited, and can only act within the delegation limits you sign.
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {isLoading ? (
            <>
              <Skeleton className="h-56 w-full" />
              <Skeleton className="h-56 w-full" />
              <Skeleton className="h-56 w-full" />
            </>
          ) : agents?.length === 0 ? (
            <EmptyState
              icon={<Bot className="h-6 w-6" />}
              title="No agents available"
              description="Otter agents are protocol-curated. Check back soon."
            />
          ) : (
            agents?.map((agent, index) => (
              <Link
                key={agent.id}
                to={`/app/agents/${agent.id}`}
                className="group flex flex-col rounded-xl border border-border/60 bg-card p-5 transition-colors hover:border-accent/40"
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-3">
                    <div className="flex h-12 w-12 items-center justify-center rounded-full bg-accent-subtle text-accent">
                      <span className="font-heading text-lg font-bold">{agent.name.charAt(0)}</span>
                    </div>
                    <div>
                      <p className="font-heading text-lg font-bold transition-colors group-hover:text-accent">{agent.name}</p>
                      <div className="flex items-center gap-2 text-sm text-muted-foreground">
                        <Star className="h-3 w-3 text-accent" />
                        {agent.reputation}
                      </div>
                    </div>
                  </div>
                  <span id={index === 0 ? "onboarding-agents-risk" : undefined}>
                    <Badge variant={riskVariant[agent.riskProfile]} className="text-[10px]">{agent.riskProfile}</Badge>
                  </span>
                </div>

                <p className="mt-4 flex-1 text-sm text-muted-foreground">{agent.description}</p>

                <div className="mt-4 grid grid-cols-2 gap-2 text-sm">
                  <div className="rounded-lg bg-secondary p-2">
                    <p className="text-xs text-muted-foreground">Proofs</p>
                    <p className="font-medium">{agent.proofsSubmitted.toLocaleString()}</p>
                  </div>
                  <div className="rounded-lg bg-secondary p-2">
                    <p className="text-xs text-muted-foreground">Yield</p>
                    <p className="font-medium">${(agent.yieldGenerated / 1_000_000).toFixed(1)}M</p>
                  </div>
                  <div className="rounded-lg bg-secondary p-2">
                    <p className="text-xs text-muted-foreground">MEV</p>
                    <p className="font-medium">${agent.mevCaptured.toLocaleString()}</p>
                  </div>
                  <div className="rounded-lg bg-secondary p-2">
                    <p className="text-xs text-muted-foreground">Uptime</p>
                    <p className="font-medium">{agent.uptime}%</p>
                  </div>
                </div>

                <Button id={index === 0 ? "onboarding-agents-delegate" : undefined} className="mt-4 w-full rounded-full opacity-90 transition-opacity group-hover:opacity-100">
                  View agent
                </Button>
              </Link>
            ))
          )}
        </CardContent>
      </Card>
    </div>
  );
}
