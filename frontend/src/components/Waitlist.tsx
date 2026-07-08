import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Mail, ArrowRight, CheckCircle2, Sparkles, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { cn } from "@/lib/utils";

const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export function Waitlist() {
  const [email, setEmail] = useState("");
  const [status, setStatus] = useState<"idle" | "loading" | "success" | "error">("idle");
  const errorId = "waitlist-error";
  const successId = "waitlist-success";

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!emailRegex.test(email)) {
      setStatus("error");
      return;
    }
    setStatus("loading");
    // Simulated async submission.
    setTimeout(() => {
      setStatus("success");
      setEmail("");
    }, 1200);
  };

  return (
    <section id="waitlist" className="relative z-10 mx-auto max-w-6xl px-6 py-28">
      <motion.div
        initial={{ opacity: 0, y: 24 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true, margin: "-100px" }}
        transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
      >
        <Card className="overflow-hidden border-border/50 bg-card/60 backdrop-blur-sm">
          <CardContent className="relative p-8 md:p-12">
            <div className="pointer-events-none absolute -right-24 -top-24 h-64 w-64 rounded-full bg-accent/10 blur-3xl" />
            <div className="pointer-events-none absolute -bottom-24 -left-24 h-64 w-64 rounded-full bg-accent/5 blur-3xl" />

            <div className="relative mx-auto max-w-2xl text-center">
              <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-border/60 bg-secondary/50 px-3 py-1 text-xs text-muted-foreground">
                <Sparkles className="h-3 w-3 text-accent" aria-hidden="true" />
                Early access
              </div>
              <h2 className="font-heading text-3xl font-bold tracking-tight text-foreground sm:text-4xl md:text-5xl">
                Get early access
              </h2>
              <p className="mt-4 text-lg text-muted-foreground">
                Join the waitlist to be among the first to delegate intents on mainnet.
              </p>

              <AnimatePresence mode="wait">
                {status === "success" ? (
                  <motion.div
                    key="success"
                    id={successId}
                    initial={{ opacity: 0, scale: 0.96 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0, scale: 0.96 }}
                    transition={{ duration: 0.3, ease: [0.22, 1, 0.36, 1] }}
                    className="mt-8 flex flex-col items-center justify-center gap-3 rounded-xl border border-emerald-500/20 bg-emerald-500/10 p-6"
                    aria-live="polite"
                  >
                    <CheckCircle2 className="h-8 w-8 text-emerald-500" aria-hidden="true" />
                    <p className="text-base font-medium text-foreground">
                      You are on the list.
                    </p>
                    <p className="text-sm text-muted-foreground">
                      We will reach out when early access opens.
                    </p>
                  </motion.div>
                ) : (
                  <motion.form
                    key="form"
                    initial={{ opacity: 0, y: 8 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -8 }}
                    transition={{ duration: 0.3, ease: [0.22, 1, 0.36, 1] }}
                    onSubmit={handleSubmit}
                    className="mt-8 flex flex-col gap-3 sm:flex-row"
                    aria-describedby={status === "error" ? errorId : undefined}
                  >
                    <div className="relative flex-1">
                      <label htmlFor="waitlist-email" className="sr-only">
                        Email address
                      </label>
                      <Mail
                        className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground"
                        aria-hidden="true"
                      />
                      <Input
                        id="waitlist-email"
                        type="email"
                        name="email"
                        autoComplete="email"
                        placeholder="you@example.com"
                        value={email}
                        disabled={status === "loading"}
                        aria-invalid={status === "error"}
                        aria-describedby={status === "error" ? errorId : undefined}
                        onChange={(e) => {
                          setEmail(e.target.value);
                          if (status === "error") setStatus("idle");
                        }}
                        className={cn(
                          "h-12 rounded-full border-border bg-secondary/60 pl-10 pr-4 text-foreground placeholder:text-muted-foreground focus-visible:ring-accent",
                          status === "error" && "border-rose-500 focus-visible:ring-rose-500"
                        )}
                      />
                    </div>
                    <Button
                      type="submit"
                      size="lg"
                      disabled={status === "loading"}
                      className="h-12 rounded-full px-8"
                    >
                      {status === "loading" ? (
                        <>
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" aria-hidden="true" />
                          Joining…
                        </>
                      ) : (
                        <>
                          Join waitlist
                          <ArrowRight className="ml-2 h-4 w-4" aria-hidden="true" />
                        </>
                      )}
                    </Button>
                  </motion.form>
                )}
              </AnimatePresence>

              {status === "error" && (
                <p id={errorId} className="mt-3 text-sm text-rose-500" role="alert">
                  Please enter a valid email address.
                </p>
              )}

              <p className="mt-4 text-xs text-muted-foreground">
                No spam. Unsubscribe at any time.
              </p>
            </div>
          </CardContent>
        </Card>
      </motion.div>
    </section>
  );
}
