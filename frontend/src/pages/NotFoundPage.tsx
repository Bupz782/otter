import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";

export function NotFoundPage() {
  useDocumentTitle("404");

  return (
    <div className="relative min-h-screen overflow-hidden bg-background">
      {/* Full-bleed painted scene: small lost otter lower-right, vast void above */}
      <img
        src="/otter-404-scene.webp"
        alt="A small armored otter lost in the dark, holding a tiny glowing light"
        className="absolute inset-0 z-0 h-full w-full origin-bottom translate-y-[12%] scale-110 object-cover object-bottom"
      />
      <div className="absolute inset-x-0 top-0 z-0 h-72 bg-gradient-to-b from-background/70 to-transparent" />

      <div className="relative z-10 flex min-h-screen flex-col items-center px-6 pt-32 text-center sm:pt-40">
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease: [0.22, 1, 0.36, 1] }}
        >
          <h1 className="font-heading text-4xl font-bold tracking-tight text-foreground sm:text-5xl md:text-6xl">
            Lost in the dark.
          </h1>
          <p className="mx-auto mt-4 max-w-md text-lg text-muted-foreground">
            This page doesn't exist — but the otter found a light. Let's get you back.
          </p>
          <Button asChild size="lg" className="mt-8 rounded-full px-8">
            <Link to="/">Back to safety</Link>
          </Button>
        </motion.div>
      </div>
    </div>
  );
}
