import { motion } from "framer-motion";
import { TriangleAlert } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export function ErrorState({
  subject,
  onRetry,
  detail,
  className,
}: {
  subject: string;
  onRetry?: () => void;
  detail?: string;
  className?: string;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      className={cn(
        "flex flex-col items-center justify-center rounded-xl border border-dashed border-border bg-secondary/30 px-6 py-16 text-center",
        className
      )}
    >
      <div className="flex h-14 w-14 items-center justify-center rounded-2xl border border-border bg-secondary text-rose-400">
        <TriangleAlert className="h-6 w-6" />
      </div>
      <h3 className="mt-5 font-heading text-lg font-bold">Couldn't load {subject}</h3>
      <p className="mt-1 max-w-xs text-sm text-muted-foreground">
        {detail ?? "Something went wrong. Check your connection and try again."}
      </p>
      {onRetry && (
        <Button variant="outline" onClick={onRetry} className="mt-5 rounded-full">
          Try again
        </Button>
      )}
    </motion.div>
  );
}
