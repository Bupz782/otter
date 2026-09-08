import type { CreateStrategyPayload, Delegation, Intent, ParsedIntent, Strategy } from "@/types/app";

const API_BASE = import.meta.env.VITE_API_URL || "";

let authToken: string | null = null;
let refreshToken: string | null = null;

const AUTH_TOKEN_KEY = "otter_token";
const REFRESH_TOKEN_KEY = "otter_refresh_token";

// Fired on window after setAuthToken so hooks and components in this tab
// react to sign-in/sign-out. Cross-tab sync uses the native storage event.
export const AUTH_TOKEN_CHANGED_EVENT = "otter-auth-changed";

export function setAuthTokens(accessToken: string | null, refreshTokenValue: string | null) {
  authToken = accessToken;
  refreshToken = refreshTokenValue;
  if (accessToken) {
    localStorage.setItem(AUTH_TOKEN_KEY, accessToken);
  } else {
    localStorage.removeItem(AUTH_TOKEN_KEY);
  }
  if (refreshTokenValue) {
    localStorage.setItem(REFRESH_TOKEN_KEY, refreshTokenValue);
  } else {
    localStorage.removeItem(REFRESH_TOKEN_KEY);
  }
  if (typeof window !== "undefined") {
    window.dispatchEvent(new Event(AUTH_TOKEN_CHANGED_EVENT));
  }
}

export function loadAuthToken(): string | null {
  if (authToken) return authToken;
  return localStorage.getItem(AUTH_TOKEN_KEY);
}

export function loadRefreshToken(): string | null {
  if (refreshToken) return refreshToken;
  return localStorage.getItem(REFRESH_TOKEN_KEY);
}

// Auth bypass: set when the API reports auth_enabled=false (local demo
// default) and a wallet is connected — no SIWE round-trip exists in that
// mode, the middleware accepts requests without a token.
const AUTH_BYPASS_KEY = "otter_auth_bypass";

export function setAuthBypass(active: boolean) {
  if (active) {
    localStorage.setItem(AUTH_BYPASS_KEY, "1");
  } else {
    localStorage.removeItem(AUTH_BYPASS_KEY);
  }
  if (typeof window !== "undefined") {
    window.dispatchEvent(new Event(AUTH_TOKEN_CHANGED_EVENT));
  }
}

export function loadAuthBypass(): boolean {
  return localStorage.getItem(AUTH_BYPASS_KEY) === "1";
}

export function getAuthToken(): string | null {
  return authToken;
}

export class ApiClientError extends Error {
  status: number;
  constructor(message: string, status: number) {
    super(message);
    this.status = status;
  }
}

async function request<T>(path: string, options: RequestInit = {}, retry = true): Promise<T> {
  const url = `${API_BASE}${path}`;
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options.headers as Record<string, string>),
  };
  const token = loadAuthToken();
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const response = await fetch(url, { ...options, headers });
  if (response.status === 204) {
    return undefined as T;
  }

  // Try to refresh on 401 if we have a refresh token and haven't retried yet.
  if (response.status === 401 && retry) {
    const storedRefresh = loadRefreshToken();
    if (storedRefresh) {
      try {
        const refreshed = await request<{ access_token: string }>(
          "/api/v1/auth/refresh",
          {
            method: "POST",
            body: JSON.stringify({ refresh_token: storedRefresh }),
          },
          false
        );
        setAuthTokens(refreshed.access_token, storedRefresh);
        return request<T>(path, options, false);
      } catch {
        setAuthTokens(null, null);
      }
    }
  }

  let body: unknown;
  try {
    body = await response.json();
  } catch {
    body = {};
  }

  if (!response.ok) {
    const message = (body as { error?: string }).error || response.statusText;
    throw new ApiClientError(message, response.status);
  }

  return body as T;
}

// Backend shape of ConditionalIntent returned by /api/v1/intents/parse and /api/v1/intents/:id.
export interface BackendConditionalIntent {
  intent: BackendIntentVariant;
  condition: BackendCondition | null;
}

export type BackendIntentVariant =
  | { Lend: { asset: BackendAsset; amount: string; protocol: BackendLendingProtocol } }
  | {
      Swap: {
        from_asset: BackendAsset;
        to_asset: BackendAsset;
        amount: string;
        protocol: BackendDexProtocol;
      };
    }
  | { Stake: { asset: BackendAsset; amount: string; protocol: BackendLendingProtocol } }
  | {
      Borrow: {
        asset: BackendAsset;
        amount: string;
        collateral: BackendAsset;
        collateral_amount: string;
        protocol: BackendLendingProtocol;
      };
    }
  | { Composite: { intents: BackendIntentVariant[] } };

export type BackendAsset = "Eth" | "Dai" | "Usdc" | "Wbtc" | "Link" | "Sol";
export type BackendDexProtocol = "Uniswap" | "Sushiswap" | "Balancer";
export type BackendLendingProtocol = "Aave" | "Compound";

export interface BackendCondition {
  Comparison: {
    metric: "Yield" | "Price" | "GasCost" | "Volume";
    comparator:
      "GreaterThan" | "LessThan" | "EqualTo" | "LessThanOrEqualTo" | "GreaterThanOrEqualTo";
    value: string;
  };
}

export interface BackendIntentRecord {
  id: string;
  text: string;
  intent: BackendConditionalIntent;
  state: string;
  created_at: number;
  updated_at: number;
  delegation_id?: string | null;
}

/** One lifecycle step of an intent (GET /api/v1/intents/:id/events). */
export interface BackendIntentEvent {
  kind: "parsed" | "condition_met" | "proof_started" | "proof_generated" | "submitted" | "confirmed" | "failed";
  detail: string | null;
  at: number;
}

export interface BackendExecutionRecord {
  id: string;
  intent_id: string;
  tx_hash: string;
  status: string;
  gas_used: number;
  created_at: number;
}

export interface BackendDelegationRecord {
  hash: string;
  payload_json: string;
  signature: string;
  created_at: number;
}

export interface BackendAgentSummary {
  id: string;
  name: string;
  operated_by: string;
  // Present (true) while the API serves built-in demonstration data (A2).
  demo?: boolean;
  bond: number;
  proofs_submitted: number;
  yield_generated: number;
  mev_captured: number;
  uptime: number;
  description: string;
}

export interface BackendAgentPubkeyResponse {
  pubkey_x: string;
  pubkey_y: string;
}

export interface BackendStrategySummary {
  id: string;
  agent_id: string;
  agent_name: string;
  title: string;
  description: string;
  raw_text: string;
  risk_profile: string;
  copies: number;
  total_volume: number;
  apy: number;
  created_at: number;
  updated_at: number;
}

export interface BackendStrategyDetail extends BackendStrategySummary {
  intent: BackendConditionalIntent;
  creator_address: string | null;
  updated_at: number;
}

export interface BackendPositionSummary {
  asset: string;
  protocol: string;
  chain: string;
  amount: string;
  value: number;
  apy: number;
}

export interface BackendPortfolioResponse {
  address: string;
  total_balance: string;
  allocated: string;
  available: string;
  yield_earned: number;
  mev_rebates: number;
  positions: BackendPositionSummary[];
}

export interface BackendProofSummary {
  id: string;
  proof_type: string;
  intent_id: string | null;
  verifier: string;
  constraints: number;
  proof_time: number;
  timestamp: number;
  verified: boolean;
  tx_hash: string | null;
}

export interface ChallengeResponse {
  message: string;
}

export interface BackendNetworkStatus {
  name: string;
  chain_id: number;
  vault_address: string;
  healthy: boolean;
}

export interface BackendBridgeTransfer {
  bridge_id: string;
  source_chain_id: number;
  destination_chain_id: number;
  amount_wei: string;
  lock_tx_hash: string | null;
  mint_tx_hash: string | null;
  status: string;
  created_at: number;
  updated_at: number;
}

export interface BackendMevBundle {
  bundle_hash: string;
  target_tx_hash: string | null;
  status: string;
  created_at: number;
}

export interface VerifyResponse {
  access_token: string;
  refresh_token: string;
}

export interface ParsedIntentResponse {
  intent: BackendConditionalIntent;
}

function assetSymbol(asset: BackendAsset): string {
  switch (asset) {
    case "Eth":
      return "ETH";
    case "Dai":
      return "DAI";
    case "Usdc":
      return "USDC";
    case "Wbtc":
      return "WBTC";
    case "Link":
      return "LINK";
    case "Sol":
      return "SOL";
  }
}

function dexProtocolName(protocol: BackendDexProtocol): string {
  return protocol;
}

function lendingProtocolName(protocol: BackendLendingProtocol): string {
  return protocol;
}

function protocolName(intent: BackendIntentVariant): string {
  if ("Lend" in intent) return lendingProtocolName(intent.Lend.protocol);
  if ("Stake" in intent) return lendingProtocolName(intent.Stake.protocol);
  if ("Borrow" in intent) return lendingProtocolName(intent.Borrow.protocol);
  if ("Swap" in intent) return dexProtocolName(intent.Swap.protocol);
  return "Otter";
}

function formatCondition(condition: BackendCondition | null): string | undefined {
  if (!condition) return undefined;
  const c = condition.Comparison;
  const comparatorMap: Record<string, string> = {
    GreaterThan: ">",
    LessThan: "<",
    EqualTo: "=",
    LessThanOrEqualTo: "<=",
    GreaterThanOrEqualTo: ">=",
  };
  return `${c.metric} ${comparatorMap[c.comparator] || c.comparator} ${c.value}`;
}

function assetDecimals(asset: BackendAsset): number {
  switch (asset) {
    case "Usdc":
      return 6;
    case "Wbtc":
      return 8;
    case "Sol":
      return 9;
    default:
      return 18; // Eth, Dai, Link
  }
}

/** The backend carries amounts in base units; the UI works in human units. */
function humanAmount(amount: string, asset: BackendAsset): number {
  return Number(amount) / 10 ** assetDecimals(asset);
}

export function mapBackendConditionalIntent(conditional: BackendConditionalIntent): ParsedIntent {
  const condition = formatCondition(conditional.condition);
  const inner = conditional.intent;

  // The backend payload carries no chain; leave it undefined instead of guessing.
  if ("Lend" in inner) {
    return {
      type: "lend",
      amount: humanAmount(inner.Lend.amount, inner.Lend.asset),
      asset: assetSymbol(inner.Lend.asset),
      protocol: lendingProtocolName(inner.Lend.protocol),
      condition,
    };
  }

  if ("Stake" in inner) {
    return {
      type: "lend",
      amount: humanAmount(inner.Stake.amount, inner.Stake.asset),
      asset: assetSymbol(inner.Stake.asset),
      protocol: lendingProtocolName(inner.Stake.protocol),
      condition,
    };
  }

  if ("Borrow" in inner) {
    return {
      type: "withdraw",
      amount: humanAmount(inner.Borrow.amount, inner.Borrow.asset),
      asset: assetSymbol(inner.Borrow.asset),
      protocol: lendingProtocolName(inner.Borrow.protocol),
      condition,
    };
  }

  if ("Swap" in inner) {
    return {
      type: "swap",
      amount: humanAmount(inner.Swap.amount, inner.Swap.from_asset),
      asset: assetSymbol(inner.Swap.from_asset),
      protocol: dexProtocolName(inner.Swap.protocol),
      condition,
    };
  }

  // Composite fallback.
  return {
    type: "lend",
    amount: 0,
    asset: "USDC",
    protocol: protocolName(inner),
    condition,
  };
}

function mapBackendStatus(state: string): import("@/types/app").IntentStatus {
  if (state === "active") return "monitoring";
  if (state.startsWith("submitted")) return "submitted";
  if (state.startsWith("executed")) return "confirmed";
  if (state.startsWith("failed")) return "failed";
  if (state === "cancelled") return "revoked";
  return "monitoring";
}

export function mapBackendIntent(record: BackendIntentRecord): Intent {
  const status = mapBackendStatus(record.state);
  const txHash = record.state.startsWith("executed:")
    ? record.state.slice("executed:".length)
    : record.state.startsWith("submitted:")
      ? record.state.slice("submitted:".length)
      : undefined;

  return {
    id: record.id,
    userAddress: "",
    rawText: record.text,
    parsed: mapBackendConditionalIntent(record.intent),
    status,
    createdAt: new Date(record.created_at * 1000).toISOString(),
    delegationId: record.delegation_id ?? "",
    executedAt:
      status === "confirmed" ? new Date(record.updated_at * 1000).toISOString() : undefined,
    txHash,
  };
}

export function mapBackendDelegation(record: BackendDelegationRecord): Delegation {
  return {
    id: record.hash,
    createdAt: new Date(record.created_at * 1000).toISOString(),
  };
}

export function mapBackendAgent(agent: BackendAgentSummary): Agent {
  return {
    id: agent.id,
    name: agent.name,
    operatedBy: agent.operated_by,
    bond: agent.bond,
    proofsSubmitted: agent.proofs_submitted,
    yieldGenerated: agent.yield_generated,
    mevCaptured: agent.mev_captured,
    uptime: agent.uptime,
    description: agent.description,
  };
}

export function mapBackendStrategy(strategy: BackendStrategySummary): Strategy {
  return {
    id: strategy.id,
    title: strategy.title,
    description: strategy.description,
    rawText: strategy.raw_text,
    copies: strategy.copies,
    totalVolume: strategy.total_volume,
    apy: strategy.apy,
    createdAt: new Date(strategy.created_at * 1000).toISOString(),
    updatedAt: new Date(strategy.updated_at * 1000).toISOString(),
  };
}

export function mapBackendStrategyDetail(record: BackendStrategyDetail): Strategy {
  return {
    ...mapBackendStrategy(record),
    creatorAddress: record.creator_address ?? undefined,
    updatedAt: new Date(record.updated_at * 1000).toISOString(),
    intent: mapBackendConditionalIntent(record.intent),
  };
}

export function mapBackendPortfolio(portfolio: BackendPortfolioResponse): Portfolio {
  return {
    address: portfolio.address,
    totalBalance: Number(portfolio.total_balance),
    allocated: Number(portfolio.allocated),
    available: Number(portfolio.available),
    yieldEarned: portfolio.yield_earned,
    mevRebates: portfolio.mev_rebates,
    positions: portfolio.positions.map((p) => ({
      asset: p.asset,
      protocol: p.protocol,
      chain: p.chain,
      amount: Number(p.amount),
      value: p.value,
      apy: p.apy,
    })),
  };
}

export function mapBackendProof(proof: BackendProofSummary): Proof {
  return {
    id: proof.id,
    type: proof.proof_type.toLowerCase() as "delegation" | "solvency" | "execution",
    intentId: proof.intent_id ?? undefined,
    verifier: proof.verifier,
    constraints: proof.constraints,
    proofTime: proof.proof_time,
    timestamp: new Date(proof.timestamp * 1000).toISOString(),
    verified: proof.verified,
    txHash: proof.tx_hash ?? undefined,
  };
}

export const api = {
  health: {
    // /api/v1 prefix: nginx only proxies /api/ to the API; bare /health would
    // hit the SPA fallback.
    get: () =>
      request<{ status: string; version: string; timestamp: number; auth_enabled?: boolean }>(
        "/api/v1/health"
      ),
  },
  auth: {
    challenge: (address: string) =>
      request<ChallengeResponse>("/api/v1/auth/challenge", {
        method: "POST",
        body: JSON.stringify({ address }),
      }),
    verify: (message: string, signature: string) =>
      request<VerifyResponse>("/api/v1/auth/verify", {
        method: "POST",
        body: JSON.stringify({ message, signature }),
      }),
  },
  intents: {
    parse: (text: string) =>
      request<ParsedIntentResponse>("/api/v1/intents/parse", {
        method: "POST",
        body: JSON.stringify({ text }),
      }),
    create: (text: string, delegationId?: string) =>
      request<{ id: string }>("/api/v1/intents", {
        method: "POST",
        body: JSON.stringify({ text, delegation_id: delegationId ?? null }),
      }),
    list: () => request<{ intents: BackendIntentRecord[] }>("/api/v1/intents"),
    get: (id: string) => request<BackendIntentRecord>(`/api/v1/intents/${id}`),
    events: (id: string) =>
      request<{ events: BackendIntentEvent[] }>(`/api/v1/intents/${id}/events`),
    cancel: (id: string) => request<void>(`/api/v1/intents/${id}`, { method: "DELETE" }),
  },
  executions: {
    list: () => request<{ executions: BackendExecutionRecord[] }>("/api/v1/executions"),
  },
  delegations: {
    list: () => request<{ delegations: BackendDelegationRecord[] }>("/api/v1/delegation"),
    revoke: (hash: string) =>
      request<void>(`/api/v1/delegation/${encodeURIComponent(hash)}`, {
        method: "DELETE",
      }),
    hash: (body: {
      pubkey_x: string;
      pubkey_y: string;
      allowed_intents: string;
      max_amounts: string[];
      allowed_protocols: string[];
      expiry: string;
      nonce: string;
      target_contract: string;
    }) =>
      request<{ delegation_hash: string }>("/api/v1/delegation/hash", {
        method: "POST",
        body: JSON.stringify(body),
      }),
    set: (body: {
      pubkey_x: string;
      pubkey_y: string;
      allowed_intents: string;
      max_amounts: string[];
      allowed_protocols: string[];
      expiry: string;
      nonce: string;
      target_contract: string;
      signature: string[];
    }) =>
      request<{ delegation_hash: string }>("/api/v1/delegation", {
        method: "POST",
        body: JSON.stringify(body),
      }),
  },
  agents: {
    list: () => request<{ agents: BackendAgentSummary[]; demo?: boolean }>("/api/v1/agents"),
    get: (id: string) => request<BackendAgentSummary>(`/api/v1/agents/${id}`),
    // The backend serves a single configured agent key and ignores the id in
    // this path (get_agent_pubkey in crates/interfaces/src/bin/otter_api.rs
    // has no path extractor). Per-agent pubkeys are a backend follow-up.
    pubkey: () => request<BackendAgentPubkeyResponse>("/api/v1/agents/otter-agent/pubkey"),
  },
  strategies: {
    list: () =>
      request<{ strategies: BackendStrategySummary[]; demo?: boolean }>("/api/v1/strategies"),
    get: (id: string) => request<BackendStrategyDetail>(`/api/v1/strategies/${id}`),
    create: (body: CreateStrategyPayload) =>
      request<{ id: string }>("/api/v1/strategies", {
        method: "POST",
        body: JSON.stringify(body),
      }),
    fork: (id: string) =>
      request<{ strategy_id: string; redirect_to: string }>(`/api/v1/strategies/${id}/fork`, {
        method: "POST",
      }),
  },
  portfolio: {
    get: () => request<BackendPortfolioResponse>("/api/v1/portfolio"),
  },
  proofs: {
    list: () => request<{ proofs: BackendProofSummary[]; demo?: boolean }>("/api/v1/proofs"),
  },
  solvency: {
    status: () =>
      request<{
        registry?: string;
        merkle_root?: string;
        total_deposits_wei?: string;
        last_proven_at?: number;
      }>("/api/v1/solvency/status"),
  },
  rebates: {
    list: () =>
      request<{ total_rebated_wei: string; rebate_bps: number }>("/api/v1/rebates"),
  },
  solana: {
    attest: (payloadHash: string) =>
      request<{ signature: string }>("/api/v1/solana/attest", {
        method: "POST",
        body: JSON.stringify({ payload_hash: payloadHash }),
      }),
    get: (authority: string) =>
      request<{ authority: string; payload_hash: string; timestamp: number }>(
        `/api/v1/solana/attestations/${authority}`
      ),
    verify: (authority: string, payloadHash: string) =>
      request<{ valid: boolean }>("/api/v1/solana/verify", {
        method: "POST",
        body: JSON.stringify({ authority, payload_hash: payloadHash }),
      }),
  },
  networks: {
    list: () => request<BackendNetworkStatus[]>("/api/v1/networks"),
  },
  bridge: {
    lock: (body: { network: string; amount_wei: string; destination_chain_id: number }) =>
      request<{ bridge_id: string; tx_hash: string }>("/api/v1/bridge/lock", {
        method: "POST",
        body: JSON.stringify(body),
      }),
    mint: (body: {
      network: string;
      user_address: string;
      amount_wei: string;
      bridge_id: string;
    }) =>
      request<{ tx_hash: string }>("/api/v1/bridge/mint", {
        method: "POST",
        body: JSON.stringify(body),
      }),
    transfers: (sourceChainId?: number) =>
      request<BackendBridgeTransfer[]>(
        `/api/v1/bridge/transfers${sourceChainId !== undefined ? `?chain_id=${sourceChainId}` : ""}`
      ),
  },
  mev: {
    bundles: () => request<BackendMevBundle[]>("/api/v1/mev/bundles"),
    getConfig: () => request<{ rebate_bps: number }>("/api/v1/mev/config"),
    setConfig: (rebateBps: number) =>
      request<{ rebate_bps: number }>("/api/v1/mev/config", {
        method: "POST",
        body: JSON.stringify({ rebate_bps: rebateBps }),
      }),
  },
};
