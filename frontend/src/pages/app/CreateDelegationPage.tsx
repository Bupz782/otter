import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { ArrowLeft, Check, Loader2 } from "lucide-react";
import { motion } from "framer-motion";
import { Card, CardContent, CardHeader, CardTitle, CardDescription, CardFooter } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Skeleton } from "@/components/ui/skeleton";
import { useAgents } from "@/hooks/useAgents";
import { useCreateDelegation } from "@/hooks/useCreateDelegation";
import { Link } from "react-router-dom";
import { cn } from "@/lib/utils";

const protocols = ["Aave", "Compound", "Uniswap"];
const chains = ["Ethereum", "Arbitrum"];

export function CreateDelegationPage() {
  const navigate = useNavigate();
  const { data: agents, isLoading: agentsLoading } = useAgents();
  const { mutate: create, isLoading: creating, data: created } = useCreateDelegation();

  const [selectedAgent, setSelectedAgent] = useState<string | null>(null);
  const [maxAmounts, setMaxAmounts] = useState({
    lend: 5000,
    swap: 2000,
    withdraw: 3000,
    claim: 1000,
  });
  const [allowedProtocols, setAllowedProtocols] = useState<string[]>(["Aave"]);
  const [allowedChains, setAllowedChains] = useState<string[]>(["Ethereum"]);
  const [expiryDays, setExpiryDays] = useState(30);

  const toggle = (list: string[], value: string, setter: (v: string[]) => void) => {
    if (list.includes(value)) {
      setter(list.filter((v) => v !== value));
    } else {
      setter([...list, value]);
    }
  };

  const handleCreate = async () => {
    if (!selectedAgent) return;
    await create({
      agentId: selectedAgent,
      maxAmounts,
      allowedProtocols,
      allowedChains,
      expiryDays,
    });
    setTimeout(() => navigate("/app/delegations"), 800);
  };

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <div className="flex items-center gap-4">
        <Button asChild variant="ghost" size="sm">
          <Link to="/app/delegations">
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to delegations
          </Link>
        </Button>
      </div>

      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        <h1 className="font-heading text-3xl font-bold tracking-tight">New Delegation</h1>
        <p className="text-muted-foreground">Authorize an agent to execute intents within your limits.</p>
      </motion.div>

      <Card>
        <CardHeader>
          <CardTitle>1. Select agent</CardTitle>
          <CardDescription>Choose an agent from the marketplace.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {agentsLoading ? (
            <div className="space-y-3">
              <Skeleton className="h-24 w-full" />
              <Skeleton className="h-24 w-full" />
            </div>
          ) : (
            agents?.map((agent) => (
              <button
                key={agent.id}
                type="button"
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
                    <p className="text-xs text-muted-foreground">{agent.proofsSubmitted.toLocaleString()} proofs</p>
                  </div>
                </div>
                <p className="mt-1 text-sm text-muted-foreground">{agent.description}</p>
              </button>
            ))
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>2. Set limits</CardTitle>
          <CardDescription>Define the maximum amounts per action type.</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4 sm:grid-cols-2">
          {Object.entries(maxAmounts).map(([key, value]) => (
            <div key={key} className="space-y-2">
              <Label htmlFor={key} className="capitalize">{key} limit (USDC)</Label>
              <Input
                id={key}
                type="number"
                value={value}
                onChange={(e) =>
                  setMaxAmounts((prev) => ({
                    ...prev,
                    [key]: Number.parseInt(e.target.value || "0", 10),
                  }))
                }
              />
            </div>
          ))}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>3. Allowed protocols & chains</CardTitle>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-3">
            <p className="text-sm font-medium">Protocols</p>
            <div className="flex flex-wrap gap-3">
              {protocols.map((protocol) => (
                <label key={protocol} className="flex items-center gap-2 rounded-lg border border-border/60 bg-card px-3 py-2">
                  <Checkbox
                    checked={allowedProtocols.includes(protocol)}
                    onCheckedChange={() => toggle(allowedProtocols, protocol, setAllowedProtocols)}
                  />
                  <span className="text-sm">{protocol}</span>
                </label>
              ))}
            </div>
          </div>
          <div className="space-y-3">
            <p className="text-sm font-medium">Chains</p>
            <div className="flex flex-wrap gap-3">
              {chains.map((chain) => (
                <label key={chain} className="flex items-center gap-2 rounded-lg border border-border/60 bg-card px-3 py-2">
                  <Checkbox
                    checked={allowedChains.includes(chain)}
                    onCheckedChange={() => toggle(allowedChains, chain, setAllowedChains)}
                  />
                  <span className="text-sm">{chain}</span>
                </label>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>4. Expiry</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <Label htmlFor="expiry">Delegation expires in (days)</Label>
          <Input
            id="expiry"
            type="number"
            value={expiryDays}
            onChange={(e) => setExpiryDays(Number.parseInt(e.target.value || "0", 10))}
          />
        </CardContent>
        <CardFooter>
          <Button
            onClick={handleCreate}
            disabled={!selectedAgent || creating || !!created}
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
        </CardFooter>
      </Card>
    </div>
  );
}
