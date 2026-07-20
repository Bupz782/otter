import { useEffect, useState, type ReactNode } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { ArrowLeft, Check, Loader2 } from "lucide-react";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { useAgents } from "@/hooks/useAgents";
import { useCreateDelegation } from "@/hooks/useCreateDelegation";
import { useAuthToken } from "@/hooks/useAuthToken";
import { ConnectWalletState } from "@/components/app/ConnectWalletState";
import { useStrategy } from "@/hooks/useStrategy";
import { cn } from "@/lib/utils";

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

const protocols = ["Aave", "Compound", "Uniswap"];
const chains = ["Ethereum", "Arbitrum"];

const numericInputClass = "rounded-lg border-border bg-secondary/60";

export function CreateDelegationPage() {
  useDocumentTitle("New Delegation");
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { isAuthenticated } = useAuthToken();
  const isDemo = !isAuthenticated;
  const strategyId = searchParams.get("strategy");
  const { data: strategy } = useStrategy(strategyId ?? undefined);
  const { data: agents, isLoading: agentsLoading } = useAgents();
  const { mutate: create, isLoading: creating, data: created } = useCreateDelegation();

  const [selectedAgent, setSelectedAgent] = useState<string | null>(null);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [maxAmounts, setMaxAmounts] = useState({
    lend: 5000,
    swap: 2000,
    withdraw: 3000,
    claim: 1000,
  });
  const [allowedProtocols, setAllowedProtocols] = useState<string[]>(["Aave"]);
  const [allowedChains, setAllowedChains] = useState<string[]>(["Ethereum"]);
  const [expiryDays, setExpiryDays] = useState(30);

  // Preselect an agent passed in from another page (e.g. /app/agents).
  useEffect(() => {
    const agentId = searchParams.get("agent");
    if (agentId && agents?.some((agent) => agent.id === agentId)) {
      setSelectedAgent(agentId);
    }
  }, [searchParams, agents]);

  // Prefill from a SocialFi strategy: its agent and protocol become the
  // delegation defaults (forking a strategy starts a matching delegation).
  useEffect(() => {
    if (strategy?.intent) {
      setSelectedAgent(strategy.agentId);
      if (strategy.intent.protocol && protocols.includes(strategy.intent.protocol)) {
        setAllowedProtocols([strategy.intent.protocol]);
      }
    }
  }, [strategy]);

  const limitsValid = Object.values(maxAmounts).every(
    (value) => Number.isFinite(value) && value > 0
  );
  const protocolsValid = allowedProtocols.length > 0;
  const chainsValid = allowedChains.length > 0;
  const expiryValid = Number.isFinite(expiryDays) && expiryDays >= 1;
  const formValid = !!selectedAgent && limitsValid && protocolsValid && chainsValid && expiryValid;

  const selectedAgentData = agents?.find((agent) => agent.id === selectedAgent);

  const toggle = (list: string[], value: string, setter: (v: string[]) => void) => {
    if (list.includes(value)) {
      setter(list.filter((v) => v !== value));
    } else {
      setter([...list, value]);
    }
  };

  const handleCreate = async () => {
    if (!selectedAgent || !formValid) return;
    setSubmitError(null);
    try {
      const newDelegation = await create({
        agentId: selectedAgent,
        maxAmounts,
        allowedProtocols,
        allowedChains,
        expiryDays,
      });
      setTimeout(() => {
        // Forking a strategy: carry delegation and strategy into intent creation.
        if (strategyId && newDelegation) {
          navigate(`/app/intents/new?delegation=${newDelegation.id}&strategy=${strategyId}`);
        } else {
          navigate("/app/delegations");
        }
      }, 800);
    } catch {
      setSubmitError("Couldn't sign the delegation. Try again.");
    }
  };

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <FadeIn>
        <Button asChild variant="ghost" size="sm">
          <Link to="/app/delegations">
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to delegations
          </Link>
        </Button>
      </FadeIn>

      <FadeIn delay={0.05}>
        <PageHeader
          title="New Delegation"
          subtitle="Set the limits. Sign. The agent works inside them and nowhere else."
        />
      </FadeIn>

      <FadeIn delay={0.1}>
        <SectionCard title="1. Agent" subtitle="Pick the Otter agent that executes for you.">
          {agentsLoading ? (
            <div className="space-y-3">
              <Skeleton className="h-24 w-full" />
              <Skeleton className="h-24 w-full" />
            </div>
          ) : (
            <div role="radiogroup" aria-label="Select an agent" className="space-y-3">
              {agents?.map((agent) => (
                <button
                  key={agent.id}
                  type="button"
                  role="radio"
                  aria-checked={selectedAgent === agent.id}
                  onClick={() => setSelectedAgent(agent.id)}
                  className={cn(
                    "w-full rounded-xl border p-4 text-left transition-colors",
                    selectedAgent === agent.id
                      ? "border-accent bg-accent-subtle"
                      : "border-border/60 bg-card hover:border-accent/40"
                  )}
                >
                  <div className="flex items-center justify-between">
                    <p className="font-heading text-lg font-bold">{agent.name}</p>
                    <div className="text-right">
                      <p className="text-sm font-medium">{agent.reputation} ★</p>
                      <p className="text-xs text-muted-foreground">
                        {agent.proofsSubmitted.toLocaleString()} proofs
                      </p>
                    </div>
                  </div>
                  <p className="mt-1 text-sm text-muted-foreground">{agent.description}</p>
                </button>
              ))}
            </div>
          )}
        </SectionCard>
      </FadeIn>

      <FadeIn delay={0.15}>
        <SectionCard title="2. Limits" subtitle="The most this agent can move per action.">
          <div className="grid gap-4 sm:grid-cols-2">
            {Object.entries(maxAmounts).map(([key, value]) => (
              <div key={key} className="space-y-2">
                <Label htmlFor={key} className="capitalize">
                  {key} limit (USDC)
                </Label>
                <Input
                  id={key}
                  type="number"
                  min="1"
                  aria-invalid={!Number.isFinite(value) || value <= 0}
                  value={value}
                  onChange={(e) =>
                    setMaxAmounts((prev) => ({
                      ...prev,
                      [key]: Number.parseInt(e.target.value || "0", 10),
                    }))
                  }
                  className={numericInputClass}
                />
              </div>
            ))}
            {!limitsValid && (
              <p role="alert" className="text-sm text-rose-400 sm:col-span-2">
                Limits must be numbers above 0.
              </p>
            )}
          </div>
        </SectionCard>
      </FadeIn>

      <FadeIn delay={0.2}>
        <SectionCard
          title="3. Protocols & chains"
          subtitle="Anything you leave unchecked stays off limits."
        >
          <div className="space-y-6">
            <div className="space-y-3">
              <p className="text-sm font-medium">Protocols</p>
              <div className="flex flex-wrap gap-3">
                {protocols.map((protocol) => (
                  <label
                    key={protocol}
                    className="flex items-center gap-2 rounded-lg border border-border/60 bg-card px-3 py-2"
                  >
                    <Checkbox
                      checked={allowedProtocols.includes(protocol)}
                      onCheckedChange={() =>
                        toggle(allowedProtocols, protocol, setAllowedProtocols)
                      }
                    />
                    <span className="text-sm">{protocol}</span>
                  </label>
                ))}
              </div>
              {!protocolsValid && (
                <p role="alert" className="text-sm text-rose-400">
                  Pick at least one protocol.
                </p>
              )}
            </div>
            <div className="space-y-3">
              <p className="text-sm font-medium">Chains</p>
              <div className="flex flex-wrap gap-3">
                {chains.map((chain) => (
                  <label
                    key={chain}
                    className="flex items-center gap-2 rounded-lg border border-border/60 bg-card px-3 py-2"
                  >
                    <Checkbox
                      checked={allowedChains.includes(chain)}
                      onCheckedChange={() => toggle(allowedChains, chain, setAllowedChains)}
                    />
                    <span className="text-sm">{chain}</span>
                  </label>
                ))}
              </div>
              <p className="text-xs text-muted-foreground">
                Chains are not part of the signed message yet. The circuit enforces amounts and
                protocols only.
              </p>
              {!chainsValid && (
                <p role="alert" className="text-sm text-rose-400">
                  Pick at least one chain.
                </p>
              )}
            </div>
          </div>
        </SectionCard>
      </FadeIn>

      <FadeIn delay={0.25}>
        <SectionCard title="4. Expiry">
          <div className="space-y-2">
            <Label htmlFor="expiry">Delegation expires in (days)</Label>
            <Input
              id="expiry"
              type="number"
              min="1"
              aria-invalid={!expiryValid}
              value={expiryDays}
              onChange={(e) => setExpiryDays(Number.parseInt(e.target.value || "0", 10))}
              className={numericInputClass}
            />
            {!expiryValid && (
              <p role="alert" className="text-sm text-rose-400">
                Expiry must be at least 1 day.
              </p>
            )}
          </div>
        </SectionCard>
      </FadeIn>

      <FadeIn delay={0.3}>
        <SectionCard title="5. Review & sign" subtitle="What your signature puts on record.">
          <div className="space-y-4">
            <dl className="grid gap-3 sm:grid-cols-2">
              <div className="rounded-lg border border-border/60 bg-secondary p-3">
                <dt className="text-xs text-muted-foreground">Agent</dt>
                <dd className="mt-1 font-heading text-lg font-bold">
                  {selectedAgentData?.name ?? "No agent selected"}
                </dd>
              </div>
              <div className="rounded-lg border border-border/60 bg-secondary p-3">
                <dt className="text-xs text-muted-foreground">Expires in</dt>
                <dd className="mt-1 font-heading text-lg font-bold">
                  {expiryValid ? `${expiryDays} days` : "Not set"}
                </dd>
              </div>
              <div className="rounded-lg border border-border/60 bg-secondary p-3">
                <dt className="text-xs text-muted-foreground">Protocols</dt>
                <dd className="mt-1 font-heading text-lg font-bold">
                  {allowedProtocols.length > 0 ? allowedProtocols.join(", ") : "None"}
                </dd>
              </div>
              <div className="rounded-lg border border-border/60 bg-secondary p-3">
                <dt className="text-xs text-muted-foreground">Chains</dt>
                <dd className="mt-1 font-heading text-lg font-bold">
                  {allowedChains.length > 0 ? allowedChains.join(", ") : "None"}
                </dd>
              </div>
              <div className="rounded-lg border border-border/60 bg-secondary p-3 sm:col-span-2">
                <dt className="text-xs text-muted-foreground">Limits</dt>
                <dd className="mt-1 font-heading text-lg font-bold">
                  {Object.entries(maxAmounts)
                    .map(([key, value]) => `${key} $${value.toLocaleString()}`)
                    .join(" · ")}
                </dd>
              </div>
            </dl>
            {submitError && (
              <p role="alert" className="text-sm text-rose-400">
                {submitError}
              </p>
            )}
          </div>
          <div className="mt-6">
            {isDemo ? (
              <ConnectWalletState message="Connect wallet to sign." className="w-full" />
            ) : (
              <Button
                onClick={handleCreate}
                disabled={!formValid || creating || !!created}
                className="w-full rounded-full"
              >
                {creating ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Signing...
                  </>
                ) : created ? (
                  <>
                    <Check className="mr-2 h-4 w-4" />
                    Delegated
                  </>
                ) : (
                  "Sign delegation"
                )}
              </Button>
            )}
          </div>
        </SectionCard>
      </FadeIn>
    </div>
  );
}
