import { motion } from "framer-motion";
import { FileCode, Server, ShieldCheck, Activity } from "lucide-react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

const technologies = [
  {
    icon: FileCode,
    title: "Smart contracts",
    description:
      "Vault and Verifier contracts handle proof verification and on-chain execution. The agent never holds user funds.",
  },
  {
    icon: Server,
    title: "Agent backend",
    description:
      "Rust service built with Alloy and an event-driven architecture. Monitors conditions, generates proofs, and submits transactions.",
  },
  {
    icon: ShieldCheck,
    title: "Noir circuits",
    description:
      "Zero-knowledge circuits prove the intent respects the delegation limits: amount, protocols, expiration, and target contract.",
  },
  {
    icon: Activity,
    title: "Oracle feeds",
    description:
      "Price and APY data from Chainlink and Uniswap feeds trigger execution only when the user's condition is met.",
  },
];

export function ProtocolStack() {
  return (
    <section id="protocol" className="relative z-10 mx-auto max-w-6xl px-6 py-28">
      <div className="mb-16 text-center">
        <motion.h2
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
          className="font-heading text-3xl font-bold tracking-tight text-foreground sm:text-4xl md:text-5xl"
        >
          Protocol stack
        </motion.h2>
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.6, delay: 0.1, ease: [0.22, 1, 0.36, 1] }}
          className="mx-auto mt-4 max-w-2xl text-lg text-muted-foreground"
        >
          Built to execute intents without custody or blind trust.
        </motion.p>
      </div>

      <div className="grid gap-6 sm:grid-cols-2">
        {technologies.map((tech, index) => (
          <motion.div
            key={tech.title}
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
              <CardHeader>
                <div className="mb-4 flex h-10 w-10 items-center justify-center rounded-lg border border-border bg-secondary">
                  <tech.icon className="h-5 w-5 text-foreground" aria-hidden="true" />
                </div>
                <CardTitle className="text-lg">{tech.title}</CardTitle>
              </CardHeader>
              <CardContent>
                <CardDescription className="text-base leading-relaxed">
                  {tech.description}
                </CardDescription>
              </CardContent>
            </Card>
          </motion.div>
        ))}
      </div>
    </section>
  );
}
