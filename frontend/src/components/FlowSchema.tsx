import { motion } from "framer-motion";
import { ArrowRight, FileText, PenTool, Eye, Send, ShieldCheck } from "lucide-react";

const steps = [
  {
    icon: FileText,
    title: "Intent",
    detail: "You describe a conditional action in plain text.",
  },
  {
    icon: PenTool,
    title: "Delegate",
    detail: "You sign a limited authorization: amount, protocols, expiration.",
  },
  {
    icon: Eye,
    title: "Monitor",
    detail: "The agent watches prices, APY, or any on-chain condition.",
  },
  {
    icon: Send,
    title: "Execute",
    detail: "The agent calls the Vault with a ZKP when the condition is met.",
  },
  {
    icon: ShieldCheck,
    title: "Verify",
    detail: "The Vault verifies the proof and executes the transaction.",
  },
];

export function FlowSchema() {
  return (
    <section className="relative z-10 mx-auto max-w-6xl px-6 py-28">
      <div className="mb-16 text-center">
        <motion.h2
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
          className="font-heading text-3xl font-bold tracking-tight text-foreground sm:text-4xl md:text-5xl"
        >
          Trustless execution pipeline
        </motion.h2>
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.6, delay: 0.1, ease: [0.22, 1, 0.36, 1] }}
          className="mx-auto mt-4 max-w-2xl text-lg text-muted-foreground"
        >
          From intent to on-chain execution. No custody, no blind trust.
        </motion.p>
      </div>

      <div className="relative">
        <div className="absolute top-1/2 left-0 right-0 hidden h-px bg-border lg:block" />

        <ol className="grid list-none gap-6 md:grid-cols-2 lg:grid-cols-5">
          {steps.map((step, index) => (
            <motion.li
              key={step.title}
              initial={{ opacity: 0, y: 24 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: "-80px" }}
              transition={{
                duration: 0.6,
                delay: index * 0.1,
                ease: [0.22, 1, 0.36, 1],
              }}
              className="group relative"
            >
              <div className="relative z-10 flex flex-col gap-4 rounded-xl border border-border/50 bg-card/60 p-5 backdrop-blur-sm transition-colors hover:bg-card">
                <div className="flex h-10 w-10 items-center justify-center rounded-lg border border-border bg-secondary">
                  <step.icon className="h-5 w-5 text-foreground" aria-hidden="true" />
                </div>
                <div>
                  <h3 className="font-heading text-base font-bold text-foreground">{step.title}</h3>
                  <p className="mt-1 text-sm leading-relaxed text-muted-foreground">
                    {step.detail}
                  </p>
                </div>
              </div>

              {index < steps.length - 1 && (
                <div className="absolute top-1/2 -right-3 z-20 hidden h-6 w-6 -translate-y-1/2 items-center justify-center rounded-full border border-border bg-secondary text-muted-foreground lg:flex">
                  <ArrowRight className="h-3 w-3" aria-hidden="true" />
                </div>
              )}
            </motion.li>
          ))}
        </ol>
      </div>
    </section>
  );
}
