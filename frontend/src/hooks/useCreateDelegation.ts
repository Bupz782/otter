import { useState } from "react";
import { useAccount, useWalletClient } from "wagmi";
import { api } from "@/lib/api";
import {
  buildDelegationMessage,
  generateNonce,
  splitSignatureForBackend,
  type DelegationLimits,
} from "@/lib/delegation";
import type { Delegation, IntentType } from "@/types/app";

export function useCreateDelegation() {
  const { address } = useAccount();
  const { data: walletClient } = useWalletClient();
  const [data, setData] = useState<Delegation | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = async (payload: {
    agentId: string;
    maxAmounts: Record<IntentType, number>;
    allowedProtocols: string[];
    allowedChains: string[];
    expiryDays: number;
  }) => {
    if (!address) throw new Error("Wallet not connected");
    if (!walletClient) throw new Error("Wallet client not available");

    setIsLoading(true);
    setError(null);
    try {
      const pubkey = await api.agents.pubkey();
      const expirySeconds = Math.floor(Date.now() / 1000) + payload.expiryDays * 24 * 60 * 60;
      const nonce = generateNonce();
      // Use wallet address as target contract placeholder for dev/test.
      // In production this should be the token/asset contract the delegation restricts.
      const targetContract = address;

      const limits: DelegationLimits = {
        lend: payload.maxAmounts.lend ?? 0,
        swap: payload.maxAmounts.swap ?? 0,
        withdraw: payload.maxAmounts.withdraw ?? 0,
        claim: payload.maxAmounts.claim ?? 0,
      };

      const message = buildDelegationMessage(
        pubkey.pubkey_x,
        pubkey.pubkey_y,
        limits,
        payload.allowedProtocols,
        expirySeconds,
        nonce,
        targetContract
      );

      const hashResponse = await api.delegations.hash(message);
      const hash = hashResponse.delegation_hash as `0x${string}`;
      const signature = await walletClient.sign({ hash });
      const signatureArray = splitSignatureForBackend(signature);

      const result = await api.delegations.set({ ...message, signature: signatureArray });

      const delegation: Delegation = {
        id: result.delegation_hash,
        userAddress: address,
        agentId: payload.agentId,
        agentName: "Otter Agent",
        maxAmounts: payload.maxAmounts,
        allowedProtocols: payload.allowedProtocols,
        allowedChains: payload.allowedChains,
        expiry: new Date(expirySeconds * 1000).toISOString(),
        createdAt: new Date().toISOString(),
        status: "active",
      };
      setData(delegation);
      return delegation;
    } catch (err) {
      const error = err instanceof Error ? err : new Error("Failed to create delegation");
      setError(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  return { data, isLoading, error, mutate };
}
