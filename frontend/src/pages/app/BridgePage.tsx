import { useState } from "react";
import { ArrowRightLeft, AlertCircle } from "lucide-react";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { EmptyState } from "@/components/app/EmptyState";

export function BridgePage() {
  useDocumentTitle("Bridge");
  const [amount, setAmount] = useState("");
  const [destinationChainId, setDestinationChainId] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // Cross-chain bridge execution is a backend V2 follow-up; the contract
    // layer (OtterBridge + BridgeToken) is already in place.
    alert("Bridge backend integration is not wired yet.");
  };

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <PageHeader title="Bridge" subtitle="Cross-chain EVM lock / mint (V1 contracts deployed)." />

      <SectionCard title="Lock tokens" subtitle="Lock ERC-20 on the source chain to mint a wrapped representation on the destination.">
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="amount">Amount</Label>
              <Input
                id="amount"
                type="number"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="0.0"
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="destination-chain">Destination chain id</Label>
              <Input
                id="destination-chain"
                type="number"
                value={destinationChainId}
                onChange={(e) => setDestinationChainId(e.target.value)}
                placeholder="e.g. 421614"
                required
              />
            </div>
          </div>
          <Button type="submit">
            <ArrowRightLeft className="mr-2 h-4 w-4" />
            Preview lock
          </Button>
        </form>
      </SectionCard>

      <SectionCard>
        <EmptyState
          icon={<AlertCircle className="h-6 w-6" />}
          title="Bridge execution pending"
          description="The smart-contract layer is ready. End-to-end bridge execution via the API is the next backend follow-up."
        />
      </SectionCard>
    </div>
  );
}
