import { motion } from "framer-motion";
import { Check, PenTool, ShieldCheck } from "lucide-react";

const fields = [
  { key: "max_amount", value: "1,000 USDC" },
  { key: "protocols", value: "uniswap-v3, aave-v3" },
  { key: "condition", value: "ETH/USDC < 1,800" },
  { key: "expires", value: "30 days from signature" },
  { key: "target", value: "0x4c1f…9a2e" },
];

const guarantees = [
  "Cap the amount and the protocols the agent may touch",
  "Expires on its own and can be revoked any time",
  "Enforced by the Vault contract, not by trust",
];

export function DelegationCard() {
  return (
    <section className="relative z-10 mx-auto max-w-6xl px-6 py-28">
      <div className="grid items-center gap-12 lg:grid-cols-2">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        >
          <h2 className="font-heading text-3xl font-bold tracking-tight text-foreground sm:text-4xl md:text-5xl">
            Your agent, on a leash.
          </h2>
          <p className="mt-4 max-w-xl text-lg text-muted-foreground">
            Every intent runs inside a delegation you sign. Otter can't move a wei beyond it.
            Anything outside the limits fails on-chain verification.
          </p>
          <ul className="mt-8 space-y-3">
            {guarantees.map((item) => (
              <li key={item} className="flex items-start gap-3 text-sm text-muted-foreground">
                <span className="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full border border-accent/40 bg-accent-subtle">
                  <Check className="h-3 w-3 text-accent" aria-hidden="true" />
                </span>
                {item}
              </li>
            ))}
          </ul>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-80px" }}
          transition={{ duration: 0.6, delay: 0.15, ease: [0.22, 1, 0.36, 1] }}
        >
          <div className="overflow-hidden rounded-2xl border border-border/50 bg-card/60 backdrop-blur-sm">
            <div className="flex items-center justify-between border-b border-border/50 px-6 py-4">
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <PenTool className="h-4 w-4 text-accent" aria-hidden="true" />
                <span className="font-mono">delegation.json</span>
              </div>
              <span className="rounded-full bg-accent-subtle px-3 py-1 text-xs font-medium text-accent">
                Signed by you
              </span>
            </div>
            <dl className="space-y-3 px-6 py-6 font-mono text-sm">
              {fields.map((field) => (
                <div key={field.key} className="flex items-baseline justify-between gap-4">
                  <dt className="shrink-0 text-muted-foreground">{field.key}</dt>
                  <dd className="truncate text-foreground">{field.value}</dd>
                </div>
              ))}
            </dl>
            <div className="flex items-center gap-2 border-t border-border/50 px-6 py-4 text-xs text-muted-foreground">
              <ShieldCheck className="h-4 w-4 text-emerald-400" aria-hidden="true" />
              <span>
                sig <span className="font-mono text-foreground">0x7a3f…9e2c</span>, verified by
                the Vault on every execution
              </span>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
