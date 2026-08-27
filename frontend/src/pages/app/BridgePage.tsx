import { useCallback, useEffect, useState } from "react";
import { ArrowRightLeft, AlertCircle, Wallet } from "lucide-react";
import { useAccount } from "wagmi";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { useAuthToken } from "@/hooks/useAuthToken";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { EmptyState } from "@/components/app/EmptyState";
import { ErrorState } from "@/components/app/ErrorState";
import { api, type BackendBridgeTransfer, type BackendNetworkStatus } from "@/lib/api";
import { truncateHash } from "@/lib/utils";

const SELECT_CLASS =
  "flex h-10 w-full rounded-md border border-input bg-secondary/60 px-3 py-2 text-sm text-foreground shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50";

function statusVariant(status: string): "default" | "secondary" | "outline" {
  if (status === "minted") return "default";
  if (status === "pending") return "secondary";
  return "outline";
}

export function BridgePage() {
  useDocumentTitle("Bridge");
  const { address, isConnected } = useAccount();
  const { isAuthenticated } = useAuthToken();

  const [networks, setNetworks] = useState<BackendNetworkStatus[]>([]);
  const [network, setNetwork] = useState("");
  const [amountWei, setAmountWei] = useState("");
  const [destinationChainId, setDestinationChainId] = useState("");
  const [locking, setLocking] = useState(false);
  const [lockError, setLockError] = useState<string | null>(null);
  const [lockResult, setLockResult] = useState<{ bridge_id: string; tx_hash: string } | null>(null);

  const [transfers, setTransfers] = useState<BackendBridgeTransfer[] | null>(null);
  const [transfersLoading, setTransfersLoading] = useState(true);
  const [transfersError, setTransfersError] = useState<Error | null>(null);
  const [mintingId, setMintingId] = useState<string | null>(null);
  const [mintError, setMintError] = useState<string | null>(null);

  const fetchTransfers = useCallback(async () => {
    setTransfersLoading(true);
    setTransfersError(null);
    try {
      setTransfers(await api.bridge.transfers());
    } catch (err) {
      setTransfersError(err instanceof Error ? err : new Error("Unknown error"));
    } finally {
      setTransfersLoading(false);
    }
  }, []);

  useEffect(() => {
    api.networks
      .list()
      .then((list) => {
        setNetworks(list);
        if (list.length > 0) setNetwork((prev) => prev || list[0].name);
      })
      .catch(() => setNetworks([]));
  }, []);

  useEffect(() => {
    if (isAuthenticated) {
      fetchTransfers();
    } else {
      setTransfers(null);
      setTransfersLoading(false);
    }
  }, [isAuthenticated, fetchTransfers]);

  const handleLock = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!network || !amountWei || !destinationChainId) return;
    setLocking(true);
    setLockError(null);
    setLockResult(null);
    try {
      const result = await api.bridge.lock({
        network,
        amount_wei: amountWei,
        destination_chain_id: Number(destinationChainId),
      });
      setLockResult(result);
      setAmountWei("");
      fetchTransfers();
    } catch (err) {
      setLockError(err instanceof Error ? err.message : "Lock failed");
    } finally {
      setLocking(false);
    }
  };

  const handleMint = async (transfer: BackendBridgeTransfer) => {
    if (!address) return;
    const destination = networks.find((n) => n.chain_id === transfer.destination_chain_id);
    if (!destination) return;
    setMintingId(transfer.bridge_id);
    setMintError(null);
    try {
      await api.bridge.mint({
        network: destination.name,
        user_address: address,
        amount_wei: transfer.amount_wei,
        bridge_id: transfer.bridge_id,
      });
      fetchTransfers();
    } catch (err) {
      setMintError(err instanceof Error ? err.message : "Mint failed");
    } finally {
      setMintingId(null);
    }
  };

  if (!isAuthenticated) {
    return (
      <div className="mx-auto max-w-6xl space-y-6">
        <PageHeader title="Bridge" subtitle="Cross-chain EVM lock / mint." />
        <SectionCard>
          <EmptyState
            icon={<Wallet className="h-6 w-6" />}
            title="Sign in required"
            description="Connect your wallet and sign in to lock tokens and track bridge transfers."
          />
        </SectionCard>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <PageHeader title="Bridge" subtitle="Cross-chain EVM lock / mint (owner-gated V1)." />

      <SectionCard
        title="Lock tokens"
        subtitle="Lock ERC-20 on the source chain to mint a wrapped representation on the destination."
      >
        {networks.length === 0 ? (
          <EmptyState
            icon={<AlertCircle className="h-6 w-6" />}
            title="No networks configured"
            description="The backend reports no EVM networks. Configure OTTER_NETWORKS with a bridge address."
          />
        ) : (
          <form onSubmit={handleLock} className="space-y-4">
            <div className="grid gap-4 sm:grid-cols-3">
              <div className="space-y-2">
                <Label htmlFor="source-network">Source network</Label>
                <select
                  id="source-network"
                  className={SELECT_CLASS}
                  value={network}
                  onChange={(e) => setNetwork(e.target.value)}
                  required
                >
                  {networks.map((n) => (
                    <option key={n.name} value={n.name}>
                      {n.name} (chain {n.chain_id})
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="amount">Amount (wei)</Label>
                <Input
                  id="amount"
                  type="text"
                  inputMode="numeric"
                  value={amountWei}
                  onChange={(e) => setAmountWei(e.target.value)}
                  placeholder="1000000000000000000"
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="destination-chain">Destination network</Label>
                <select
                  id="destination-chain"
                  className={SELECT_CLASS}
                  value={destinationChainId}
                  onChange={(e) => setDestinationChainId(e.target.value)}
                  required
                >
                  <option value="" disabled>
                    Select a destination
                  </option>
                  {networks
                    .filter((n) => n.name !== network)
                    .map((n) => (
                      <option key={n.name} value={n.chain_id}>
                        {n.name} (chain {n.chain_id})
                      </option>
                    ))}
                </select>
              </div>
            </div>
            {lockError && <p className="text-sm text-destructive">{lockError}</p>}
            {lockResult && (
              <p className="break-all text-sm text-muted-foreground">
                Locked. Bridge id{" "}
                <span className="font-mono">{truncateHash(lockResult.bridge_id)}</span>, tx{" "}
                <span className="font-mono">{truncateHash(lockResult.tx_hash)}</span>.
              </p>
            )}
            <Button type="submit" disabled={locking}>
              <ArrowRightLeft className="mr-2 h-4 w-4" />
              {locking ? "Locking…" : "Lock tokens"}
            </Button>
          </form>
        )}
      </SectionCard>

      <SectionCard title="Your transfers" subtitle="Lock and mint status per bridge transfer.">
        {transfersLoading ? (
          <Skeleton className="h-32 w-full" />
        ) : transfersError ? (
          <ErrorState subject="bridge transfers" onRetry={fetchTransfers} />
        ) : !transfers || transfers.length === 0 ? (
          <EmptyState
            icon={<ArrowRightLeft className="h-6 w-6" />}
            title="No transfers yet"
            description="Lock tokens above to start your first cross-chain transfer."
          />
        ) : (
          <ul className="space-y-3">
            {transfers.map((t) => {
              const destination = networks.find((n) => n.chain_id === t.destination_chain_id);
              const canMint =
                t.status === "pending" && isConnected && !!address && !!destination;
              return (
                <li
                  key={t.bridge_id}
                  className="flex flex-wrap items-center gap-3 rounded-lg border border-border/60 bg-secondary p-4"
                >
                  <div className="min-w-0 flex-1">
                    <p className="font-mono text-sm">{truncateHash(t.bridge_id)}</p>
                    <p className="mt-1 text-xs text-muted-foreground">
                      chain {t.source_chain_id} → {t.destination_chain_id} · {t.amount_wei} wei
                    </p>
                    <p className="mt-1 break-all font-mono text-xs text-muted-foreground">
                      {t.lock_tx_hash && <>lock {truncateHash(t.lock_tx_hash)}</>}
                      {t.mint_tx_hash && <> · mint {truncateHash(t.mint_tx_hash)}</>}
                    </p>
                  </div>
                  <Badge variant={statusVariant(t.status)}>{t.status}</Badge>
                  {t.status === "pending" && (
                    <Button
                      size="sm"
                      variant="outline"
                      disabled={!canMint || mintingId === t.bridge_id}
                      title={
                        canMint
                          ? "Mint the wrapped tokens on the destination chain"
                          : "Connect the wallet that locked, on a configured destination network"
                      }
                      onClick={() => handleMint(t)}
                    >
                      {mintingId === t.bridge_id ? "Minting…" : "Mint"}
                    </Button>
                  )}
                </li>
              );
            })}
          </ul>
        )}
        {mintError && <p className="mt-3 text-sm text-destructive">{mintError}</p>}
        {!transfersLoading && !transfersError && transfers && transfers.length > 0 && (
          <div className="mt-4 flex justify-end">
            <Button onClick={fetchTransfers} variant="outline">
              Refresh
            </Button>
          </div>
        )}
      </SectionCard>
    </div>
  );
}
