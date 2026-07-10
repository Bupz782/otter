import { useState, useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Sparkles, Check, Loader2, ArrowRight, ShieldCheck, Bot } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
  CardFooter,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { Stepper } from "@/components/app/Stepper";
import { EmptyState } from "@/components/app/EmptyState";
import { useOnboardingContext } from "@/components/app/OnboardingProvider";
import { useParseIntent } from "@/hooks/useParseIntent";
import { useCreateIntent } from "@/hooks/useCreateIntent";
import { useDelegations } from "@/hooks/useDelegations";
import { useAgents } from "@/hooks/useAgents";
import { useStrategy } from "@/hooks/useStrategy";

const examples = [
  "Lend 1000 USDC on Aave if yield > 3%",
  "Swap 500 USDC to ETH on Uniswap when gas < 20 gwei",
  "Claim Aave rewards every Monday",
  "Withdraw 2000 USDC from Compound if utilization > 85%",
];

const steps = [
  { label: "Describe", description: "Type your intent" },
  { label: "Review", description: "Check parsing" },
  { label: "Delegate", description: "Pick limits" },
  { label: "Confirm", description: "Submit intent" },
];

export function CreateIntentPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const delegationId = searchParams.get("delegation");
  const strategyId = searchParams.get("strategy");
  const agentId = searchParams.get("agent");
  const { data: strategy } = useStrategy(strategyId ?? undefined);
  const [step, setStep] = useState(0);
  const [text, setText] = useState("");
  const [selectedDelegation, setSelectedDelegation] = useState<string | null>(null);
  const { data: parsed, isLoading: parsing, parse, reset } = useParseIntent();
  const { mutate: create, isLoading: creating, data: created } = useCreateIntent();
  const { data: delegations, isLoading: delegationsLoading } = useDelegations();
  const { data: agents } = useAgents();
  const { step: onboardingStep } = useOnboardingContext();

  useEffect(() => {
    const map: Record<string, number> = {
      "create-intent-input": 0,
      "create-intent-review": 1,
      "create-intent-delegate": 2,
      "create-intent-confirm": 3,
    };
    if (onboardingStep && onboardingStep in map) setStep(map[onboardingStep]);
  }, [onboardingStep]);

  useEffect(() => {
    if (strategyId && strategy) {
      setText(strategy.rawText);
    } else if (agentId && delegations) {
      const delegation = delegations.find((d) => d.agentId === agentId);
      if (delegation) setSelectedDelegation(delegation.id);
    }
    if (delegationId && delegations) {
      setSelectedDelegation(delegationId);
    }
  }, [searchParams, strategy, delegations, agentId, delegationId, strategyId]);

  const handleParse = async () => {
    if (!text.trim()) return;
    reset();
    const result = await parse(text);
    if (result) setStep(1);
  };

  const handleDelegate = () => {
    if (!selectedDelegation) return;
    setStep(3);
  };

  const handleCreate = async () => {
    if (!parsed || !selectedDelegation) return;
    await create({ rawText: text, parsed, delegationId: selectedDelegation });
    setTimeout(() => navigate("/app/intents"), 1200);
  };

  const selectedAgent = agents?.find(
    (a) => delegations?.find((d) => d.id === selectedDelegation)?.agentId === a.id
  );

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="space-y-2"
      >
        <h1 className="font-heading text-3xl font-bold tracking-tight">Create Intent</h1>
        <p className="text-muted-foreground">
          Describe a conditional DeFi action in plain English.
        </p>
      </motion.div>

      <div id="onboarding-create-intent-stepper">
        <Stepper steps={steps} currentStep={created ? 4 : step} />
      </div>

      <AnimatePresence mode="wait">
        {step === 0 && (
          <motion.div
            key="describe"
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
          >
            <Card id="onboarding-create-intent-input">
              <CardHeader>
                <CardTitle>What do you want to do?</CardTitle>
                <CardDescription>
                  Otter parses your intent and enforces it with a zero-knowledge proof.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <Textarea
                  value={text}
                  onChange={(e) => setText(e.target.value)}
                  placeholder="e.g. Lend 1000 USDC on Aave if yield > 3%"
                  className="min-h-[140px] resize-none"
                />
                <div className="space-y-2">
                  <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                    Try an example
                  </p>
                  <div className="flex flex-wrap gap-2">
                    {examples.map((example) => (
                      <Button
                        key={example}
                        variant="outline"
                        size="sm"
                        onClick={() => {
                          setText(example);
                          reset();
                        }}
                      >
                        <Sparkles className="mr-2 h-3 w-3 text-accent" />
                        {example}
                      </Button>
                    ))}
                  </div>
                </div>
                <Button
                  onClick={handleParse}
                  disabled={!text.trim() || parsing}
                  className="w-full rounded-full"
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
              </CardContent>
            </Card>
          </motion.div>
        )}

        {step === 1 && parsed && (
          <motion.div
            key="review"
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
            className="space-y-4"
          >
            <Card id="onboarding-create-intent-review">
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Check className="h-5 w-5 text-emerald-400" />
                  Parsed Intent
                </CardTitle>
                <CardDescription>Review the detected parameters before continuing.</CardDescription>
              </CardHeader>
              <CardContent className="grid gap-3 sm:grid-cols-2">
                <div className="rounded-lg border border-border/60 bg-secondary p-3">
                  <p className="text-xs text-muted-foreground">Action</p>
                  <p className="font-heading text-lg font-bold capitalize">{parsed.type}</p>
                </div>
                <div className="rounded-lg border border-border/60 bg-secondary p-3">
                  <p className="text-xs text-muted-foreground">Amount</p>
                  <p className="font-heading text-lg font-bold">
                    {parsed.amount} {parsed.asset}
                  </p>
                </div>
                <div className="rounded-lg border border-border/60 bg-secondary p-3">
                  <p className="text-xs text-muted-foreground">Protocol</p>
                  <p className="font-heading text-lg font-bold">{parsed.protocol}</p>
                </div>
                <div className="rounded-lg border border-border/60 bg-secondary p-3">
                  <p className="text-xs text-muted-foreground">Chain</p>
                  <p className="font-heading text-lg font-bold">{parsed.chain}</p>
                </div>
                {parsed.condition && (
                  <div className="rounded-lg border border-border/60 bg-secondary p-3 sm:col-span-2">
                    <p className="text-xs text-muted-foreground">Condition</p>
                    <p className="font-heading text-lg font-bold">{parsed.condition}</p>
                  </div>
                )}
              </CardContent>
              <CardFooter className="flex justify-between">
                <Button variant="ghost" onClick={() => setStep(0)}>
                  Back
                </Button>
                <Button onClick={() => setStep(2)} className="rounded-full">
                  Choose agent
                  <ArrowRight className="ml-2 h-4 w-4" />
                </Button>
              </CardFooter>
            </Card>
          </motion.div>
        )}

        {step === 2 && (
          <motion.div
            key="delegate"
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
          >
            <Card id="onboarding-create-intent-delegate">
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <ShieldCheck className="h-5 w-5 text-accent" />
                  Choose a delegation
                </CardTitle>
                <CardDescription>
                  Select an Otter agent and the limits that will execute this intent.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                {delegationsLoading ? (
                  <div className="space-y-3">
                    <Skeleton className="h-24 w-full" />
                    <Skeleton className="h-24 w-full" />
                  </div>
                ) : delegations?.length === 0 ? (
                  <EmptyState
                    icon={<Bot className="h-6 w-6" />}
                    title="No active delegations"
                    description="Create a delegation to an Otter agent before you can set an intent."
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
                  delegations?.map((delegation) => {
                    const agent = agents?.find((a) => a.id === delegation.agentId);
                    return (
                      <button
                        key={delegation.id}
                        type="button"
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
                                {delegation.agentName.charAt(0)}
                              </span>
                            </div>
                            <div>
                              <p className="font-medium">{delegation.agentName}</p>
                              <p className="text-xs text-muted-foreground">
                                {agent?.operatedBy}-operated · {agent?.riskProfile}
                              </p>
                            </div>
                          </div>
                          {selectedDelegation === delegation.id && (
                            <Badge variant="default">Selected</Badge>
                          )}
                        </div>
                        <p className="mt-2 text-xs text-muted-foreground">
                          Max lend ${delegation.maxAmounts.lend.toLocaleString()} ·{" "}
                          {delegation.allowedProtocols.join(", ")} · Expires{" "}
                          {new Date(delegation.expiry).toLocaleDateString()}
                        </p>
                      </button>
                    );
                  })
                )}
              </CardContent>
              <CardFooter className="flex justify-between">
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
              </CardFooter>
            </Card>
          </motion.div>
        )}

        {step === 3 && (
          <motion.div
            key="confirm"
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
          >
            <Card id="onboarding-create-intent-confirm">
              <CardHeader>
                <CardTitle>Confirm intent</CardTitle>
                <CardDescription>Review everything before submitting.</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="rounded-xl border border-border/60 bg-secondary p-4">
                  <p className="text-xs text-muted-foreground">Intent</p>
                  <p className="font-heading text-lg font-bold">{text}</p>
                </div>{" "}
                <div className="grid gap-3 sm:grid-cols-2">
                  <div className="rounded-lg border border-border/60 bg-secondary p-3">
                    <p className="text-xs text-muted-foreground">Action</p>
                    <p className="font-heading text-lg font-bold capitalize">{parsed?.type}</p>
                  </div>
                  <div className="rounded-lg border border-border/60 bg-secondary p-3">
                    <p className="text-xs text-muted-foreground">Agent</p>
                    <p className="font-heading text-lg font-bold">{selectedAgent?.name}</p>
                  </div>
                </div>
                <div className="flex items-start gap-2 rounded-lg border border-accent/20 bg-accent-subtle p-3">
                  <ShieldCheck className="mt-0.5 h-4 w-4 shrink-0 text-accent" />
                  <p className="text-xs text-accent-foreground">
                    Otter will generate a zero-knowledge proof proving this intent respects your
                    delegation limits before execution.
                  </p>
                </div>
              </CardContent>
              <CardFooter className="flex justify-between">
                <Button variant="ghost" onClick={() => setStep(2)}>
                  Back
                </Button>
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
              </CardFooter>
            </Card>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
