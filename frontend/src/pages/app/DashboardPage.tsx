import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import {
  Wallet,
  TrendingUp,
  Zap,
  Radar,
  Coins,
  Lightbulb,
  Activity,
  Plus,
  ArrowRight,
  ChevronRight,
} from "lucide-react";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { StatCard } from "@/components/app/StatCard";
import { DataRow } from "@/components/app/DataRow";
import { CountUp } from "@/components/app/CountUp";
import { EmptyState } from "@/components/app/EmptyState";
import { ErrorState } from "@/components/app/ErrorState";
import { usePortfolio } from "@/hooks/usePortfolio";
import { useIntents } from "@/hooks/useIntents";
import { useActivity } from "@/hooks/useActivity";
import { getStatusPresentation } from "@/lib/status";
import { cn } from "@/lib/utils";
import type { ActivityItem } from "@/types/app";

const EASE: [number, number, number, number] = [0.22, 1, 0.36, 1];

/** Mount-only fade/slide used to stagger the dashboard cards. */
function FadeIn({
  children,
  delay = 0,
  className,
  id,
}: {
  children: ReactNode;
  delay?: number;
  className?: string;
  id?: string;
}) {
  return (
    <motion.div
      id={id}
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4, ease: EASE, delay }}
      className={className}
    >
      {children}
    </motion.div>
  );
}

function formatCurrency(value: number): string {
  return `$${value.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })}`;
}

/** Activity dots reuse the status palette: emerald for money events. */
const ACTIVITY_DOT: Record<ActivityItem["type"], string> = {
  deposit: getStatusPresentation("confirmed").dotClass,
  intent_executed: getStatusPresentation("confirmed").dotClass,
  mev_rebate: getStatusPresentation("confirmed").dotClass,
  intent_created: getStatusPresentation("submitted").dotClass,
  delegation_created: getStatusPresentation("monitoring").dotClass,
  withdraw: getStatusPresentation("revoked").dotClass,
};

export function DashboardPage() {
  useDocumentTitle("Dashboard");
  const {
    data: portfolio,
    isLoading: portfolioLoading,
    error: portfolioError,
    refetch: refetchPortfolio,
  } = usePortfolio();
  const {
    data: intents,
    isLoading: intentsLoading,
    error: intentsError,
    refetch: refetchIntents,
  } = useIntents();
  const {
    data: activity,
    isLoading: activityLoading,
    error: activityError,
    refetch: refetchActivity,
  } = useActivity();

  const activeIntents = intents.filter(
    (i) =>
      i.status === "monitoring" ||
      i.status === "condition_met" ||
      i.status === "proving" ||
      i.status === "submitted"
  );

  const allocatedPct =
    portfolio && portfolio.totalBalance > 0
      ? Math.min(100, (portfolio.allocated / portfolio.totalBalance) * 100)
      : 0;

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <FadeIn>
        <PageHeader
          title="Dashboard"
          subtitle="What your capital is doing right now."
          action={
            <Button asChild id="onboarding-dashboard-create" className="rounded-full">
              <Link to="/app/intents/new">
                <Plus className="mr-2 h-4 w-4" />
                Create intent
              </Link>
            </Button>
          }
        />
      </FadeIn>

      <div className="grid gap-6 lg:grid-cols-3">
        <FadeIn id="onboarding-dashboard-balance" delay={0.05} className="lg:col-span-2">
          <SectionCard className="h-full">
            {portfolioError ? (
              <ErrorState subject="your vault" onRetry={refetchPortfolio} />
            ) : portfolioLoading ? (
              <div className="space-y-6">
                <Skeleton className="h-12 w-56" />
                <Skeleton className="h-2 w-full rounded-full" />
                <div className="grid grid-cols-2 gap-4">
                  <Skeleton className="h-10 w-full" />
                  <Skeleton className="h-10 w-full" />
                </div>
              </div>
            ) : (
              <>
                <p className="text-xs uppercase tracking-wider text-muted-foreground">
                  Total vault balance
                </p>
                <p className="mt-2 font-heading text-4xl font-bold tabular-nums md:text-5xl">
                  <CountUp value={portfolio?.totalBalance ?? 0} prefix="$" decimals={2} />
                </p>

                <div className="mt-8">
                  <div className="h-2 overflow-hidden rounded-full bg-secondary">
                    <div
                      className="h-full rounded-full bg-accent"
                      style={{ width: `${allocatedPct}%` }}
                    />
                  </div>
                  <div className="mt-3 grid grid-cols-2 gap-4">
                    <div>
                      <p className="text-xs text-muted-foreground">Allocated</p>
                      <p className="mt-0.5 font-heading text-lg font-bold tabular-nums">
                        {formatCurrency(portfolio?.allocated ?? 0)}
                      </p>
                    </div>
                    <div>
                      <p className="text-xs text-muted-foreground">Available</p>
                      <p className="mt-0.5 font-heading text-lg font-bold tabular-nums">
                        {formatCurrency(portfolio?.available ?? 0)}
                      </p>
                    </div>
                  </div>
                </div>

                <div className="mt-6 grid grid-cols-2 gap-4 border-t border-border/50 pt-4">
                  <div>
                    <p className="text-xs text-muted-foreground">Yield earned</p>
                    <p
                      className={cn(
                        "mt-0.5 text-sm font-medium tabular-nums",
                        (portfolio?.yieldEarned ?? 0) >= 0 ? "text-emerald-400" : "text-rose-400"
                      )}
                    >
                      {(portfolio?.yieldEarned ?? 0) >= 0 ? "+" : ""}
                      {formatCurrency(portfolio?.yieldEarned ?? 0)}
                    </p>
                  </div>
                  <div>
                    <p className="text-xs text-muted-foreground">Execution rebates</p>
                    <p
                      className={cn(
                        "mt-0.5 text-sm font-medium tabular-nums",
                        (portfolio?.mevRebates ?? 0) >= 0 ? "text-emerald-400" : "text-rose-400"
                      )}
                    >
                      {(portfolio?.mevRebates ?? 0) >= 0 ? "+" : ""}
                      {formatCurrency(portfolio?.mevRebates ?? 0)}
                    </p>
                  </div>
                </div>
              </>
            )}
          </SectionCard>
        </FadeIn>

        <FadeIn id="onboarding-dashboard-activity" delay={0.1}>
          <SectionCard title="Recent activity" className="h-full">
            {activityLoading ? (
              <div className="space-y-3">
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
              </div>
            ) : activityError ? (
              <ErrorState subject="recent activity" onRetry={refetchActivity} />
            ) : activity.length === 0 ? (
              <EmptyState
                icon={<Activity className="h-6 w-6" />}
                title="No activity yet"
                description="Once an intent executes, every move shows up here."
              />
            ) : (
              <div className="space-y-3">
                {activity.slice(0, 4).map((item) => (
                  <DataRow key={item.id}>
                    <span
                      className={cn(
                        "h-2 w-2 shrink-0 rounded-full",
                        ACTIVITY_DOT[item.type] ?? "bg-accent"
                      )}
                    />
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm">{item.title}</p>
                      <p className="text-xs text-muted-foreground">
                        {new Date(item.timestamp).toLocaleString()}
                      </p>
                    </div>
                  </DataRow>
                ))}
              </div>
            )}
          </SectionCard>
        </FadeIn>
      </div>

      {!portfolioError && (
        <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
          <FadeIn delay={0.15}>
            <StatCard
              icon={Wallet}
              label="Allocated"
              value={
                portfolioLoading ? (
                  <Skeleton className="h-7 w-24" />
                ) : (
                  formatCurrency(portfolio?.allocated ?? 0)
                )
              }
              hint={portfolio ? `${portfolio.positions.length} positions` : undefined}
            />
          </FadeIn>
          <FadeIn delay={0.2}>
            <StatCard
              icon={TrendingUp}
              label="Yield earned"
              value={
                portfolioLoading ? (
                  <Skeleton className="h-7 w-24" />
                ) : (
                  formatCurrency(portfolio?.yieldEarned ?? 0)
                )
              }
            />
          </FadeIn>
          <FadeIn delay={0.25}>
            <StatCard
              icon={Zap}
              label="Execution rebates"
              value={
                portfolioLoading ? (
                  <Skeleton className="h-7 w-24" />
                ) : (
                  formatCurrency(portfolio?.mevRebates ?? 0)
                )
              }
            />
          </FadeIn>
          <FadeIn delay={0.3}>
            <StatCard
              icon={Radar}
              label="Active intents"
              value={intentsLoading ? <Skeleton className="h-7 w-12" /> : activeIntents.length}
            />
          </FadeIn>
        </div>
      )}

      <FadeIn id="onboarding-dashboard-intents" delay={0.35}>
        <SectionCard
          title="Active intents"
          action={
            <Button asChild variant="ghost" size="sm">
              <Link to="/app/intents">
                View all
                <ArrowRight className="ml-1 h-3 w-3" />
              </Link>
            </Button>
          }
        >
          {intentsLoading ? (
            <div className="space-y-3">
              <Skeleton className="h-16 w-full" />
              <Skeleton className="h-16 w-full" />
            </div>
          ) : intentsError ? (
            <ErrorState subject="active intents" onRetry={refetchIntents} />
          ) : activeIntents.length === 0 ? (
            <EmptyState
              icon={<Lightbulb className="h-6 w-6" />}
              title="Nothing in the water yet."
              description="Set your first intent and Otter starts watching the market."
              action={
                <Button asChild className="rounded-full">
                  <Link to="/app/intents/new">Create intent</Link>
                </Button>
              }
            />
          ) : (
            <div className="space-y-3">
              {activeIntents.slice(0, 3).map((intent) => {
                const status = getStatusPresentation(intent.status);
                return (
                  <Link key={intent.id} to={`/app/intents/${intent.id}`} className="block">
                    <DataRow>
                      <div className="flex w-28 shrink-0 items-center gap-2">
                        <span className={cn("h-2 w-2 rounded-full", status.dotClass)} />
                        <span className={cn("truncate text-xs", status.textClass)}>
                          {status.label}
                        </span>
                      </div>
                      <div className="min-w-0 flex-1">
                        <p className="truncate text-sm font-medium">{intent.rawText}</p>
                        <p className="truncate font-mono text-xs text-muted-foreground">
                          {intent.parsed.condition ?? "Not specified"}
                        </p>
                      </div>
                      <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
                    </DataRow>
                  </Link>
                );
              })}
            </div>
          )}
        </SectionCard>
      </FadeIn>

      <FadeIn id="onboarding-dashboard-positions" delay={0.4}>
        <SectionCard title="Positions" subtitle="Where your capital sits.">
          {portfolioLoading ? (
            <Skeleton className="h-40 w-full" />
          ) : portfolioError ? (
            <ErrorState subject="positions" onRetry={refetchPortfolio} />
          ) : !portfolio || portfolio.positions.length === 0 ? (
            <EmptyState
              icon={<Coins className="h-6 w-6" />}
              title="No open positions"
              description="When an intent puts capital to work, it lands here."
            />
          ) : (
            <>
              <div className="hidden overflow-x-auto md:block">
                <table className="w-full text-left text-sm">
                  <thead>
                    <tr className="border-b border-border text-muted-foreground">
                      <th scope="col" className="pb-3 font-medium">
                        Asset
                      </th>
                      <th scope="col" className="pb-3 font-medium">
                        Protocol
                      </th>
                      <th scope="col" className="pb-3 font-medium">
                        Chain
                      </th>
                      <th scope="col" className="pb-3 text-right font-medium">
                        Amount
                      </th>
                      <th scope="col" className="pb-3 text-right font-medium">
                        APY
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-border/50">
                    {portfolio.positions.map((position) => (
                      <tr key={`${position.asset}-${position.protocol}-${position.chain}`}>
                        <td className="py-3 font-medium">{position.asset}</td>
                        <td className="py-3 text-muted-foreground">{position.protocol}</td>
                        <td className="py-3 text-muted-foreground">{position.chain}</td>
                        <td className="py-3 text-right tabular-nums">
                          {position.amount.toLocaleString()} {position.asset}
                        </td>
                        <td className="py-3 text-right tabular-nums text-emerald-400">
                          +{position.apy}%
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <div className="space-y-3 md:hidden">
                {portfolio.positions.map((position) => (
                  <DataRow key={`${position.asset}-${position.protocol}-${position.chain}`}>
                    <div className="min-w-0 flex-1">
                      <p className="text-sm font-medium">{position.asset}</p>
                      <p className="text-xs text-muted-foreground">
                        {position.protocol} · {position.chain}
                      </p>
                    </div>
                    <div className="shrink-0 text-right">
                      <p className="text-sm tabular-nums">
                        {position.amount.toLocaleString()} {position.asset}
                      </p>
                      <p className="text-xs tabular-nums text-emerald-400">+{position.apy}% APY</p>
                    </div>
                  </DataRow>
                ))}
              </div>
            </>
          )}
        </SectionCard>
      </FadeIn>
    </div>
  );
}
