import { motion } from "framer-motion";
import { TrendingDown, Percent, ShieldAlert, CalendarClock } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

const cases = [
  {
    icon: TrendingDown,
    title: "Conditional buy",
    description:
      "Set a target price and let Otter execute the swap only when the market hits it. No more staring at charts.",
  },
  {
    icon: Percent,
    title: "Yield rebalancing",
    description:
      "Move liquidity automatically when another pool or vault offers a better risk-adjusted APY.",
  },
  {
    icon: ShieldAlert,
    title: "Liquidation guard",
    description:
      "Repay or add collateral before your position becomes unsafe, protected by strict delegation limits.",
  },
  {
    icon: CalendarClock,
    title: "Recurring DCA",
    description:
      "Convert a fixed amount of capital into another asset on a schedule, executed only if gas is reasonable.",
  },
];

export function UseCases() {
  return (
    <section id="use-cases" className="relative z-10 mx-auto max-w-6xl px-6 py-28">
      <div className="mb-16 text-center">
        <motion.h2
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
          className="font-heading text-3xl font-bold tracking-tight text-foreground sm:text-4xl md:text-5xl"
        >
          Built for real DeFi workflows
        </motion.h2>
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.6, delay: 0.1, ease: [0.22, 1, 0.36, 1] }}
          className="mt-4 text-lg text-muted-foreground"
        >
          Concrete intents you can delegate and verify.
        </motion.p>
      </div>

      <div className="grid gap-6 sm:grid-cols-2">
        {cases.map((item, index) => (
          <motion.div
            key={item.title}
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
                  <item.icon className="h-5 w-5 text-foreground" />
                </div>
                <CardTitle className="text-lg">{item.title}</CardTitle>
              </CardHeader>
              <CardContent>
                <CardDescription className="text-base leading-relaxed">
                  {item.description}
                </CardDescription>
              </CardContent>
            </Card>
          </motion.div>
        ))}
      </div>
    </section>
  );
}
