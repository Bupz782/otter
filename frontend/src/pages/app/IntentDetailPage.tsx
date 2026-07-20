import { useState, type ReactNode } from "react";
import { useParams, Link } from "react-router-dom";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { ArrowLeft, Copy, Check, ExternalLink, Loader2 } from "lucide-react";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { IntentStatusBadge } from "@/components/app/IntentStatusBadge";
import { KineticTimeline } from "@/components/app/KineticTimeline";
import { ErrorState } from "@/components/app/ErrorState";
import { useIntent } from "@/hooks/useIntent";
import { useExecutionStatus } from "@/hooks/useExecutionStatus";
import { api } from "@/lib/api";
import { getStatusPresentation } from "@/lib/status";
import type { IntentStatus } from "@/types/app";

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

function explorerTxUrl(chain: string | undefined, hash: string): string {
  const base = chain?.toLowerCase().includes("arbitrum")
    ? "https://arbiscan.io"
    : "https://etherscan.io";
  return `${base}/tx/${hash}`;
}

const TERMINAL_STATUSES: IntentStatus[] = ["confirmed", "failed", "revoked"];

// Honest copy for intents with no execution record yet (or ever).
function idleDetail(status: IntentStatus): string {
  switch (status) {
    case "confirmed":
      return "Executed, but no execution record is on file.";
    case "failed":
      return "Execution failed before anything was recorded.";
    case "revoked":
      return "This intent was cancelled.";
    default:
      return "Otter checks the condition and executes when it hits.";
  }
}

export function IntentDetailPage() {
  useDocumentTitle("Intent");
  const { id } = useParams<{ id: string }>();
  const { data: intent, isLoading: intentLoading, error, refetch, isDemo } = useIntent(id);
  const {
    data: status,
    isLoading: statusLoading,
    error: statusError,
  } = useExecutionStatus(id, true);
  const [copied, setCopied] = useState(false);
  const [confirmCancel, setConfirmCancel] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  const [cancelError, setCancelError] = useState<string | null>(null);

  const handleCopyHash = async (hash: string) => {
    try {
      await navigator.clipboard.writeText(hash);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // Clipboard unavailable; leave the hash selectable instead.
    }
  };

  const handleCancel = async () => {
    if (!id) return;
    setCancelling(true);
    setCancelError(null);
    try {
      await api.intents.cancel(id);
      setConfirmCancel(false);
      refetch();
    } catch {
      setCancelError("Couldn't cancel the intent. Try again.");
    } finally {
      setCancelling(false);
    }
  };

  if (intentLoading) {
    return (
      <div className="mx-auto max-w-6xl space-y-6">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-40 w-full" />
        <Skeleton className="h-64 w-full" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="mx-auto max-w-3xl space-y-6">
        <ErrorState subject="this intent" onRetry={refetch} />
        <div className="flex justify-center">
          <Button asChild variant="ghost" size="sm">
            <Link to="/app/intents">
              <ArrowLeft className="mr-2 h-4 w-4" />
              Back to intents
            </Link>
          </Button>
        </div>
      </div>
    );
  }

  if (!intent) {
    return (
      <div className="mx-auto max-w-3xl">
        <SectionCard className="py-16 text-center">
          <h1 className="font-heading text-2xl font-bold">Intent not found</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            It may have been removed, or the link is wrong.
          </p>
          <Button asChild className="mt-6 rounded-full">
            <Link to="/app/intents">Back to intents</Link>
          </Button>
        </SectionCard>
      </div>
    );
  }

  const presentation = getStatusPresentation(intent.status);
  const IdleIcon = presentation.icon;
  const isTerminal = TERMINAL_STATUSES.includes(intent.status);

  const parameters = [
    { label: "Action", value: intent.parsed.type ?? "Not specified", className: "capitalize" },
    {
      label: "Amount",
      value: intent.parsed.amount != null ? intent.parsed.amount.toLocaleString() : "Not specified",
    },
    { label: "Asset", value: intent.parsed.asset ?? "Not specified" },
    { label: "Protocol", value: intent.parsed.protocol ?? "Not specified" },
    { label: "Chain", value: intent.parsed.chain ?? "Not specified" },
    { label: "Condition", value: intent.parsed.condition ?? "Not specified", wide: true },
  ];

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <FadeIn>
        <Button asChild variant="ghost" size="sm">
          <Link to="/app/intents">
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to intents
          </Link>
        </Button>
      </FadeIn>

      <div className="grid gap-6 lg:grid-cols-5">
        <div className="space-y-6 lg:col-span-3">
          <FadeIn delay={0.05}>
            <PageHeader
              title="Intent"
              subtitle={intent.rawText}
              action={<IntentStatusBadge status={intent.status} />}
            />
          </FadeIn>

          <FadeIn delay={0.1}>
            <SectionCard title="Parameters" subtitle="What Otter parsed from your words.">
              <dl className="grid gap-3 sm:grid-cols-2">
                {parameters.map((param) => (
                  <div
                    key={param.label}
                    className={
                      param.wide
                        ? "rounded-lg border border-border/60 bg-secondary p-3 sm:col-span-2"
                        : "rounded-lg border border-border/60 bg-secondary p-3"
                    }
                  >
                    <dt className="text-xs text-muted-foreground">{param.label}</dt>
                    <dd className={`mt-1 font-mono text-sm ${param.className ?? ""}`}>
                      {param.value}
                    </dd>
                  </div>
                ))}
              </dl>
            </SectionCard>
          </FadeIn>

          {intent.txHash && (
            <FadeIn delay={0.15}>
              <SectionCard title="Transaction">
                <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                  <div className="flex items-center gap-1">
                    <code className="break-all rounded-lg bg-secondary px-3 py-2 font-mono text-xs">
                      {intent.txHash}
                    </code>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => handleCopyHash(intent.txHash!)}
                      aria-label={copied ? "Copied" : "Copy transaction hash"}
                    >
                      {copied ? (
                        <Check className="h-4 w-4 text-emerald-400" />
                      ) : (
                        <Copy className="h-4 w-4" />
                      )}
                    </Button>
                    <span aria-live="polite" className="text-xs text-emerald-400">
                      {copied ? "Copied" : ""}
                    </span>
                    <Button asChild variant="ghost" size="icon">
                      <a
                        href={explorerTxUrl(intent.parsed.chain, intent.txHash)}
                        target="_blank"
                        rel="noreferrer"
                        aria-label="View transaction on block explorer"
                      >
                        <ExternalLink className="h-4 w-4" />
                      </a>
                    </Button>
                  </div>
                  <Badge variant="secondary" className="w-fit">
                    {intent.mevRebate ? `+${intent.mevRebate} USDC execution rebate` : "No rebate"}
                  </Badge>
                </div>
              </SectionCard>
            </FadeIn>
          )}
        </div>

        <div className="lg:col-span-2">
          <FadeIn delay={0.1}>
            <SectionCard title="Execution" subtitle="Where this intent stands.">
              {statusLoading ? (
                <KineticTimeline status={null} isLoading />
              ) : statusError ? (
                <p className="py-4 text-sm text-muted-foreground">
                  Execution history unavailable right now.
                </p>
              ) : status ? (
                <KineticTimeline status={status} />
              ) : (
                <div className="flex items-start gap-4 py-4">
                  <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full border border-border bg-secondary">
                    <IdleIcon className={`h-5 w-5 ${presentation.textClass}`} />
                  </div>
                  <div>
                    <p className="font-medium">{presentation.label}</p>
                    <p className="text-sm text-muted-foreground">{idleDetail(intent.status)}</p>
                  </div>
                </div>
              )}
              {!isTerminal && !isDemo && (
                <div className="border-t border-border/50 pt-4">
                  {confirmCancel ? (
                    <div className="flex flex-wrap items-center gap-3">
                      <p className="text-sm text-muted-foreground">
                        Cancel this intent? Otter stops monitoring it.
                      </p>
                      <Button
                        variant="destructive"
                        size="sm"
                        onClick={handleCancel}
                        disabled={cancelling}
                      >
                        {cancelling ? (
                          <>
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                            Canceling...
                          </>
                        ) : (
                          "Confirm cancel"
                        )}
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => {
                          setConfirmCancel(false);
                          setCancelError(null);
                        }}
                        disabled={cancelling}
                      >
                        Keep it
                      </Button>
                    </div>
                  ) : (
                    <Button
                      variant="outline"
                      size="sm"
                      className="border-rose-400/40 text-rose-400 hover:border-rose-400/60 hover:bg-rose-400/10 hover:text-rose-400"
                      onClick={() => setConfirmCancel(true)}
                    >
                      Cancel intent
                    </Button>
                  )}
                  {cancelError && (
                    <p role="alert" className="mt-2 text-sm text-rose-400">
                      {cancelError}
                    </p>
                  )}
                </div>
              )}
            </SectionCard>
          </FadeIn>
        </div>
      </div>
    </div>
  );
}
