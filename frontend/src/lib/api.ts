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
  risk_profile: string;
  bond: number;
  reputation: number;
  proofs_submitted: number;
  yield_generated: number;
  mev_captured: number;
  uptime: number;
  strategies: number;
  followers: number;
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

export interface BackendLeaderboardEntry {
  rank: number;
  agent_id: string;
  agent_name: string;
  proofs_submitted: number;
  yield_generated: number;
  mev_captured: number;
  uptime: number;
}

export interface ChallengeResponse {
  message: string;
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

export function mapBackendConditionalIntent(conditional: BackendConditionalIntent): ParsedIntent {
  const condition = formatCondition(conditional.condition);
  const inner = conditional.intent;

  // The backend payload carries no chain; leave it undefined instead of guessing.
  if ("Lend" in inner) {
    return {
      type: "lend",
      amount: Number(inner.Lend.amount),
      asset: assetSymbol(inner.Lend.asset),
      protocol: lendingProtocolName(inner.Lend.protocol),
      condition,
    };
  }

  if ("Stake" in inner) {
    return {
      type: "lend",
      amount: Number(inner.Stake.amount),
      asset: assetSymbol(inner.Stake.asset),
      protocol: lendingProtocolName(inner.Stake.protocol),
      condition,
    };
  }

  if ("Borrow" in inner) {
    return {
      type: "withdraw",
      amount: Number(inner.Borrow.amount),
      asset: assetSymbol(inner.Borrow.asset),
      protocol: lendingProtocolName(inner.Borrow.protocol),
      condition,
    };
  }

  if ("Swap" in inner) {
    return {
      type: "swap",
      amount: Number(inner.Swap.amount),
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
    delegationId: "",
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
    operatedBy: agent.operated_by as "Otter",
    riskProfile: agent.risk_profile as "Conservative" | "Balanced" | "Advanced",
    bond: agent.bond,
    reputation: agent.reputation,
    proofsSubmitted: agent.proofs_submitted,
    yieldGenerated: agent.yield_generated,
    mevCaptured: agent.mev_captured,
    uptime: agent.uptime,
    strategies: agent.strategies,
    followers: agent.followers,
    description: agent.description,
  };
}

export function mapBackendStrategy(strategy: BackendStrategySummary): Strategy {
  return {
    id: strategy.id,
    agentId: strategy.agent_id,
    agentName: strategy.agent_name,
    title: strategy.title,
    description: strategy.description,
    rawText: strategy.raw_text,
    riskProfile: strategy.risk_profile as "Conservative" | "Balanced" | "Advanced",
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

export function mapBackendLeaderboardEntry(entry: BackendLeaderboardEntry): LeaderboardEntry {
  return {
    rank: entry.rank,
    agentId: entry.agent_id,
    agentName: entry.agent_name,
    proofsSubmitted: entry.proofs_submitted,
    yieldGenerated: entry.yield_generated,
    mevCaptured: entry.mev_captured,
    uptime: entry.uptime,
  };
}

export const api = {
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
    create: (text: string) =>
      request<{ id: string }>("/api/v1/intents", {
        method: "POST",
        body: JSON.stringify({ text }),
      }),
    list: () => request<{ intents: BackendIntentRecord[] }>("/api/v1/intents"),
    get: (id: string) => request<BackendIntentRecord>(`/api/v1/intents/${id}`),
    cancel: (id: string) => request<void>(`/api/v1/intents/${id}`, { method: "DELETE" }),
  },
  executions: {
    list: () => request<{ executions: BackendExecutionRecord[] }>("/api/v1/executions"),
  },
  delegations: {
    list: () => request<{ delegations: BackendDelegationRecord[] }>("/api/v1/delegation"),
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
  leaderboard: {
    get: () =>
      request<{ entries: BackendLeaderboardEntry[]; demo?: boolean }>("/api/v1/leaderboard"),
  },
};
