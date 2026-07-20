import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Plus } from "lucide-react";
import { cn } from "@/lib/utils";

const faqs = [
  {
    question: "Who holds my funds?",
    answer:
      "You do. Otter is non-custodial: assets stay in your wallet until your condition triggers and the Vault executes, always inside the limits you signed.",
  },
  {
    question: "What can Otter do with my delegation?",
    answer:
      "Only what you sign: a maximum amount, an allowlist of protocols, a target contract, and an expiry. Anything beyond that is rejected by on-chain verification, and you can revoke a delegation at any time.",
  },
  {
    question: "What does the zero-knowledge proof actually guarantee?",
    answer:
      "That the executed action matches your intent and respects your delegation limits. The Vault contract verifies the proof itself. You don't have to trust the agent, the backend, or us.",
  },
  {
    question: "What happens if the agent goes offline?",
    answer:
      "Nothing executes and nothing is at risk. Your funds stay in your wallet, and delegations expire on their own if you don't renew them.",
  },
  {
    question: "Which chains and protocols are supported?",
    answer:
      "Ethereum, Base, and Arbitrum at launch, with Uniswap, Aave, and Curve integrations first. Conditions are fed by Chainlink and on-chain price data.",
  },
  {
    question: "Is Otter audited?",
    answer:
      "The contracts and Noir circuits are built to be auditable and will be open sourced. An external audit is planned before mainnet, and we'll publish the report when it lands.",
  },
];

export function Faq() {
  const [openIndex, setOpenIndex] = useState<number | null>(0);

  return (
    <section id="faq" className="relative z-10 mx-auto max-w-3xl px-6 py-28">
      <div className="mb-12 text-center">
        <motion.h2
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
          className="font-heading text-3xl font-bold tracking-tight text-foreground sm:text-4xl md:text-5xl"
        >
          Before you dive.
        </motion.h2>
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.6, delay: 0.1, ease: [0.22, 1, 0.36, 1] }}
          className="mt-4 text-lg text-muted-foreground"
        >
          Straight answers, no fine print.
        </motion.p>
      </div>

      <motion.div
        initial={{ opacity: 0, y: 24 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true, margin: "-80px" }}
        transition={{ duration: 0.6, delay: 0.15, ease: [0.22, 1, 0.36, 1] }}
        className="divide-y divide-border/60 rounded-2xl border border-border/50 bg-card/60 backdrop-blur-sm"
      >
        {faqs.map((faq, index) => {
          const isOpen = openIndex === index;
          return (
            <div key={faq.question}>
              <button
                type="button"
                onClick={() => setOpenIndex(isOpen ? null : index)}
                aria-expanded={isOpen}
                aria-controls={`faq-panel-${index}`}
                className="flex w-full items-center justify-between gap-4 rounded px-6 py-5 text-left focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent"
              >
                <span className="font-heading text-base font-bold text-foreground">
                  {faq.question}
                </span>
                <Plus
                  className={cn(
                    "h-4 w-4 shrink-0 text-accent transition-transform duration-200",
                    isOpen && "rotate-45"
                  )}
                  aria-hidden="true"
                />
              </button>
              <AnimatePresence initial={false}>
                {isOpen && (
                  <motion.div
                    id={`faq-panel-${index}`}
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: "auto", opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    transition={{ duration: 0.25, ease: [0.22, 1, 0.36, 1] }}
                    className="overflow-hidden"
                  >
                    <p className="px-6 pb-5 text-sm leading-relaxed text-muted-foreground">
                      {faq.answer}
                    </p>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          );
        })}
      </motion.div>
    </section>
  );
}
