import { motion } from "framer-motion";
import { CheckCircle2, Hash, Fuel, Box, ArrowRightLeft, Target } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { MockIntent } from "@/data/intents";

interface IntentResultsProps {
  intent: MockIntent;
}

export function IntentResults({ intent }: IntentResultsProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, ease: [0.22, 1, 0.36, 1] }}
      className="mx-auto max-w-3xl space-y-4"
    >
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <CheckCircle2 className="h-4 w-4 text-emerald-400" />
        <span>Intent parsed and ready for execution</span>
      </div>

      <Card className="border-border/60 bg-card/80 backdrop-blur-sm">
        <CardContent className="space-y-6 p-6">
          <div className="flex justify-end">
            <Badge variant="secondary" className="w-fit rounded-full bg-accent-subtle text-accent">
              {intent.parsed.chain}
            </Badge>
          </div>

          <div className="grid gap-4 sm:grid-cols-2">
            <div className="rounded-lg border border-border/50 bg-secondary/40 p-4">
              <div className="mb-2 flex items-center gap-2 text-xs text-muted-foreground">
                <ArrowRightLeft className="h-3.5 w-3.5" />
                Action
              </div>
              <p className="text-sm font-medium text-foreground capitalize">
                {intent.parsed.action}
              </p>
              <p className="text-sm text-muted-foreground">{intent.parsed.amount}</p>
            </div>
            <div className="rounded-lg border border-border/50 bg-secondary/40 p-4">
              <div className="mb-2 flex items-center gap-2 text-xs text-muted-foreground">
                <Target className="h-3.5 w-3.5" />
                Condition
              </div>
              <p className="text-sm font-medium text-foreground">{intent.parsed.condition}</p>
              <p className="text-sm text-muted-foreground">{intent.parsed.target}</p>
            </div>
          </div>

          <div className="space-y-2 rounded-lg border border-border/50 bg-secondary/40 p-4">
            <p className="text-xs text-muted-foreground">Execution simulation</p>
            <div className="flex items-center gap-2 text-sm text-foreground">
              <Hash className="h-4 w-4 text-accent" />
              <span className="font-mono text-xs">{intent.result.txHash}</span>
            </div>
            <div className="flex items-center gap-2 text-sm text-foreground">
              <Fuel className="h-4 w-4 text-accent" />
              <span>{intent.result.gasUsed} gas</span>
            </div>
            <div className="flex items-center gap-2 text-sm text-foreground">
              <Box className="h-4 w-4 text-accent" />
              <span>Block {intent.result.blockNumber.toLocaleString()}</span>
            </div>
          </div>

          <div className="flex items-center gap-4 text-xs text-muted-foreground">
            <span>
              Delegation: <span className="font-mono text-foreground">{intent.delegationHash}</span>
            </span>
            <span>
              Proof: <span className="font-mono text-foreground">{intent.proofHash}</span>
            </span>
          </div>
        </CardContent>
      </Card>
    </motion.div>
  );
}
