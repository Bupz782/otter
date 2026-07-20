import { motion } from "framer-motion";
import { ShieldCheck, Lock, Clock, Award, Wallet, ListChecks, Network } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";

const facts = [
  {
    icon: Wallet,
    title: "0 custody",
    description: "The agent never holds your funds. Assets stay in your wallet until execution.",
  },
  {
    icon: ListChecks,
    title: "5-step pipeline",
    description: "Intent, delegation, monitoring, execution, verification. Every step is checkable.",
  },
  {
    icon: Network,
    title: "3 networks",
    description: "Ethereum, Base, and Arbitrum supported from day one.",
  },
  {
    icon: ShieldCheck,
    title: "ZK-verified execution",
    description: "Noir circuits prove every action respects your delegation limits.",
  },
];

const badges = [
  { icon: ShieldCheck, label: "Auditable circuits" },
  { icon: Lock, label: "Non-custodial" },
  { icon: Clock, label: "24/7 monitoring" },
  { icon: Award, label: "On-chain verifier" },
];

export function TrustSection() {
  return (
    <section id="trust" className="relative z-10 mx-auto max-w-6xl px-6 py-28">
      <div className="mb-16 text-center">
        <motion.h2
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
          className="font-heading text-3xl font-bold tracking-tight text-foreground sm:text-4xl md:text-5xl"
        >
          Trustless by design
        </motion.h2>
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.6, delay: 0.1, ease: [0.22, 1, 0.36, 1] }}
          className="mt-4 text-lg text-muted-foreground"
        >
          Verifiable execution without custody.
        </motion.p>
      </div>

      <div className="mb-16 grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
        {facts.map((fact, index) => (
          <motion.div
            key={fact.title}
            initial={{ opacity: 0, y: 24 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: "-80px" }}
            transition={{
              duration: 0.6,
              delay: index * 0.1,
              ease: [0.22, 1, 0.36, 1],
            }}
          >
            <Card className="h-full border-border/50 bg-card/60 backdrop-blur-sm transition-colors hover:bg-card">
              <CardContent className="p-6">
                <div className="mb-4 flex h-10 w-10 items-center justify-center rounded-lg border border-border bg-secondary">
                  <fact.icon className="h-5 w-5 text-foreground" aria-hidden="true" />
                </div>
                <p className="font-heading text-lg font-bold text-foreground">{fact.title}</p>
                <p className="mt-2 text-sm leading-relaxed text-muted-foreground">
                  {fact.description}
                </p>
              </CardContent>
            </Card>
          </motion.div>
        ))}
      </div>

      <div className="flex flex-wrap items-center justify-center gap-3">
        {badges.map((badge, index) => (
          <motion.div
            key={badge.label}
            initial={{ opacity: 0, scale: 0.95 }}
            whileInView={{ opacity: 1, scale: 1 }}
            viewport={{ once: true, margin: "-80px" }}
            transition={{
              duration: 0.4,
              delay: index * 0.08,
              ease: [0.22, 1, 0.36, 1],
            }}
            className="flex items-center gap-2 rounded-full border border-border/60 bg-secondary/50 px-4 py-2 text-sm text-muted-foreground"
          >
            <badge.icon className="h-4 w-4 text-accent" aria-hidden="true" />
            {badge.label}
          </motion.div>
        ))}
      </div>
    </section>
  );
}
