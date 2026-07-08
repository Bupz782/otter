import { useState, useCallback } from "react";
import { motion } from "framer-motion";
import { ArrowLeft, Bot } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { PromptInput } from "@/components/demo/PromptInput";
import { ReasoningSteps } from "@/components/demo/ReasoningSteps";
import { IntentResults } from "@/components/demo/IntentResults";
import { mockIntents } from "@/data/intents";

export function DemoPage() {
  const [isLoading, setIsLoading] = useState(false);
  const [showReasoning, setShowReasoning] = useState(false);
  const [showResults, setShowResults] = useState(false);
  const [submittedPrompt, setSubmittedPrompt] = useState("");
  const [selectedIntent, setSelectedIntent] = useState(mockIntents[0]);

  const handleSubmit = useCallback((prompt: string) => {
    setSubmittedPrompt(prompt);
    setIsLoading(true);
    setShowReasoning(true);
    setShowResults(false);

    const lower = prompt.toLowerCase();
    const matched =
      mockIntents.find((i) => lower.includes(i.parsed.asset.toLowerCase())) || mockIntents[0];
    setSelectedIntent(matched);
  }, []);

  const handleReasoningComplete = useCallback(() => {
    setIsLoading(false);
    setShowReasoning(false);
    setShowResults(true);
  }, []);

  return (
    <div className="relative min-h-screen bg-background px-6 pt-32 pb-20">
      <div className="mx-auto max-w-5xl">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        >
          <Button asChild variant="ghost" size="sm" className="mb-8">
            <Link to="/">
              <ArrowLeft className="mr-2 h-4 w-4" />
              Back home
            </Link>
          </Button>

          <div className="mb-12 text-center">
            <div className="mb-4 inline-flex h-10 w-10 items-center justify-center rounded-xl border border-border bg-secondary">
              <Bot className="h-5 w-5 text-accent" />
            </div>
            <h1 className="font-heading text-4xl font-bold tracking-tight text-foreground sm:text-5xl">
              Try an intent
            </h1>
            <p className="mx-auto mt-4 max-w-xl text-lg text-muted-foreground">
              Describe a conditional DeFi action and see how Otter parses, proves, and executes it.
            </p>
          </div>

          <PromptInput onSubmit={handleSubmit} isLoading={isLoading} />

          {showReasoning && (
            <ReasoningSteps isActive={showReasoning} onComplete={handleReasoningComplete} />
          )}

          {isLoading && !showReasoning && (
            <div className="mx-auto max-w-3xl space-y-4 py-8">
              {[...Array(3)].map((_, i) => (
                <Skeleton key={i} className="h-24 w-full rounded-xl" />
              ))}
            </div>
          )}

          {showResults && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ duration: 0.5 }}
            >
              <div className="mx-auto mb-6 max-w-3xl text-center">
                <p className="text-sm text-muted-foreground">
                  Simulated execution for:{" "}
                  <span className="text-foreground">{submittedPrompt}</span>
                </p>
              </div>
              <IntentResults intent={selectedIntent} />
            </motion.div>
          )}
        </motion.div>
      </div>
    </div>
  );
}
