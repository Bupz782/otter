import { useEffect, useState } from "react";
import { Search, Send, CheckCircle2, XCircle } from "lucide-react";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { EmptyState } from "@/components/app/EmptyState";
import { ErrorState } from "@/components/app/ErrorState";
import { api, ApiClientError } from "@/lib/api";

export function SolanaPage() {
  useDocumentTitle("Solana");
  const [payloadHash, setPayloadHash] = useState("");
  const [authority, setAuthority] = useState("");
  const [verifyHash, setVerifyHash] = useState("");
  const [signature, setSignature] = useState<string | null>(null);
  const [attestation, setAttestation] = useState<{ payload_hash: string; timestamp: number } | null>(null);
  const [verifyResult, setVerifyResult] = useState<boolean | null>(null);
  const [solvencyRoot, setSolvencyRoot] = useState<string | null>(null);
  const [nowSignature, setNowSignature] = useState<string | null>(null);
  const [loading, setLoading] = useState<"attest" | "get" | "verify" | "now" | null>(null);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    api.solvency
      .status()
      .then((s) => setSolvencyRoot(s.merkle_root ?? null))
      .catch(() => setSolvencyRoot(null));
  }, []);

  const clear = () => {
    setError(null);
    setSignature(null);
    setAttestation(null);
    setVerifyResult(null);
    setNowSignature(null);
  };

  const handleAttestNow = async () => {
    clear();
    if (!solvencyRoot) return;
    setLoading("now");
    try {
      const res = await api.solana.attest(solvencyRoot);
      setNowSignature(res.signature);
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Attest failed"));
    } finally {
      setLoading(null);
    }
  };

  const handleAttest = async (e: React.FormEvent) => {
    e.preventDefault();
    clear();
    setLoading("attest");
    try {
      const res = await api.solana.attest(payloadHash);
      setSignature(res.signature);
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Attest failed"));
    } finally {
      setLoading(null);
    }
  };

  const handleGet = async (e: React.FormEvent) => {
    e.preventDefault();
    clear();
    setLoading("get");
    try {
      const res = await api.solana.get(authority);
      setAttestation(res);
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Fetch failed"));
    } finally {
      setLoading(null);
    }
  };

  const handleVerify = async (e: React.FormEvent) => {
    e.preventDefault();
    clear();
    setLoading("verify");
    try {
      const res = await api.solana.verify(authority, verifyHash);
      setVerifyResult(res.valid);
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Verify failed"));
    } finally {
      setLoading(null);
    }
  };

  const isUnavailable = error && error instanceof ApiClientError && error.status === 503;

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <PageHeader title="Solana attestations" subtitle="Anchor on-chain hash attestations." />

      {isUnavailable ? (
        <SectionCard>
          <EmptyState
            icon={<XCircle className="h-6 w-6" />}
            title="Solana adapter not configured"
            description="Set OTTER_SOLANA_ENABLED and related env vars on the backend to use this page."
          />
        </SectionCard>
      ) : (
        <>
          <SectionCard title="Attest" subtitle="Store a 32-byte payload hash on-chain.">
            <form onSubmit={handleAttest} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="payload-hash">Payload hash (0x…)</Label>
                <Input
                  id="payload-hash"
                  value={payloadHash}
                  onChange={(e) => setPayloadHash(e.target.value)}
                  placeholder="0x0000000000000000000000000000000000000000000000000000000000000000"
                  required
                />
              </div>
              <Button type="submit" disabled={loading === "attest"}>
                <Send className="mr-2 h-4 w-4" />
                {loading === "attest" ? "Submitting…" : "Attest"}
              </Button>
              {signature && (
                <p className="break-all text-sm text-muted-foreground">
                  Signature: <span className="font-mono">{signature}</span>
                </p>
              )}
            </form>
          </SectionCard>

          <SectionCard
            title="Solvency attestation"
            subtitle="The scheduler periodically anchors the current solvency Merkle root on-chain."
          >
            <div className="space-y-4">
              <p className="break-all text-sm">
                Current root:{" "}
                {solvencyRoot ? (
                  <span className="font-mono">{solvencyRoot}</span>
                ) : (
                  <span className="text-muted-foreground">
                    unavailable (registry not configured or not proven yet)
                  </span>
                )}
              </p>
              <Button onClick={handleAttestNow} disabled={loading === "now" || !solvencyRoot}>
                <Send className="mr-2 h-4 w-4" />
                {loading === "now" ? "Submitting…" : "Attest now"}
              </Button>
              {nowSignature && (
                <p className="break-all text-sm text-muted-foreground">
                  Signature: <span className="font-mono">{nowSignature}</span>
                </p>
              )}
            </div>
          </SectionCard>

          <SectionCard title="Read" subtitle="Fetch the attestation for an authority.">
            <form onSubmit={handleGet} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="authority">Authority address</Label>
                <Input
                  id="authority"
                  value={authority}
                  onChange={(e) => setAuthority(e.target.value)}
                  placeholder="11111111111111111111111111111111"
                  required
                />
              </div>
              <Button type="submit" variant="outline" disabled={loading === "get"}>
                <Search className="mr-2 h-4 w-4" />
                {loading === "get" ? "Fetching…" : "Fetch"}
              </Button>
              {attestation && (
                <div className="space-y-1 text-sm">
                  <p className="break-all font-mono">Hash: {attestation.payload_hash}</p>
                  <p>
                    Timestamp:{" "}
                    {new Date(attestation.timestamp * 1000).toLocaleString()}
                  </p>
                </div>
              )}
            </form>
          </SectionCard>

          <SectionCard title="Verify" subtitle="Check whether an authority attested to a hash.">
            <form onSubmit={handleVerify} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="verify-hash">Payload hash (0x…)</Label>
                <Input
                  id="verify-hash"
                  value={verifyHash}
                  onChange={(e) => setVerifyHash(e.target.value)}
                  placeholder="0x…"
                  required
                />
              </div>
              <Button type="submit" variant="outline" disabled={loading === "verify"}>
                {loading === "verify" ? "Verifying…" : "Verify"}
              </Button>
              {verifyResult !== null && (
                <div className="flex items-center gap-2 text-sm font-medium">
                  {verifyResult ? (
                    <>
                      <CheckCircle2 className="h-4 w-4 text-emerald-400" /> Valid attestation
                    </>
                  ) : (
                    <>
                      <XCircle className="h-4 w-4 text-rose-400" /> Hash mismatch
                    </>
                  )}
                </div>
              )}
            </form>
          </SectionCard>

          {error && !isUnavailable && <ErrorState subject="solana attestation" onRetry={() => setError(null)} />}
        </>
      )}
    </div>
  );
}
