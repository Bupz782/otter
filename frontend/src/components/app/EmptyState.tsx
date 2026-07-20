import { motion } from "framer-motion";
import { cn } from "@/lib/utils";

export function EmptyState({
  icon,
  title,
  description,
  action,
  className,
}: {
  icon: React.ReactNode;
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
        "flex flex-col items-center justify-center rounded-xl border border-dashed border-border bg-secondary/30 px-6 py-16 text-center",
        className
      )}
    >
      <div className="relative">
        <div className="flex h-14 w-14 items-center justify-center rounded-2xl border border-border bg-secondary text-accent">
          {icon}
        </div>
        <div className="absolute -right-1 -top-1 h-3 w-3 rounded-full bg-accent" />
      </div>
      <h3 className="mt-5 font-heading text-lg font-bold">{title}</h3>
      <p className="mt-1 max-w-xs text-sm text-muted-foreground">{description}</p>
      {action && <div className="mt-5">{action}</div>}
    </motion.div>
  );
}
