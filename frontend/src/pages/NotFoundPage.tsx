import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";

export function NotFoundPage() {
  useDocumentTitle("404");

  return (
    <div className="relative flex min-h-screen flex-col items-center justify-center bg-background px-6 text-center">
      <motion.div
        initial={{ opacity: 0, y: 24 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8, ease: [0.22, 1, 0.36, 1] }}
        className="flex flex-col items-center"
      >
        <img
          src="/otter-404.webp"
          alt="Armored otter puzzling over a small glowing light"
          className="w-72 [mask-image:radial-gradient(closest-side,black_60%,transparent_98%)] [-webkit-mask-image:radial-gradient(closest-side,black_60%,transparent_98%)] md:w-96"
        />
        <h1 className="mt-8 font-heading text-4xl font-bold tracking-tight text-foreground sm:text-5xl">
          Lost in the dark.
        </h1>
        <p className="mt-3 max-w-md text-lg text-muted-foreground">
          This page doesn't exist — but the otter found a light. Let's get you back.
        </p>
        <Button asChild size="lg" className="mt-8 rounded-full px-8">
          <Link to="/">Back to safety</Link>
        </Button>
      </motion.div>
    </div>
  );
}
