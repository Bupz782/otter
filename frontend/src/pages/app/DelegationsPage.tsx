import { Link } from "react-router-dom";
import { Plus, FileSignature, ShieldCheck } from "lucide-react";
import { motion } from "framer-motion";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { useDelegations } from "@/hooks/useDelegations";

export function DelegationsPage() {
  const { data: delegations, isLoading } = useDelegations();

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
      >
        <div>
          <h1 className="font-heading text-3xl font-bold tracking-tight">Delegations</h1>
          <p className="text-muted-foreground">
            Agents you have authorized to execute intents on your behalf.
          </p>
        </div>
        <Button asChild className="rounded-full">
          <Link to="/app/delegations/new">
            <Plus className="mr-2 h-4 w-4" />
            New delegation
          </Link>
        </Button>
      </motion.div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ShieldCheck className="h-5 w-5 text-accent" />
            Active delegations
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {isLoading ? (
            <div className="space-y-3">
              <Skeleton className="h-32 w-full" />
              <Skeleton className="h-32 w-full" />
            </div>
          ) : delegations?.length === 0 ? (
            <div className="rounded-xl border border-dashed border-border py-16 text-center">
              <FileSignature className="mx-auto h-10 w-10 text-muted-foreground" />
              <p className="mt-4 text-muted-foreground">No delegations yet.</p>
              <Button asChild className="mt-4 rounded-full">
                <Link to="/app/delegations/new">Create your first delegation</Link>
              </Button>
            </div>
          ) : (
            delegations?.map((delegation) => (
              <div
                key={delegation.id}
                className="rounded-xl border border-border/60 bg-card p-5 transition-colors hover:border-accent/40"
              >
                <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                  <div>
                    <div className="flex items-center gap-3">
                      <p className="font-heading text-lg font-bold">{delegation.agentName}</p>
                      <Badge variant={delegation.status === "active" ? "default" : "outline"}>
                        {delegation.status}
                      </Badge>
                    </div>
                    <p className="mt-1 text-sm text-muted-foreground">
                      Expires {new Date(delegation.expiry).toLocaleDateString()}
                    </p>
                  </div>
                  <Button variant="outline" size="sm" disabled={delegation.status !== "active"}>
                    Revoke
                  </Button>
                </div>

                <div className="mt-4 grid gap-3 border-t border-border/50 pt-4 sm:grid-cols-3">
                  <div>
                    <p className="text-xs text-muted-foreground">Allowed protocols</p>
                    <p className="text-sm font-medium">{delegation.allowedProtocols.join(", ")}</p>
                  </div>
                  <div>
                    <p className="text-xs text-muted-foreground">Allowed chains</p>
                    <p className="text-sm font-medium">{delegation.allowedChains.join(", ")}</p>
                  </div>
                  <div>
                    <p className="text-xs text-muted-foreground">Max amounts</p>
                    <p className="text-sm font-medium">
                      Lend ${delegation.maxAmounts.lend.toLocaleString()}
                    </p>
                  </div>
                </div>
              </div>
            ))
          )}
        </CardContent>
      </Card>
    </div>
  );
}
