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
      // target_contract constrains which token contract executions may move:
      // the circuit rejects any proposed intent whose asset token address
      // differs (delegation_circuit/src/main.nr step 5b), and the backend
      // derives the proposed target from the intent's asset
      // (crates/application/src/use_cases/execute_intent.rs
      // `asset_to_target_contract`). Zero means unconstrained, which matches
      // this form: it limits amounts and protocols, not a specific asset.
      // The vault address would be wrong here: no intent ever targets it.
      const targetContract = "0x0";

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
      // The circuit expects a secp256k1 ECDSA signature over the raw 32-byte
      // delegation hash (delegation_circuit/src/main.nr step 2). Wallets only
      // expose EIP-191 personal_sign, so sign the raw hash bytes; the backend
      // stores r || s for the proof flow. viem has no WalletClient.sign; the
      // old call only compiled because of loose typing and threw at runtime.
      const signature = await walletClient.signMessage({
        account: address,
        message: { raw: hash },
      });
      const signatureArray = splitSignatureForBackend(signature);

      const result = await api.delegations.set({ ...message, signature: signatureArray });

      // Note: allowedChains is kept on the local object only. The backend
      // message format has no chains field (SetDelegationRequest in
      // crates/interfaces/src/bin/metis_api.rs), so chains are neither signed
      // nor sent. They are not enforced anywhere yet.
      const delegation: Delegation = {
        id: result.delegation_hash,
        userAddress: address,
        agentId: payload.agentId,
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
