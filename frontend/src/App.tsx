import { useEffect } from "react";
import { useLocation } from "react-router-dom";
import { AnimatePresence } from "framer-motion";
import { Navigation } from "@/components/Navigation";
import { HeroSection } from "@/components/HeroSection";
import { DemoPreview } from "@/components/DemoPreview";
import { FlowSchema } from "@/components/FlowSchema";
import { UseCases } from "@/components/UseCases";
import { LiveIntents } from "@/components/LiveIntents";
import { TrustSection } from "@/components/TrustSection";
import { ProtocolStack } from "@/components/ProtocolStack";
import { Community } from "@/components/Community";
import { Waitlist } from "@/components/Waitlist";
import { Footer } from "@/components/Footer";
import { AmbientBackground } from "@/components/AmbientBackground";
import { PageTransition } from "@/components/PageTransition";
import { ScrollProgress } from "@/components/ScrollProgress";
import { NoiseOverlay } from "@/components/NoiseOverlay";

export function App() {
  const location = useLocation();

  useEffect(() => {
    const main = document.getElementById("main-content");
    if (main) {
      main.focus({ preventScroll: true });
    }
  }, [location.pathname]);

  return (
    <div className="relative min-h-screen bg-background text-foreground">
      <a
        href="#main-content"
        className="sr-only fixed left-4 top-4 z-[200] rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-foreground focus:not-sr-only focus:outline-none focus:ring-2 focus:ring-ring"
      >
        Skip to main content
      </a>
      <ScrollProgress />
      <AmbientBackground />
      <NoiseOverlay />
      <Navigation />
      <AnimatePresence mode="wait">
        <PageTransition key={location.pathname}>
          <main id="main-content" tabIndex={-1} className="outline-none">
            <HeroSection />
            <DemoPreview />
            <FlowSchema />
            <UseCases />
            <LiveIntents />
            <TrustSection />
            <ProtocolStack />
            <Community />
            <Waitlist />
          </main>
          <Footer />
        </PageTransition>
      </AnimatePresence>
    </div>
  );
}
