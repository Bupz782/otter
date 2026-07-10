import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { ArrowLeft, Loader2, Check, Sparkles } from "lucide-react";
import { motion } from "framer-motion";
import { Card, CardContent, CardHeader, CardTitle, CardDescription, CardFooter } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { useAgents } from "@/hooks/useAgents";
import { useCreateStrategy } from "@/hooks/useCreateStrategy";
import { useParseIntent } from "@/hooks/useParseIntent";
import { Link } from "react-router-dom";
import type { Strategy } from "@/types/app";

const riskProfiles: Strategy["riskProfile"][] = ["Conservative", "Balanced", "Advanced"];

export function CreateStrategyPage() {
  const navigate = useNavigate();
  const { data: agents, isLoading: agentsLoading } = useAgents();
  const { mutate: create, isLoading: creating, data: created } = useCreateStrategy();
  const { parse, isLoading: parsing, data: parsed, reset } = useParseIntent();

  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [rawText, setRawText] = useState("");
  const [agentId, setAgentId] = useState<string | null>(null);
  const [riskProfile, setRiskProfile] = useState<Strategy["riskProfile"]>("Balanced");

  const handleParse = async () => {
    if (!rawText.trim()) return;
    reset();
    await parse(rawText);
  };

  const handleSubmit = async () => {
    if (!agentId || !parsed) return;
    await create({ title, description, rawText, agentId, riskProfile });
    setTimeout(() => navigate("/app/strategies"), 800);
  };

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <div className="flex items-center gap-4">
        <Button asChild variant="ghost" size="sm">
          <Link to="/app/strategies">
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back
          </Link>
        </Button>
      </div>
      <motion.div initial={{ opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.5 }}>
        <h1 className="font-heading text-3xl font-bold tracking-tight">Publish Strategy</h1>
        <p className="text-muted-foreground">Share an intent template with the Otter community.</p>
      </motion.div>

      <Card>
        <CardHeader>
          <CardTitle>Strategy details</CardTitle>
          <CardDescription>Write a clear title and the raw intent text Otter will parse.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="title">Title</Label>
            <Input id="title" value={title} onChange={(e) => setTitle(e.target.value)} placeholder="e.g. Steady USDC Lending" />
          </div>
          <div className="space-y-2">
            <Label htmlFor="description">Description</Label>
            <Textarea id="description" value={description} onChange={(e) => setDescription(e.target.value)} placeholder="Explain when and why this strategy is useful." />
          </div>
          <div className="space-y-2">
            <Label htmlFor="rawText">Intent text</Label>
            <Textarea id="rawText" value={rawText} onChange={(e) => setRawText(e.target.value)} placeholder="e.g. Lend 1000 USDC on Aave if yield > 3%" />
            <Button onClick={handleParse} disabled={!rawText.trim() || parsing} variant="outline" size="sm">
              {parsing ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Sparkles className="mr-2 h-4 w-4" />}
              Parse intent
            </Button>
            {parsed && <p className="text-sm text-emerald-400">Parsed: {parsed.type} {parsed.amount} {parsed.asset} on {parsed.protocol}</p>}
          </div>
          <div className="space-y-2">
            <Label>Agent</Label>
            {agentsLoading ? (
              <Skeleton className="h-24 w-full" />
            ) : (
              <div className="grid gap-3">
                {agents?.map((agent) => (
                  <button
                    key={agent.id}
                    type="button"
                    onClick={() => setAgentId(agent.id)}
                    className={`rounded-xl border p-4 text-left transition-colors ${agentId === agent.id ? "border-accent bg-accent-subtle" : "border-border/60 bg-card hover:border-accent/40"}`}
                  >
                    <p className="font-heading text-lg font-bold">{agent.name}</p>
                    <p className="text-sm text-muted-foreground">{agent.description}</p>
                  </button>
                ))}
              </div>
            )}
          </div>
          <div className="space-y-2">
            <Label>Risk profile</Label>
            <div className="flex gap-2">
              {riskProfiles.map((profile) => (
                <Badge
                  key={profile}
                  variant={riskProfile === profile ? "default" : "outline"}
                  className="cursor-pointer"
                  onClick={() => setRiskProfile(profile)}
                >
                  {profile}
                </Badge>
              ))}
            </div>
          </div>
        </CardContent>
        <CardFooter>
          <Button onClick={handleSubmit} disabled={!title || !description || !rawText || !agentId || !parsed || creating || !!created} className="w-full rounded-full">
            {creating ? <><Loader2 className="mr-2 h-4 w-4 animate-spin" /> Publishing...</> : created ? <><Check className="mr-2 h-4 w-4" /> Published</> : "Publish strategy"}
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}
