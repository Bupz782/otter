import { Separator } from "@/components/ui/separator";

const links = [
  { label: "Use cases", href: "#use-cases" },
  { label: "Trust", href: "#trust" },
  { label: "Protocol", href: "#protocol" },
  { label: "Waitlist", href: "#waitlist" },
];

export function Footer() {
  return (
    <footer className="relative z-10 border-t border-border/50 bg-background">
      <div className="mx-auto max-w-6xl px-6 py-12">
        <div className="flex flex-col items-start justify-between gap-8 md:flex-row md:items-center">
          <div>
            <p className="font-heading text-xl font-bold tracking-tight text-foreground">otter</p>
            <p className="mt-2 text-sm text-muted-foreground">
              Trustless DeFi intents. Conditional execution with zero-knowledge proofs.
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-6">
            {links.map((link) => (
              <a
                key={link.label}
                href={link.href}
                className="text-sm text-muted-foreground transition-colors duration-200 hover:text-accent focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent rounded px-1"
              >
                {link.label}
              </a>
            ))}
          </div>
        </div>

        <Separator className="my-8 bg-border/50" />

        <p className="text-xs text-muted-foreground">
          © {new Date().getFullYear()} Otter. All rights reserved.
        </p>
      </div>
    </footer>
  );
}
