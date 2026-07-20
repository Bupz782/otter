import { Link, useNavigate } from "react-router-dom";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { BookOpen, FilePlus, Sparkles, Trophy } from "lucide-react";
import { motion } from "framer-motion";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { useStrategies } from "@/hooks/useStrategies";
import { useLeaderboard } from "@/hooks/useLeaderboard";
import { EmptyState } from "@/components/app/EmptyState";
import { api } from "@/lib/api";

const riskVariant: Record<string, "default" | "secondary" | "outline" | "destructive"> = {
  Conservative: "default",
  Balanced: "secondary",
  Advanced: "outline",
};

export function StrategiesPage() {
  useDocumentTitle("Strategies");
  const navigate = useNavigate();
  const { data: strategies, isLoading: strategiesLoading } = useStrategies();
  const { data: leaderboard, isLoading: leaderboardLoading } = useLeaderboard();

  const handleFork = async (strategyId: string) => {
    try {
      await api.strategies.fork(strategyId);
      navigate(`/app/delegations/new?strategy=${strategyId}`);
    } catch (err) {
      // TODO surface toast
      console.error(err);
    }
  };

  return (
    <div className="mx-auto max-w-6xl space-y-8">
      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="space-y-2"
      >
        <h1 className="font-heading text-3xl font-bold tracking-tight">Strategies</h1>
        <p className="text-muted-foreground">
          Official Otter strategies you can use as a starting point.
        </p>
      </motion.div>

      <Card className="border-accent/20 bg-accent-subtle">
        <CardContent className="flex flex-col items-start justify-between gap-4 p-6 sm:flex-row sm:items-center">
          <div className="space-y-1">
            <p className="font-heading text-lg font-bold">Have a winning strategy?</p>
            <p className="text-sm text-muted-foreground">
              Publish your own Otter strategy and share it with the community.
            </p>
          </div>
          <Button asChild className="rounded-full">
            <Link to="/app/strategies/new">
              <FilePlus className="mr-2 h-4 w-4" />
              Publish strategy
            </Link>
          </Button>
        </CardContent>
      </Card>

      <div className="grid gap-8 lg:grid-cols-3">
        <Card id="onboarding-strategies-list" className="lg:col-span-2">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <BookOpen className="h-5 w-5 text-accent" />
              Official strategies
            </CardTitle>
            <CardDescription>Curated, audited strategies from Otter agents.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {strategiesLoading ? (
              <div className="space-y-3">
                <Skeleton className="h-32 w-full" />
                <Skeleton className="h-32 w-full" />
              </div>
            ) : strategies?.length === 0 ? (
              <EmptyState
                icon={<BookOpen className="h-6 w-6" />}
                title="No strategies yet"
                description="Otter will publish new strategies as agents are added."
              />
            ) : (
              strategies?.map((strategy, index) => (
                <div
                  key={strategy.id}
                  className="rounded-xl border border-border/60 bg-card p-5 transition-colors hover:border-accent/40"
                >
                  <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
                    <div className="flex-1">
                      <div className="flex flex-wrap items-center gap-2">
                        <p className="font-heading text-lg font-bold">{strategy.title}</p>
                        <Badge variant="secondary">{strategy.agentName}</Badge>
                        <Badge variant={riskVariant[strategy.riskProfile]} className="text-[10px]">
                          {strategy.riskProfile}
                        </Badge>
                      </div>
                      <p className="mt-1 text-sm text-muted-foreground">{strategy.description}</p>
                      <div className="mt-3 flex flex-wrap gap-3 text-xs text-muted-foreground">
                        <span>{strategy.copies.toLocaleString()} users</span>
                        <span>·</span>
                        <span>${strategy.totalVolume.toLocaleString()} volume</span>
                        {strategy.apy > 0 && (
                          <>
                            <span>·</span>
                            <span className="text-emerald-400">+{strategy.apy}% APY</span>
                          </>
                        )}
                      </div>
                    </div>
                    <Button
                      id={index === 0 ? "onboarding-strategies-use" : undefined}
                      variant="outline"
                      size="sm"
                      className="shrink-0"
                      onClick={() => handleFork(strategy.id)}
                    >
                      <Sparkles className="mr-2 h-4 w-4" />
                      Fork strategy
                    </Button>
                  </div>
                </div>
              ))
            )}
          </CardContent>
        </Card>

        <Card id="onboarding-strategies-leaderboard">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Trophy className="h-5 w-5 text-accent" />
              Agent Leaderboard
            </CardTitle>
            <CardDescription>Otter agents ranked by proof count.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {leaderboardLoading ? (
              <div className="space-y-3">
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
              </div>
            ) : (
              leaderboard?.slice(0, 5).map((entry) => (
                <Link
                  key={entry.agentId}
                  to={`/app/agents/${entry.agentId}`}
                  className="flex items-center justify-between rounded-lg border border-border/60 bg-card p-3 transition-colors hover:border-accent/40"
                >
                  <div className="flex items-center gap-3">
                    <span className="flex h-6 w-6 items-center justify-center rounded-full bg-secondary text-xs font-bold">
                      {entry.rank}
                    </span>
                    <span className="text-sm font-medium">{entry.agentName}</span>
                  </div>
                  <span className="text-xs text-muted-foreground">
                    {entry.proofsSubmitted.toLocaleString()} proofs
                  </span>
                </Link>
              ))
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
