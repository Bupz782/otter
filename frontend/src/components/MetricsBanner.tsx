import { useState } from "react";
import { FileText, ShieldCheck, Zap, Wallet, Activity } from "lucide-react";

const metrics = [
  { label: "Intents parsed", value: "24,800+", icon: FileText },
  { label: "Proofs generated", value: "12,400+", icon: ShieldCheck },
  { label: "Vault executions", value: "8,900+", icon: Zap },
  { label: "Active delegations", value: "3,400+", icon: Wallet },
];

function MetricItem({ metric }: { metric: (typeof metrics)[0] }) {
  return (
    <div className="flex shrink-0 items-center gap-3 px-6 text-foreground/80">
      <metric.icon className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
      <span className="whitespace-nowrap text-sm font-medium">{metric.label}</span>
      <span className="whitespace-nowrap text-sm font-semibold tabular-nums text-foreground">
        {metric.value}
      </span>
    </div>
  );
}

function MetricTrack({ suffix }: { suffix: string }) {
  return (
    <div className="flex shrink-0 items-center">
      {metrics.map((metric, index) => (
        <MetricItem key={`${metric.label}-${suffix}-${index}`} metric={metric} />
      ))}
    </div>
  );
}

export function MetricsBanner() {
  const [paused, setPaused] = useState(false);

  return (
    <div
      className="absolute bottom-0 left-0 right-0 z-10 flex flex-col items-center overflow-hidden border-y border-border/40 bg-background/80 py-3 backdrop-blur-md"
      onMouseEnter={() => setPaused(true)}
      onMouseLeave={() => setPaused(false)}
      onFocusCapture={() => setPaused(true)}
      onBlurCapture={() => setPaused(false)}
    >
      <div className="mb-2 flex items-center gap-2 text-xs text-muted-foreground">
        <Activity className="h-3 w-3" aria-hidden="true" />
        Simulated metrics
      </div>
      <div
        className="flex w-max items-center motion-reduce:animate-none"
        style={{
          animationPlayState: paused ? "paused" : "running",
        }}
      >
        <div
          className="flex shrink-0 items-center animate-marquee-linear motion-reduce:animate-none"
          style={{ animationPlayState: paused ? "paused" : "running" }}
        >
          <MetricTrack suffix="a-0" />
          <MetricTrack suffix="a-1" />
        </div>
        <div
          className="flex shrink-0 items-center animate-marquee-linear motion-reduce:animate-none"
          style={{ animationPlayState: paused ? "paused" : "running" }}
        >
          <MetricTrack suffix="b-0" />
          <MetricTrack suffix="b-1" />
        </div>
      </div>
    </div>
  );
}
