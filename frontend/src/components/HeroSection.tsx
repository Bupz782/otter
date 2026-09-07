import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { ArrowRight } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { MetricsBanner } from "./MetricsBanner";

const EASE: [number, number, number, number] = [0.22, 1, 0.36, 1];

export function HeroSection() {
  const [reducedMotion, setReducedMotion] = useState(false);

  useEffect(() => {
    const media = window.matchMedia("(prefers-reduced-motion: reduce)");
    setReducedMotion(media.matches);
    const onChange = (e: MediaQueryListEvent) => setReducedMotion(e.matches);
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, []);

  return (
    <section className="relative flex min-h-screen flex-col justify-center overflow-hidden px-6 pt-20">
      <div className="mx-auto grid w-full max-w-6xl items-center gap-12 lg:grid-cols-[1.1fr_1fr]">
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease: EASE }}
          className="relative z-10 flex flex-col items-center text-center lg:items-start lg:text-left"
        >
          <h1 className="font-heading text-balance text-5xl font-bold leading-[1.05] tracking-tight text-foreground sm:text-6xl md:text-7xl">
            Set the terms. Otter dives.
          </h1>

          <p className="mt-6 max-w-2xl text-balance text-lg text-muted-foreground md:text-xl">
            Describe a condition, sign a limited delegation, and Otter executes the moment it's
            met, with zero-knowledge proofs. Your keys never leave your hands.
          </p>

          <div className="mt-10 flex flex-col items-center gap-4 sm:flex-row">
            <Button asChild size="lg" className="rounded-full px-8">
              <Link to="/app/dashboard">
                Launch app
                <ArrowRight className="ml-2 h-4 w-4" />
              </Link>
            </Button>
            <Button asChild variant="outline" size="lg" className="rounded-full px-8">
              <a href="#demo">Try an intent</a>
            </Button>
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, scale: 0.96 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 1, delay: 0.15, ease: EASE }}
          className="relative z-10 mx-auto w-full max-w-md lg:max-w-none"
        >
          {/* Faint ember glow echoing the painting's firelight */}
          <div className="pointer-events-none absolute inset-x-8 bottom-0 top-1/3 rounded-full bg-accent/10 blur-[100px]" />
          <motion.img
            src="/otter-hero.webp"
            alt="Otter mascot in dark plate armor"
            animate={reducedMotion ? undefined : { y: [0, -10, 0] }}
            transition={reducedMotion ? undefined : { duration: 7, repeat: Infinity, ease: "easeInOut" }}
            className="relative w-full [mask-image:radial-gradient(closest-side,black_62%,transparent_99%)] [-webkit-mask-image:radial-gradient(closest-side,black_62%,transparent_99%)]"
          />
        </motion.div>
      </div>

      <MetricsBanner />
    </section>
  );
}
