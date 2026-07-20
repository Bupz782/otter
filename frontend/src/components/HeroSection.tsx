import { useState, useEffect, lazy, Suspense } from "react";
import { motion } from "framer-motion";
import { ArrowRight } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { MetricsBanner } from "./MetricsBanner";

const WebGLSpiral = lazy(() => import("./WebGLSpiral").then((m) => ({ default: m.WebGLSpiral })));

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
    <section className="relative flex min-h-screen flex-col items-center justify-center overflow-hidden px-6 pt-20">
      {!reducedMotion && (
        <div className="pointer-events-none absolute inset-0 z-0">
          <Suspense fallback={null}>
            <WebGLSpiral className="h-full w-full opacity-90" />
          </Suspense>
        </div>
      )}

      <div className="absolute inset-0 z-0 bg-[radial-gradient(circle_at_center,transparent_0%,transparent_50%,#050505_90%)]" />

      <motion.div
        initial={{ opacity: 0, y: 24 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8, ease: [0.22, 1, 0.36, 1] }}
        className="relative z-10 flex max-w-4xl flex-col items-center text-center"
      >
        <h1 className="font-heading text-balance text-5xl font-bold leading-[1.05] tracking-tight text-foreground sm:text-6xl md:text-7xl lg:text-8xl">
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

      <MetricsBanner />
    </section>
  );
}
