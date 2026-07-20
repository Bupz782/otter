import { useState } from "react";

const protocols = [
  "Ethereum",
  "Base",
  "Arbitrum",
  "Uniswap",
  "Aave",
  "Curve",
  "Chainlink",
  "Noir",
];

export function PoweredBy() {
  const [paused, setPaused] = useState(false);

  return (
    <section aria-label="Supported protocols and networks" className="relative z-10 border-y border-border/40 py-10">
      <p className="mb-6 text-center text-xs uppercase tracking-[0.2em] text-muted-foreground">
        Where Otter swims
      </p>
      <div
        className="overflow-hidden"
        onMouseEnter={() => setPaused(true)}
        onMouseLeave={() => setPaused(false)}
        onFocusCapture={() => setPaused(true)}
        onBlurCapture={() => setPaused(false)}
      >
        <div
          className="flex w-max animate-marquee-linear motion-reduce:animate-none"
          style={{ animationPlayState: paused ? "paused" : "running" }}
        >
          {[0, 1].map((copy) => (
            <div key={copy} className="flex shrink-0 items-center" aria-hidden={copy === 1}>
              {protocols.map((name) => (
                <span
                  key={name}
                  className="whitespace-nowrap px-8 font-heading text-lg font-bold text-muted-foreground/50"
                >
                  {name}
                </span>
              ))}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
