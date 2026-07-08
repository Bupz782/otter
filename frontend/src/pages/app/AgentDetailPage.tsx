import { useParams, Link } from "react-router-dom";
import { ArrowLeft, Star, ShieldCheck, TrendingUp, Coins, Activity, Plus, Bot } from "lucide-react";
import { motion } from "framer-motion";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { useAgent } from "@/hooks/useAgent";

export function AgentDetailPage() {
  const { agentId } = useParams<{ agentId: string }>();
  const { data: agent, isLoading } = useAgent(agentId);

  if (isLoading) {
    return (
      <div className="mx-auto max-w-3xl space-y-6">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-64 w-full" />
      </div>
    );
  }

  if (!agent) {
    return (
      <div className="mx-auto max-w-3xl py-20 text-center">
        <h1 className="font-heading text-2xl font-bold">Agent not found</h1>
        <Button asChild className="mt-6 rounded-full">
          <Link to="/app/agents">Back to agents</Link>
        </Button>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <div className="flex items-center gap-4">
        <Button asChild variant="ghost" size="sm">
          <Link to="/app/agents">
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to agents
          </Link>
        </Button>
      </div>

      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        <Card>
          <CardHeader className="flex flex-col gap-4 sm:flex-row sm:items-start">
            <div className="flex h-16 w-16 items-center justify-center rounded-full bg-accent-subtle text-accent">
              <span className="font-heading text-2xl font-bold">{agent.name.charAt(0)}</span>
            </div>
            <div className="flex-1">
              <div className="flex flex-wrap items-center gap-3">
                <CardTitle className="font-heading text-2xl">{agent.name}</CardTitle>
                <Badge variant="secondary">${agent.bond.toLocaleString()} bond</Badge>
                <Badge variant="outline">{agent.riskProfile}</Badge>
              </div>
              <CardDescription className="mt-2">{agent.description}</CardDescription>
              <div className="mt-3 flex flex-wrap gap-4 text-sm text-muted-foreground">
                <span className="flex items-center gap-1">
                  <Star className="h-4 w-4 text-accent" /> {agent.reputation}
                </span>
                <span className="flex items-center gap-1">
                  <Bot className="h-4 w-4" /> {agent.followers.toLocaleString()} delegators
                </span>
                <span className="flex items-center gap-1">
                  <ShieldCheck className="h-4 w-4" /> {agent.proofsSubmitted.toLocaleString()}{" "}
                  proofs
                </span>
              </div>
            </div>
          </CardHeader>
        </Card>
      </motion.div>

      <div className="grid gap-4 sm:grid-cols-2">
        <Card>
          <CardHeader className="flex flex-row items-center gap-2">
            <TrendingUp className="h-5 w-5 text-accent" />
            <CardTitle>Yield Generated</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="font-heading text-3xl font-bold">
              ${agent.yieldGenerated.toLocaleString()}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center gap-2">
            <Coins className="h-5 w-5 text-accent" />
            <CardTitle>MEV Captured</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="font-heading text-3xl font-bold">${agent.mevCaptured.toLocaleString()}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center gap-2">
            <Activity className="h-5 w-5 text-accent" />
            <CardTitle>Uptime</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="font-heading text-3xl font-bold">{agent.uptime}%</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center gap-2">
            <ShieldCheck className="h-5 w-5 text-accent" />
            <CardTitle>Strategies</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="font-heading text-3xl font-bold">{agent.strategies}</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Delegate to {agent.name}</CardTitle>
          <CardDescription>
            Authorize this Otter agent to execute intents within limits you set.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button asChild className="rounded-full">
            <Link to={`/app/delegations/new?agent=${agent.id}`}>
              <Plus className="mr-2 h-4 w-4" />
              Create delegation
            </Link>
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
