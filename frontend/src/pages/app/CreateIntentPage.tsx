import { useState, useEffect, type ReactNode } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Sparkles, Check, Loader2, ArrowRight, ShieldCheck, Bot } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { Stepper } from "@/components/app/Stepper";
import { EmptyState } from "@/components/app/EmptyState";
import { ErrorState } from "@/components/app/ErrorState";
import { ConnectWalletState } from "@/components/app/ConnectWalletState";
import { useParseIntent } from "@/hooks/useParseIntent";
import { useCreateIntent } from "@/hooks/useCreateIntent";
import { useDelegations } from "@/hooks/useDelegations";
import { useAgents } from "@/hooks/useAgents";
import { useStrategies } from "@/hooks/useStrategies";
import { useStrategy } from "@/hooks/useStrategy";
import { useAuthToken } from "@/hooks/useAuthToken";
import { matchMockIntent, type MockIntent } from "@/data/intents";
import { truncateHash } from "@/lib/utils";
import type { IntentType, ParsedIntent } from "@/types/app";

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

const examples = [
  "Lend 1000 USDC on Aave if yield > 3%",
  "Swap 500 USDC to ETH on Uniswap when gas < 20 gwei",
  "Claim Aave rewards every Monday",
  "Withdraw 2000 USDC from Compound if utilization > 85%",
];

const steps = [
  { label: "Describe", description: "Say what you want" },
  { label: "Review", description: "Check the parse" },
  { label: "Delegate", description: "Pick a signed delegation" },
  { label: "Confirm", description: "Set it live" },
];

// Demo-mode parse: map the landing-page mock parser output onto the wizard's
// ParsedIntent shape so visitors can try the flow without a session.
function mapMockToParsedIntent(mock: MockIntent): ParsedIntent {
  const action = mock.parsed.action.toLowerCase();
  const type: IntentType =
    action === "lend"
      ? "lend"
      : action === "withdraw"
        ? "withdraw"
        : action === "claim"
          ? "claim"
          : "swap";
  const amount = Number.parseFloat(mock.parsed.amount.replace(/,/g, ""));
  return {
    type,
    amount: Number.isFinite(amount) ? amount : 0,
    asset: mock.parsed.asset,
    protocol: mock.parsed.target,
    condition: mock.parsed.condition || undefined,
    chain: mock.parsed.chain || undefined,
  };
}

export function CreateIntentPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { isAuthenticated } = useAuthToken();
  const isDemo = !isAuthenticated;
  const { data: strategies } = useStrategies();
  const delegationId = searchParams.get("delegation");
  const strategyId = searchParams.get("strategy");
  const agentId = searchParams.get("agent");
  const { data: strategy } = useStrategy(strategyId ?? undefined);
  const [step, setStep] = useState(0);
  const [text, setText] = useState("");
  const [selectedDelegation, setSelectedDelegation] = useState<string | null>(null);
  const { isLoading: parsing, parse, reset } = useParseIntent();
  const { mutate: create, isLoading: creating, data: created } = useCreateIntent();
  const {
    data: delegations,
    isLoading: delegationsLoading,
    error: delegationsError,
    refetch: refetchDelegations,
  } = useDelegations();
  const { data: agents } = useAgents();
  const [parseError, setParseError] = useState<string | null>(null);
  const [createError, setCreateError] = useState<string | null>(null);
  // Editable copy of the parsed intent, so a misparsed field can be corrected
  // on the Review step. This draft drives the compatibility check and the
  // confirm summary; the backend still re-parses the raw text server-side.
  const [parsedDraft, setParsedDraft] = useState<ParsedIntent | null>(null);

  useEffect(() => {
    if (strategyId) {
      // Authenticated: strategy detail from the API; demo mode: fall back to
      // the local strategy list so the prefill works without a session.
      const prefill = strategy ?? strategies?.find((s) => s.id === strategyId);
      if (prefill) setText(prefill.rawText);
    } else if (agentId && delegations) {
      const delegation = delegations.find((d) => d.agentId === agentId);
      if (delegation) setSelectedDelegation(delegation.id);
    }
    if (delegationId && delegations) {
      setSelectedDelegation(delegationId);
    }
  }, [searchParams, strategy, strategies, delegations, agentId, delegationId, strategyId]);

  const handleParse = async () => {
    if (!text.trim()) return;
    setParseError(null);
    reset();
    if (isDemo) {
      // No session: parse locally with the landing-page mock parser.
      setParsedDraft(mapMockToParsedIntent(matchMockIntent(text)));
      setStep(1);
      return;
    }
    try {
      const result = await parse(text);
      if (result) {
        setParsedDraft(result);
        setStep(1);
      }
    } catch {
      setParseError("Couldn't parse that intent. Try rephrasing.");
    }
  };

  const updateDraft = (patch: Partial<ParsedIntent>) => {
    setParsedDraft((prev) => (prev ? { ...prev, ...patch } : prev));
  };

  const handleDelegate = () => {
    if (!selectedDelegation) return;
    setStep(3);
  };

  const handleCreate = async () => {
    if (!parsedDraft || !selectedDelegation) return;
    setCreateError(null);
    try {
      await create({ rawText: text, parsed: parsedDraft, delegationId: selectedDelegation });
      setTimeout(() => navigate("/app/intents"), 1200);
    } catch {
      setCreateError("Couldn't create the intent. Try again.");
    }
  };

  const activeDelegation = delegations?.find((d) => d.id === selectedDelegation);
  const selectedAgent = agents?.find((a) => activeDelegation?.agentId === a.id);
  const delegationLabel =
    activeDelegation?.agentName ??
    selectedAgent?.name ??
    (selectedDelegation ? `Delegation ${truncateHash(selectedDelegation)}` : "Not specified");

  // Warn when the parsed intent conflicts with the selected delegation's
  // known fields. Only fields that exist on the record are checked; when the
  // backend returns no limits there is nothing to compare and no warning.
  const compatWarnings: string[] = [];
  if (parsedDraft && activeDelegation) {
    const allowed = activeDelegation.allowedProtocols;
    if (allowed && allowed.length > 0 && !allowed.includes(parsedDraft.protocol)) {
      compatWarnings.push(
        `${parsedDraft.protocol} is not in this delegation's protocol list (${allowed.join(", ")}).`
      );
    }
    const max = activeDelegation.maxAmounts?.[parsedDraft.type];
    if (max !== undefined && parsedDraft.amount > max) {
      compatWarnings.push(
        `${parsedDraft.amount} ${parsedDraft.asset} is above this delegation's ${parsedDraft.type} limit of $${max.toLocaleString()}.`
      );
    }
  }

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <FadeIn>
        <PageHeader
          title="Create Intent"
          subtitle="Say it in plain English. Otter proves the rest."
        />
      </FadeIn>

      <FadeIn delay={0.05}>
        <SectionCard>
          <Stepper steps={steps} currentStep={created ? 4 : step} />
        </SectionCard>
      </FadeIn>

      <FadeIn delay={0.1}>
        <AnimatePresence mode="wait">
          {step === 0 && (
            <motion.div
              key="describe"
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
            >
              <SectionCard
                title="What do you want to do?"
                subtitle="Otter parses it, then proves every execution against your limits."
              >
                <div className="space-y-4">
                  <Textarea
                    value={text}
                    onChange={(e) => setText(e.target.value)}
                    placeholder="e.g. Lend 1000 USDC on Aave if yield > 3%"
                    className="min-h-[140px] resize-none rounded-2xl border-border/60 bg-card/80 px-5 py-4 text-base backdrop-blur-md"
                  />
                  <div className="space-y-2">
                    <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                      Try an example
                    </p>
                    <div className="flex flex-wrap gap-2">
                      {examples.map((example) => (
                        <button
                          key={example}
                          type="button"
                          onClick={() => {
                            setText(example);
                            reset();
                            setParsedDraft(null);
                          }}
                          className="inline-flex items-center gap-1.5 rounded-full border border-border/60 bg-secondary/50 px-3 py-1 text-xs text-muted-foreground transition-colors hover:border-accent/40 hover:text-accent"
                        >
                          <Sparkles className="h-3 w-3 text-accent" />
                          {example}
                        </button>
                      ))}
                    </div>
                  </div>
                  {parseError && (
                    <p role="alert" className="text-sm text-rose-400">
                      {parseError}
                    </p>
                  )}
                  <div className="flex justify-end">
                    <Button
                      onClick={handleParse}
                      disabled={!text.trim() || parsing}
                      className="rounded-full"
                    >
                      {parsing ? (
                        <>
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          Parsing...
                        </>
                      ) : (
                        <>
                          Parse intent
                          <ArrowRight className="ml-2 h-4 w-4" />
                        </>
                      )}
                    </Button>
                  </div>
                </div>
              </SectionCard>
            </motion.div>
          )}

          {step === 1 && parsedDraft && (
            <motion.div
              key="review"
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
            >
              <SectionCard
                title="How Otter read it"
                subtitle="Check the parse. Fix anything Otter got wrong before you delegate."
              >
                <div className="grid gap-4 sm:grid-cols-2">
                  <div className="rounded-lg border border-border/60 bg-secondary p-3">
                    <p className="text-xs text-muted-foreground">Action</p>
                    <p className="font-heading text-lg font-bold capitalize">{parsedDraft.type}</p>
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="parsed-amount">Amount</Label>
                    <Input
                      id="parsed-amount"
                      type="number"
                      min="0"
                      value={parsedDraft.amount}
                      onChange={(e) =>
                        updateDraft({ amount: Number.parseFloat(e.target.value) || 0 })
                      }
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="parsed-asset">Asset</Label>
                    <Input
                      id="parsed-asset"
                      value={parsedDraft.asset}
                      onChange={(e) => updateDraft({ asset: e.target.value })}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="parsed-protocol">Protocol</Label>
                    <Input
                      id="parsed-protocol"
                      value={parsedDraft.protocol}
                      onChange={(e) => updateDraft({ protocol: e.target.value })}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="parsed-chain">Chain</Label>
                    <Input
                      id="parsed-chain"
                      placeholder="Any chain"
                      value={parsedDraft.chain ?? ""}
                      onChange={(e) => updateDraft({ chain: e.target.value || undefined })}
                    />
                  </div>
                  <div className="space-y-2 sm:col-span-2">
                    <Label htmlFor="parsed-condition">Condition</Label>
                    <Input
                      id="parsed-condition"
                      placeholder="No condition"
                      value={parsedDraft.condition ?? ""}
                      onChange={(e) => updateDraft({ condition: e.target.value || undefined })}
                    />
                  </div>
                </div>
                <div className="mt-6 flex justify-end gap-3">
                  <Button variant="ghost" onClick={() => setStep(0)}>
                    Back
                  </Button>
                  <Button onClick={() => setStep(2)} className="rounded-full">
                    Choose delegation
                    <ArrowRight className="ml-2 h-4 w-4" />
                  </Button>
                </div>
              </SectionCard>
            </motion.div>
          )}

          {step === 2 && (
            <motion.div
              key="delegate"
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
            >
              <SectionCard
                title="Choose a delegation"
                subtitle="Pick the agent and the signed limits this intent runs under."
              >
                <div className="space-y-3">
                  {delegationsLoading ? (
                    <div className="space-y-3">
                      <Skeleton className="h-24 w-full" />
                      <Skeleton className="h-24 w-full" />
                    </div>
                  ) : delegationsError ? (
                    <ErrorState subject="delegations" onRetry={refetchDelegations} />
                  ) : delegations?.length === 0 ? (
                    <EmptyState
                      icon={<Bot className="h-6 w-6" />}
                      title="No active delegations"
                      description="Intents need a delegation to run under. Set one up first."
                      action={
                        <Button
                          onClick={() => navigate("/app/delegations/new")}
                          className="rounded-full"
                        >
                          Create delegation
                        </Button>
                      }
                    />
                  ) : (
                    <div role="radiogroup" aria-label="Choose a delegation" className="space-y-3">
                      {delegations?.map((delegation) => {
                        const agent = agents?.find((a) => a.id === delegation.agentId);
                        return (
                          <button
                            key={delegation.id}
                            type="button"
                            role="radio"
                            aria-checked={selectedDelegation === delegation.id}
                            onClick={() => setSelectedDelegation(delegation.id)}
                            className={`w-full rounded-xl border p-4 text-left transition-colors ${
                              selectedDelegation === delegation.id
                                ? "border-accent bg-accent-subtle"
                                : "border-border/60 bg-card hover:border-accent/40"
                            }`}
                          >
                            <div className="flex items-center justify-between">
                              <div className="flex items-center gap-3">
                                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-accent-subtle text-accent">
                                  <span className="font-heading text-sm font-bold">
                                    {delegation.agentName?.charAt(0) ?? "?"}
                                  </span>
                                </div>
                                <div>
                                  <p className="font-medium">
                                    {delegation.agentName ??
                                      `Delegation ${truncateHash(delegation.id)}`}
                                  </p>
                                  <p className="text-xs text-muted-foreground">
                                    {agent
                                      ? `${agent.operatedBy}-operated · ${agent.riskProfile}`
                                      : "Agent details unavailable"}
                                  </p>
                                </div>
                              </div>
                              {selectedDelegation === delegation.id && (
                                <Badge variant="default">Selected</Badge>
                              )}
                            </div>
                            <p className="mt-2 text-xs text-muted-foreground">
                              {delegation.maxAmounts
                                ? `Max lend $${delegation.maxAmounts.lend.toLocaleString()}`
                                : "No limits on record"}
                              {delegation.allowedProtocols
                                ? ` · ${delegation.allowedProtocols.join(", ")}`
                                : ""}
                              {delegation.expiry
                                ? ` · Expires ${new Date(delegation.expiry).toLocaleDateString()}`
                                : ""}
                            </p>
                          </button>
                        );
                      })}
                    </div>
                  )}
                  {compatWarnings.length > 0 && (
                    <div
                      role="alert"
                      className="rounded-lg border border-amber-400/30 bg-amber-400/10 p-3"
                    >
                      <p className="text-xs font-medium text-amber-400">
                        Heads up: this intent may not run under the selected delegation.
                      </p>
                      <ul className="mt-1 list-inside list-disc space-y-0.5 text-xs text-amber-400/90">
                        {compatWarnings.map((warning) => (
                          <li key={warning}>{warning}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
                <div className="mt-6 flex justify-end gap-3">
                  <Button variant="ghost" onClick={() => setStep(1)}>
                    Back
                  </Button>
                  <Button
                    onClick={handleDelegate}
                    disabled={!selectedDelegation}
                    className="rounded-full"
                  >
                    Confirm delegation
                  </Button>
                </div>
              </SectionCard>
            </motion.div>
          )}

          {step === 3 && (
            <motion.div
              key="confirm"
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
            >
              <SectionCard title="Confirm intent" subtitle="One last look before it goes live.">
                <div className="space-y-4">
                  <div className="rounded-xl border border-border/60 bg-secondary p-4">
                    <p className="text-xs text-muted-foreground">Intent</p>
                    <p className="font-heading text-lg font-bold">{text}</p>
                  </div>
                  <div className="grid gap-3 sm:grid-cols-2">
                    <div className="rounded-lg border border-border/60 bg-secondary p-3">
                      <p className="text-xs text-muted-foreground">Action</p>
                      <p className="font-heading text-lg font-bold capitalize">
                        {parsedDraft?.type ?? "Not specified"}
                      </p>
                    </div>
                    <div className="rounded-lg border border-border/60 bg-secondary p-3">
                      <p className="text-xs text-muted-foreground">Amount</p>
                      <p className="font-heading text-lg font-bold">
                        {parsedDraft
                          ? `${parsedDraft.amount} ${parsedDraft.asset}`
                          : "Not specified"}
                      </p>
                    </div>
                    <div className="rounded-lg border border-border/60 bg-secondary p-3">
                      <p className="text-xs text-muted-foreground">Protocol</p>
                      <p className="font-heading text-lg font-bold">
                        {parsedDraft?.protocol || "Not specified"}
                      </p>
                    </div>
                    <div className="rounded-lg border border-border/60 bg-secondary p-3">
                      <p className="text-xs text-muted-foreground">Chain</p>
                      <p className="font-heading text-lg font-bold">
                        {parsedDraft?.chain ?? "Not specified"}
                      </p>
                    </div>
                    <div className="rounded-lg border border-border/60 bg-secondary p-3">
                      <p className="text-xs text-muted-foreground">Condition</p>
                      <p className="font-heading text-lg font-bold">
                        {parsedDraft?.condition ?? "Not specified"}
                      </p>
                    </div>
                    <div className="rounded-lg border border-border/60 bg-secondary p-3">
                      <p className="text-xs text-muted-foreground">Delegation</p>
                      <p className="font-heading text-lg font-bold">{delegationLabel}</p>
                    </div>
                  </div>
                  <div className="flex items-start gap-2 rounded-lg border border-accent/20 bg-accent-subtle p-3">
                    <ShieldCheck className="mt-0.5 h-4 w-4 shrink-0 text-accent" />
                    <p className="text-xs text-accent-foreground">
                      Otter proves this intent stays inside your delegation limits before anything
                      executes.
                    </p>
                  </div>
                  {createError && (
                    <p role="alert" className="text-sm text-rose-400">
                      {createError}
                    </p>
                  )}
                </div>
                <div className="mt-6 flex justify-end gap-3">
                  <Button variant="ghost" onClick={() => setStep(2)}>
                    Back
                  </Button>
                  {isDemo ? (
                    <ConnectWalletState message="Connect wallet to set it live." />
                  ) : (
                    <Button
                      onClick={handleCreate}
                      disabled={creating || !!created}
                      className="rounded-full"
                    >
                      {creating ? (
                        <>
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          Creating...
                        </>
                      ) : created ? (
                        <>
                          <Check className="mr-2 h-4 w-4" />
                          Created
                        </>
                      ) : (
                        "Set intent"
                      )}
                    </Button>
                  )}
                </div>
              </SectionCard>
            </motion.div>
          )}
        </AnimatePresence>
      </FadeIn>
    </div>
  );
}
