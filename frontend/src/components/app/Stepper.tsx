import { Check } from "lucide-react";
import { cn } from "@/lib/utils";

export function Stepper({
  steps,
  currentStep,
}: {
  steps: { label: string; description: string }[];
  currentStep: number;
}) {
  return (
    <div className="w-full">
      <div role="list" aria-label="Progress" className="flex items-center justify-between">
        {steps.map((step, index) => {
          const isCompleted = index < currentStep;
          const isCurrent = index === currentStep;
          const isUpcoming = index > currentStep;

          return (
            <div key={step.label} role="listitem" className="flex flex-1 items-center">
              <div
                className="flex flex-col items-center"
                aria-current={isCurrent ? "step" : undefined}
              >
                <div
                  className={cn(
                    "flex h-8 w-8 items-center justify-center rounded-full border text-sm font-bold transition-colors",
                    isCompleted && "border-accent bg-accent text-accent-foreground",
                    isCurrent && "border-accent bg-accent-subtle text-accent",
                    isUpcoming && "border-border bg-secondary text-muted-foreground"
                  )}
                >
                  {isCompleted ? <Check className="h-4 w-4" /> : index + 1}
                </div>
                <div className="mt-2 hidden text-center sm:block">
                  <p
                    className={cn(
                      "text-xs font-medium",
                      isCurrent ? "text-foreground" : "text-muted-foreground"
                    )}
                  >
                    {step.label}
                  </p>
                  <p className="hidden max-w-[100px] text-[10px] text-muted-foreground lg:block">
                    {step.description}
                  </p>
                </div>
              </div>
              {index < steps.length - 1 && (
                <div className="mx-2 h-px flex-1 bg-border">
                  <div
                    className={cn(
                      "h-full bg-accent transition-all duration-500",
                      isCompleted ? "w-full" : "w-0"
                    )}
                  />
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
