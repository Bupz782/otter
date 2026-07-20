import { useState, useEffect, useRef } from "react";
import { Menu, X } from "lucide-react";
import { Link, useLocation } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export function Navigation() {
  const [scrolled, setScrolled] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);
  const location = useLocation();
  const mobileMenuRef = useRef<HTMLDivElement>(null);
  const toggleRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 16);
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  useEffect(() => {
    if (!mobileOpen) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setMobileOpen(false);
        toggleRef.current?.focus();
      }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [mobileOpen]);

  const isHome = location.pathname === "/";

  const scrollLinks = [
    { label: "Use cases", href: "#use-cases" },
    { label: "Intents", href: "#intents" },
    { label: "Trust", href: "#trust" },
    { label: "Protocol", href: "#protocol" },
  ];

  return (
    <header
      className={cn(
        "fixed top-0 left-0 right-0 z-50 transition-all duration-300",
        scrolled ? "glass-strong" : "bg-transparent"
      )}
    >
      <nav className="mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
        <Link
          to="/"
          className="font-heading text-xl font-bold tracking-tight text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent rounded"
        >
          otter
        </Link>

        <div className="hidden items-center gap-8 md:flex">
          {isHome &&
            scrollLinks.map((link) => (
              <a
                key={link.label}
                href={link.href}
                className="text-sm text-muted-foreground transition-colors duration-200 hover:text-accent focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent rounded px-1"
              >
                {link.label}
              </a>
            ))}
        </div>

        <div className="hidden md:block">
          <Button asChild size="sm" className="rounded-full">
            <Link to="/app/dashboard">Launch app</Link>
          </Button>
        </div>

        <button
          ref={toggleRef}
          type="button"
          className="rounded text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent md:hidden"
          onClick={() => setMobileOpen((open) => !open)}
          aria-label="Toggle menu"
          aria-expanded={mobileOpen}
          aria-controls="mobile-menu"
        >
          {mobileOpen ? <X size={22} /> : <Menu size={22} />}
        </button>
      </nav>

      {mobileOpen && (
        <div
          id="mobile-menu"
          ref={mobileMenuRef}
          className="border-b border-border/50 bg-background/95 backdrop-blur-md md:hidden"
        >
          <div className="flex flex-col gap-4 px-6 py-5">
            {isHome &&
              scrollLinks.map((link) => (
                <a
                  key={link.label}
                  href={link.href}
                  className="text-sm text-muted-foreground transition-colors duration-200 hover:text-accent focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent rounded px-1"
                  onClick={() => setMobileOpen(false)}
                >
                  {link.label}
                </a>
              ))}
            <Button asChild size="sm" className="w-full rounded-full">
              <Link to="/app/dashboard" onClick={() => setMobileOpen(false)}>
                Launch app
              </Link>
            </Button>
          </div>
        </div>
      )}
    </header>
  );
}
