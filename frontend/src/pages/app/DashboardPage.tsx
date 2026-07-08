import { Link } from "react-router-dom";
import { Wallet, TrendingUp, Coins, Lightbulb, ArrowRight, Plus } from "lucide-react";
import { motion } from "framer-motion";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { usePortfolio } from "@/hooks/usePortfolio";
import { useIntents } from "@/hooks/useIntents";
import { useActivity } from "@/hooks/useActivity";
import { IntentStatusBadge } from "@/components/app/IntentStatusBadge";
import { CountUp } from "@/components/app/CountUp";
import { EmptyState } from "@/components/app/EmptyState";
import { ShimmerCard } from "@/components/app/ShimmerCard";

const container = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: { staggerChildren: 0.08 },
  },
};

const item = {
  hidden: { opacity: 0, y: 12 },
  show: { opacity: 1, y: 0 },
};

function StatCard({
  title,
  value,
  prefix,
  suffix,
  decimals,
  sub,
  icon,
  loading,
  highlight,
}: {
  title: string;
  value: number;
  prefix?: string;
  suffix?: string;
  decimals?: number;
  sub?: string;
  icon: React.ReactNode;
  loading?: boolean;
  highlight?: boolean;
}) {
  return (
    <Card className={highlight ? "border-accent/30" : undefined}>
      <CardContent className="p-6">
        <div className="flex items-start justify-between">
          <div>
            <p className="text-sm text-muted-foreground">{title}</p>
            {loading ? (
              <Skeleton className="mt-2 h-8 w-32" />
            ) : (
              <p className="mt-2 font-heading text-2xl font-bold">
                <CountUp value={value} prefix={prefix} suffix={suffix} decimals={decimals} />
              </p>
            )}
            {sub && !loading && <p className="mt-1 text-xs text-muted-foreground">{sub}</p>}
          </div>
          <div className="rounded-xl border border-border bg-secondary p-2 text-accent">{icon}</div>
        </div>
      </CardContent>
    </Card>
  );
}

export function DashboardPage() {
  const { data: portfolio, isLoading: portfolioLoading } = usePortfolio();
  const { data: intents, isLoading: intentsLoading } = useIntents();
  const { data: activity, isLoading: activityLoading } = useActivity();

  const activeIntents = intents.filter(
    (i) =>
      i.status === "monitoring" ||
      i.status === "condition_met" ||
      i.status === "proving" ||
      i.status === "submitted"
  );

  return (
    <div className="mx-auto max-w-6xl space-y-8">
      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="space-y-2"
      >
        <h1 className="font-heading text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground">
          Overview of your vault, intents, and recent activity.
        </p>
      </motion.div>

      <ShimmerCard
        id="onboarding-dashboard-balance"
        className="rounded-xl border border-accent/20 bg-gradient-to-br from-accent-subtle/40 to-transparent"
      >
        <div className="p-6 sm:p-8">
          <div className="flex flex-col gap-6 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <p className="text-sm text-muted-foreground">Total vault balance</p>
              {portfolioLoading ? (
                <Skeleton className="mt-2 h-12 w-48" />
              ) : (
                <p className="mt-1 font-heading text-5xl font-bold tracking-tight">
                  <CountUp value={portfolio?.totalBalance ?? 0} prefix="$" decimals={2} />
                </p>
              )}
              <p className="mt-2 text-sm text-muted-foreground">
                {portfolio
                  ? `${portfolio.positions.length} positions · $${portfolio.available.toLocaleString()} available`
                  : ""}
              </p>
            </div>
            <div className="flex gap-3">
              <Button asChild className="rounded-full">
                <Link to="/app/intents/new">
                  <Plus className="mr-2 h-4 w-4" />
                  Create intent
                </Link>
              </Button>
              <Button asChild variant="outline" className="rounded-full">
                <Link to="/app/strategies">Browse strategies</Link>
              </Button>
            </div>
          </div>
        </div>
      </ShimmerCard>

      <motion.div
        variants={container}
        initial="hidden"
        animate="show"
        className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4"
      >
        <motion.div variants={item}>
          <StatCard
            title="Allocated"
            value={portfolio?.allocated ?? 0}
            prefix="$"
            sub={portfolio ? `${portfolio.positions.length} positions` : undefined}
            icon={<Wallet className="h-5 w-5" />}
            loading={portfolioLoading}
          />
        </motion.div>
        <motion.div variants={item}>
          <StatCard
            title="Yield Earned"
            value={portfolio?.yieldEarned ?? 0}
            prefix="+$"
            decimals={2}
            icon={<TrendingUp className="h-5 w-5" />}
            loading={portfolioLoading}
            highlight
          />
        </motion.div>
        <motion.div variants={item}>
          <StatCard
            title="MEV Rebates"
            value={portfolio?.mevRebates ?? 0}
            prefix="+$"
            decimals={2}
            icon={<Coins className="h-5 w-5" />}
            loading={portfolioLoading}
            highlight
          />
        </motion.div>
        <motion.div variants={item}>
          <StatCard
            title="Active Intents"
            value={activeIntents.length}
            icon={<Lightbulb className="h-5 w-5" />}
            loading={intentsLoading}
          />
        </motion.div>
      </motion.div>

      <div className="grid gap-8 lg:grid-cols-3">
        <Card id="onboarding-dashboard-intents" className="lg:col-span-2">
          <CardHeader className="flex flex-row items-center justify-between">
            <div>
              <CardTitle>Active Intents</CardTitle>
              <CardDescription>Intents currently being monitored or executed.</CardDescription>
            </div>
            <Button asChild size="sm" variant="outline">
              <Link to="/app/intents">View all</Link>
            </Button>
          </CardHeader>
          <CardContent className="space-y-3">
            {intentsLoading ? (
              <div className="space-y-3">
                <Skeleton className="h-20 w-full" />
                <Skeleton className="h-20 w-full" />
              </div>
            ) : activeIntents.length === 0 ? (
              <EmptyState
                icon={<Lightbulb className="h-6 w-6" />}
                title="No active intents"
                description="Create your first intent and Otter will start monitoring the market for you."
                action={
                  <Button asChild className="rounded-full">
                    <Link to="/app/intents/new">Create your first intent</Link>
                  </Button>
                }
              />
            ) : (
              activeIntents.map((intent) => (
                <Link
                  key={intent.id}
                  to={`/app/intents/${intent.id}`}
                  className="group flex items-center justify-between rounded-xl border border-border/60 bg-card p-4 transition-colors hover:border-accent/40"
                >
                  <div className="space-y-1">
                    <p className="font-medium transition-colors group-hover:text-accent">
                      {intent.rawText}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {intent.parsed.amount} {intent.parsed.asset} · {intent.parsed.protocol} ·{" "}
                      {intent.parsed.chain}
                    </p>
                  </div>
                  <IntentStatusBadge status={intent.status} />
                </Link>
              ))
            )}
          </CardContent>
        </Card>

        <Card id="onboarding-dashboard-activity">
          <CardHeader>
            <CardTitle>Recent Activity</CardTitle>
            <CardDescription>Latest vault events.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {activityLoading ? (
              <div className="space-y-3">
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
              </div>
            ) : (
              activity.slice(0, 6).map((item) => (
                <div key={item.id} className="flex items-start gap-3">
                  <div className="mt-1.5 h-2 w-2 rounded-full bg-accent" />
                  <div className="flex-1">
                    <p className="text-sm">{item.title}</p>
                    <p className="text-xs text-muted-foreground">
                      {new Date(item.timestamp).toLocaleString()}
                    </p>
                  </div>
                </div>
              ))
            )}
          </CardContent>
        </Card>
      </div>

      <Card id="onboarding-dashboard-positions">
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>Positions</CardTitle>
            <CardDescription>Where your capital is currently allocated.</CardDescription>
          </div>
          <Button asChild variant="outline" size="sm">
            <Link to="/app/strategies">
              Explore strategies
              <ArrowRight className="ml-2 h-3 w-3" />
            </Link>
          </Button>
        </CardHeader>
        <CardContent className="overflow-x-auto">
          {portfolioLoading ? (
            <Skeleton className="h-40 w-full" />
          ) : (
            <table className="w-full text-left text-sm">
              <thead>
                <tr className="border-b border-border text-muted-foreground">
                  <th className="pb-3 font-medium">Asset</th>
                  <th className="pb-3 font-medium">Protocol</th>
                  <th className="pb-3 font-medium">Chain</th>
                  <th className="pb-3 font-medium">Amount</th>
                  <th className="pb-3 font-medium">Value</th>
                  <th className="pb-3 font-medium">APY</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border/50">
                {portfolio?.positions.map((position) => (
                  <tr key={`${position.asset}-${position.protocol}-${position.chain}`}>
                    <td className="py-3 font-medium">{position.asset}</td>
                    <td className="py-3 text-muted-foreground">{position.protocol}</td>
                    <td className="py-3 text-muted-foreground">{position.chain}</td>
                    <td className="py-3">{position.amount.toLocaleString()}</td>
                    <td className="py-3">${position.value.toLocaleString()}</td>
                    <td className="py-3 text-emerald-400">+{position.apy}%</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
