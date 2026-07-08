import { useEffect, useRef, useState } from "react";
import { motion, useInView } from "framer-motion";
import { ShieldCheck, Lock, Clock, Award, Activity } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";

const metrics = [
  { label: "Intents parsed", value: 24800, suffix: "+" },
  { label: "Proofs generated", value: 12400, suffix: "+" },
  { label: "Vault executions", value: 8900, suffix: "+" },
  { label: "Active delegations", value: 3400, suffix: "+" },
];

const badges = [
  { icon: ShieldCheck, label: "Auditable circuits" },
  { icon: Lock, label: "Non-custodial" },
  { icon: Clock, label: "24/7 monitoring" },
  { icon: Award, label: "Open-source verifier" },
];

function AnimatedCounter({
  value,
  suffix,
  label,
}: {
  value: number;
  suffix: string;
  label: string;
}) {
  const ref = useRef<HTMLSpanElement>(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });
  const [count, setCount] = useState(0);

  useEffect(() => {
    if (!isInView) return;
    let start = 0;
    const duration = 1500;
    const startTime = performance.now();

    const animate = (now: number) => {
      const progress = Math.min((now - startTime) / duration, 1);
      const eased = 1 - Math.pow(1 - progress, 3);
      start = Math.floor(eased * value);
      setCount(start);
      if (progress < 1) requestAnimationFrame(animate);
    };

    requestAnimationFrame(animate);
  }, [isInView, value]);

  return (
    <>
      <span
        ref={ref}
        aria-hidden="true"
        className="font-heading text-3xl font-bold text-foreground sm:text-4xl"
      >
        {count.toLocaleString()}
        {suffix}
      </span>
      <span className="sr-only">
        {value.toLocaleString()}
        {suffix} {label}
      </span>
    </>
  );
}

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

      <div className="mb-4 flex items-center justify-center gap-2 text-xs text-muted-foreground">
        <Activity className="h-3 w-3" aria-hidden="true" />
        Simulated metrics
      </div>

      <div className="mb-16 grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
        {metrics.map((metric, index) => (
          <motion.div
            key={metric.label}
            initial={{ opacity: 0, y: 24 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: "-80px" }}
            transition={{
              duration: 0.6,
              delay: index * 0.1,
              ease: [0.22, 1, 0.36, 1],
            }}
          >
            <Card className="border-border/50 bg-card/60 text-center backdrop-blur-sm">
              <CardContent className="p-6">
                <AnimatedCounter {...metric} />
                <p className="mt-2 text-sm text-muted-foreground">{metric.label}</p>
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
