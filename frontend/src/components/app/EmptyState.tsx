import { motion } from "framer-motion";
import { cn } from "@/lib/utils";

export function EmptyState({
  title,
  description,
  action,
  className,
}: {
  title: string;
  description: string;
  action?: React.ReactNode;
  className?: string;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      className={cn(
        "relative overflow-hidden rounded-xl border border-dashed border-border px-6 py-20 text-center",
        className
      )}
    >
      <img
        src="/otter-empty-scene.webp"
        alt=""
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 h-full w-full object-cover object-[50%_30%] opacity-75"
      />
      <div className="relative flex flex-col items-center justify-center">
        <h3 className="font-heading text-lg font-bold">{title}</h3>
        <p className="mt-1 max-w-xs text-sm text-muted-foreground">{description}</p>
        {action && <div className="mt-5">{action}</div>}
      </div>
    </motion.div>
  );
}
