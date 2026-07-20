import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface DataRowProps {
  children: ReactNode;
  className?: string;
}

export function DataRow({ children, className }: DataRowProps) {
  return (
    <div
      className={cn(
        "flex items-center gap-4 rounded-xl border border-border/40 bg-secondary/30 px-4 py-3 transition-colors hover:border-accent/30 hover:bg-secondary/50",
        className
      )}
    >
      {children}
    </div>
  );
}
