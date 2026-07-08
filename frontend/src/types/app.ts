export type IntentStatus =
  "monitoring" | "condition_met" | "proving" | "submitted" | "confirmed" | "failed" | "revoked";

export type IntentType = "lend" | "swap" | "withdraw" | "claim";

export interface ParsedIntent {
  type: IntentType;
  amount: number;
  asset: string;
  protocol: string;
  condition?: string;
  chain: string;
}

export interface Intent {
  id: string;
  userAddress: string;
  rawText: string;
  parsed: ParsedIntent;
  status: IntentStatus;
  createdAt: string;
  delegationId: string;
  executedAt?: string;
  txHash?: string;
  mevRebate?: number;
}

export interface Delegation {
  id: string;
  userAddress: string;
  agentId: string;
  agentName: string;
  maxAmounts: Record<IntentType, number>;
  allowedProtocols: string[];
  allowedChains: string[];
  expiry: string;
  createdAt: string;
  status: "active" | "revoked" | "expired";
}

export interface Agent {
  id: string;
  name: string;
  avatarUrl?: string;
  operatedBy: "Otter";
  riskProfile: "Conservative" | "Balanced" | "Advanced";
  bond: number;
  reputation: number;
  proofsSubmitted: number;
  yieldGenerated: number;
  mevCaptured: number;
  uptime: number;
  strategies: number;
  followers: number;
  description: string;
}

export interface Strategy {
  id: string;
  agentId: string;
  agentName: string;
  title: string;
  description: string;
  rawText: string;
  riskProfile: "Conservative" | "Balanced" | "Advanced";
  copies: number;
  totalVolume: number;
  apy: number;
  createdAt: string;
}

export interface ComparisonPoint {
  id: string;
  title: string;
  icon: string;
  comparison: string;
  description: string;
}

export interface Portfolio {
  address: string;
  totalBalance: number;
  allocated: number;
  available: number;
  yieldEarned: number;
  mevRebates: number;
  positions: Position[];
}

export interface Position {
  asset: string;
  protocol: string;
  chain: string;
  amount: number;
  value: number;
  apy: number;
}

export interface ExecutionStatus {
  intentId: string;
  currentStep: IntentStatus;
  steps: ExecutionStep[];
  startedAt: string;
  updatedAt: string;
}

export interface ExecutionStep {
  status: IntentStatus;
  label: string;
  detail: string;
  timestamp?: string;
}

export interface Proof {
  id: string;
  type: "delegation" | "solvency" | "execution";
  intentId?: string;
  verifier: string;
  constraints: number;
  proofTime: number;
  timestamp: string;
  verified: boolean;
  txHash?: string;
}

export interface LeaderboardEntry {
  rank: number;
  agentId: string;
  agentName: string;
  proofsSubmitted: number;
  yieldGenerated: number;
  mevCaptured: number;
  uptime: number;
}

export interface ActivityItem {
  id: string;
  type:
    | "deposit"
    | "withdraw"
    | "intent_created"
    | "intent_executed"
    | "delegation_created"
    | "mev_rebate";
  title: string;
  amount?: number;
  asset?: string;
  timestamp: string;
  txHash?: string;
}
