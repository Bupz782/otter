import { motion } from "framer-motion";
import { ArrowRight } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { MetricsBanner } from "./MetricsBanner";

const EASE: [number, number, number, number] = [0.22, 1, 0.36, 1];

export function HeroSection() {
  return (
    <section className="relative flex min-h-screen flex-col justify-center overflow-hidden">
      {/* Painted scene, fully visible, floating on the right (desktop only) */}
      <div className="pointer-events-none absolute inset-y-0 right-0 z-0 hidden w-[55%] items-center lg:flex">
        <img
          src="/landing-hero-bg.webp"
          alt=""
          aria-hidden="true"
          className="max-h-[80vh] w-full object-contain object-right [mask-image:radial-gradient(closest-side,black_55%,transparent_98%)] [-webkit-mask-image:radial-gradient(closest-side,black_55%,transparent_98%)]"
        />
      </div>

      <div className="relative z-10 mx-auto w-full max-w-6xl px-6 pt-20">
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease: EASE }}
          className="flex max-w-2xl flex-col items-start text-left"
        >
          <h1 className="font-heading text-balance text-5xl font-bold leading-[1.05] tracking-tight text-foreground sm:text-6xl md:text-7xl lg:text-8xl">
            Set the terms. Otter dives.
          </h1>

          <p className="mt-6 max-w-xl text-balance text-lg text-muted-foreground md:text-xl">
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
      </div>

      <MetricsBanner />
    </section>
  );
}
