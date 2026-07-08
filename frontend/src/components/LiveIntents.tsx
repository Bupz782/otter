import { motion } from "framer-motion";
import { Activity, CheckCircle2, Clock } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

const activeIntents = [
  { id: "1", text: "Buy 1 ETH if < 1,800 USDC", chain: "Ethereum", status: "monitoring" },
  { id: "2", text: "Lend 1,000 USDC if APY > 5%", chain: "Base", status: "monitoring" },
  { id: "3", text: "Swap ARB to USDC if > $1.20", chain: "Arbitrum", status: "pending" },
];

const activityLog = [
  { text: "Bought 0.5 ETH at 1,790 USDC", time: "12s ago" },
  { text: "Lent 2,000 USDC on Aave", time: "45s ago" },
  { text: "Delegation signed for wBTC/USDT", time: "2m ago" },
  { text: "Proof verified on Vault", time: "3m ago" },
];

function StatusIndicator({ status }: { status: string }) {
  const isMonitoring = status === "monitoring";
  return (
    <div className="flex items-center gap-2">
      <span className="relative flex h-2 w-2" aria-hidden="true">
        {isMonitoring && (
          <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500/70 motion-reduce:animate-none" />
        )}
        <span
          className={`relative inline-flex h-2 w-2 rounded-full ${
            isMonitoring ? "bg-emerald-500" : "bg-amber-400"
          }`}
        />
      </span>
      <span className="text-xs font-medium text-muted-foreground capitalize">{status}</span>
      <span className="sr-only">{isMonitoring ? "Monitoring" : "Pending"}</span>
    </div>
  );
}

export function LiveIntents() {
  return (
    <section id="intents" className="relative z-10 mx-auto max-w-6xl px-6 py-28">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true, margin: "-100px" }}
        transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        className="mb-12 max-w-2xl"
      >
        <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-border/60 bg-secondary/50 px-3 py-1 text-xs text-muted-foreground">
          <Activity className="h-3 w-3 text-accent" />
          Simulated activity
        </div>
        <h2 className="font-heading text-3xl font-bold tracking-tight text-foreground sm:text-4xl md:text-5xl">
          Live intents
        </h2>
        <p className="mt-4 text-lg text-muted-foreground">
          Conditions Otter is watching and executing right now.
        </p>
      </motion.div>

      <div className="grid gap-6 lg:grid-cols-5">
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-80px" }}
          transition={{ duration: 0.6, delay: 0.1, ease: [0.22, 1, 0.36, 1] }}
          className="lg:col-span-3"
        >
          <Card className="h-full border-border/50 bg-card/60 backdrop-blur-sm">
            <CardHeader>
              <div className="flex items-center gap-2">
                <Clock className="h-4 w-4 text-muted-foreground" />
                <CardTitle className="text-base font-medium">Active intents</CardTitle>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {activeIntents.map((intent, index) => (
                  <motion.div
                    key={intent.id}
                    initial={{ opacity: 0, x: -12 }}
                    whileInView={{ opacity: 1, x: 0 }}
                    viewport={{ once: true }}
                    transition={{
                      duration: 0.4,
                      delay: index * 0.1,
                      ease: [0.22, 1, 0.36, 1],
                    }}
                    className="group flex flex-col justify-between gap-3 rounded-xl border border-border/40 bg-secondary/30 p-4 transition-colors hover:border-accent/30 hover:bg-secondary/50 sm:flex-row sm:items-center"
                  >
                    <div>
                      <p className="text-sm font-medium text-foreground">{intent.text}</p>
                      <p className="mt-0.5 text-xs text-muted-foreground">{intent.chain}</p>
                    </div>
                    <div className="flex items-center gap-3">
                      <Badge
                        variant="secondary"
                        className="rounded-full bg-accent-subtle text-accent hover:bg-accent/20"
                      >
                        condition
                      </Badge>
                      <StatusIndicator status={intent.status} />
                    </div>
                  </motion.div>
                ))}
              </div>
            </CardContent>
          </Card>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-80px" }}
          transition={{ duration: 0.6, delay: 0.2, ease: [0.22, 1, 0.36, 1] }}
          className="lg:col-span-2"
        >
          <Card className="h-full border-border/50 bg-card/60 backdrop-blur-sm">
            <CardHeader>
              <div className="flex items-center gap-2">
                <CheckCircle2 className="h-4 w-4 text-muted-foreground" />
                <CardTitle className="text-base font-medium">Recent activity</CardTitle>
              </div>
            </CardHeader>
            <CardContent>
              <div className="relative space-y-6 pl-4">
                <div className="absolute top-1 bottom-1 left-6 w-px bg-border" />
                {activityLog.map((item, index) => (
                  <motion.div
                    key={item.text}
                    initial={{ opacity: 0, x: 8 }}
                    whileInView={{ opacity: 1, x: 0 }}
                    viewport={{ once: true }}
                    transition={{
                      duration: 0.4,
                      delay: index * 0.1 + 0.2,
                      ease: [0.22, 1, 0.36, 1],
                    }}
                    className="relative"
                  >
                    <span className="absolute -left-4 top-1 h-2 w-2 rounded-full bg-accent ring-4 ring-background" />
                    <p className="text-sm text-foreground">{item.text}</p>
                    <p className="text-xs text-muted-foreground">{item.time}</p>
                  </motion.div>
                ))}
              </div>
            </CardContent>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}
