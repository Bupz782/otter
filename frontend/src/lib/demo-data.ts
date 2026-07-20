import type {
  ActivityItem,
  Agent,
  Delegation,
  Intent,
  LeaderboardEntry,
  Portfolio,
  Proof,
  Strategy,
} from "@/types/app";

/**
 * Demo fixtures shown to visitors who have not signed in. Every read hook
 * returns these when unauthenticated (flagged via `isDemo`), so the app is
 * fully explorable before connect. Agents and strategies mirror the backend
 * seed set (default_agents/default_strategies in
 * crates/interfaces/src/bin/otter_api.rs) so demo mode matches the live
 * experience. Nothing here is ever sent to the API; ids are namespaced with
 * the `demo-` prefix.
 *
 * Timestamps are computed relative to module load so the demo always looks
 * fresh, and delegation expiry always lands in the future.
 */

const DEMO_ADDRESS = "0xdea7f0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7";

function hoursAgo(hours: number): string {
  return new Date(Date.now() - hours * 60 * 60 * 1000).toISOString();
}

function daysFromNow(days: number): string {
  return new Date(Date.now() + days * 24 * 60 * 60 * 1000).toISOString();
}

/** True for ids from this module (all prefixed `demo-`). */
export function isDemoId(id: string): boolean {
  return id.startsWith("demo-");
}

// ---------------------------------------------------------------------------
// Portfolio
// ---------------------------------------------------------------------------

export const demoPortfolio: Portfolio = {
  address: DEMO_ADDRESS,
  totalBalance: 12480.52,
  allocated: 9196,
  available: 3284.52,
  yieldEarned: 312.44,
  mevRebates: 87.19,
  positions: [
    { asset: "USDC", protocol: "Aave", chain: "Ethereum", amount: 4000, value: 4000, apy: 4.2 },
    { asset: "USDC", protocol: "Compound", chain: "Arbitrum", amount: 2700, value: 2700, apy: 3.8 },
    { asset: "ETH", protocol: "Aave", chain: "Ethereum", amount: 0.78, value: 2496, apy: 2.1 },
  ],
};

// ---------------------------------------------------------------------------
// Intents: one per key status so every badge and timeline state is visible.
// ---------------------------------------------------------------------------

export const demoIntents: Intent[] = [
  {
    id: "demo-intent-1",
    userAddress: DEMO_ADDRESS,
    rawText: "Lend 1,000 USDC on Aave if yield > 3%",
    parsed: {
      type: "lend",
      amount: 1000,
      asset: "USDC",
      protocol: "Aave",
      condition: "Yield > 3%",
      chain: "Ethereum",
    },
    status: "monitoring",
    createdAt: hoursAgo(2),
    delegationId: "demo-deleg-1",
  },
  {
    id: "demo-intent-2",
    userAddress: DEMO_ADDRESS,
    rawText: "Swap 500 USDC to ETH on Uniswap when gas < 20 gwei",
    parsed: {
      type: "swap",
      amount: 500,
      asset: "USDC",
      protocol: "Uniswap",
      condition: "GasCost < 20 gwei",
      chain: "Ethereum",
    },
    status: "condition_met",
    createdAt: hoursAgo(9),
    delegationId: "demo-deleg-2",
  },
  {
    id: "demo-intent-3",
    userAddress: DEMO_ADDRESS,
    rawText: "Withdraw 2,000 USDC from Compound if utilization > 85%",
    parsed: {
      type: "withdraw",
      amount: 2000,
      asset: "USDC",
      protocol: "Compound",
      condition: "Volume > 85%",
      chain: "Arbitrum",
    },
    status: "proving",
    createdAt: hoursAgo(26),
    delegationId: "demo-deleg-1",
  },
  {
    id: "demo-intent-4",
    userAddress: DEMO_ADDRESS,
    rawText: "Lend 750 USDC on Aave if yield > 4%",
    parsed: {
      type: "lend",
      amount: 750,
      asset: "USDC",
      protocol: "Aave",
      condition: "Yield > 4%",
      chain: "Ethereum",
    },
    status: "confirmed",
    createdAt: hoursAgo(50),
    delegationId: "demo-deleg-1",
    executedAt: hoursAgo(48),
    txHash: "0x8f2c4a1b9d3e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcd",
    mevRebate: 3.42,
  },
];

// ---------------------------------------------------------------------------
// Delegations
// ---------------------------------------------------------------------------

export const demoDelegations: Delegation[] = [
  {
    id: "demo-deleg-1",
    createdAt: hoursAgo(72),
    userAddress: DEMO_ADDRESS,
    agentId: "agent-1",
    agentName: "Aave Ace",
    maxAmounts: { lend: 5000, swap: 2000, withdraw: 3000, claim: 1000 },
    allowedProtocols: ["Aave", "Compound"],
    allowedChains: ["Ethereum", "Arbitrum"],
    expiry: daysFromNow(30),
    status: "active",
  },
  {
    id: "demo-deleg-2",
    createdAt: hoursAgo(20),
    userAddress: DEMO_ADDRESS,
    agentId: "agent-2",
    agentName: "Uni-Unicorn",
    maxAmounts: { lend: 1000, swap: 2500, withdraw: 1000, claim: 500 },
    allowedProtocols: ["Uniswap"],
    allowedChains: ["Ethereum"],
    expiry: daysFromNow(14),
    status: "active",
  },
];

// ---------------------------------------------------------------------------
// Agents and strategies: exact mirrors of the backend seeds.
// ---------------------------------------------------------------------------

export const demoAgents: Agent[] = [
  {
    id: "agent-1",
    name: "Aave Ace",
    operatedBy: "Otter",
    riskProfile: "Conservative",
    bond: 50_000,
    reputation: 4.9,
    proofsSubmitted: 12_403,
    yieldGenerated: 2_450_000,
    mevCaptured: 18_400,
    uptime: 99.98,
    strategies: 12,
    followers: 3_420,
    description:
      "Otter-operated lending specialist. Executes conservative lending strategies across Aave markets with steady, audited yields.",
  },
  {
    id: "agent-2",
    name: "Uni-Unicorn",
    operatedBy: "Otter",
    riskProfile: "Balanced",
    bond: 75_000,
    reputation: 4.7,
    proofsSubmitted: 8_932,
    yieldGenerated: 4_120_000,
    mevCaptured: 52_300,
    uptime: 99.91,
    strategies: 8,
    followers: 2_180,
    description:
      "Otter-operated liquidity execution agent. Runs protected swap and LP flows, capturing MEV rebates for depositors.",
  },
  {
    id: "agent-3",
    name: "Compound King",
    operatedBy: "Otter",
    riskProfile: "Conservative",
    bond: 32_000,
    reputation: 4.5,
    proofsSubmitted: 5_611,
    yieldGenerated: 980_000,
    mevCaptured: 6_100,
    uptime: 99.85,
    strategies: 5,
    followers: 890,
    description:
      "Otter-operated Compound specialist. Automates rate arbitrage and rebalancing between Compound markets.",
  },
  {
    id: "agent-4",
    name: "Cross-Chain Carl",
    operatedBy: "Otter",
    riskProfile: "Advanced",
    bond: 100_000,
    reputation: 4.8,
    proofsSubmitted: 3_420,
    yieldGenerated: 1_890_000,
    mevCaptured: 12_400,
    uptime: 99.72,
    strategies: 6,
    followers: 1_560,
    description:
      "Otter-operated multi-chain strategist. Chases the best risk-adjusted yields across Ethereum and Arbitrum.",
  },
];

export const demoStrategies: Strategy[] = [
  {
    id: "strategy-1",
    agentId: "agent-1",
    agentName: "Aave Ace",
    title: "Steady USDC Lending",
    description:
      "Otter official strategy. Lend USDC on Aave Ethereum whenever supply APY exceeds 3%.",
    rawText: "Lend USDC on Aave if yield > 3%",
    riskProfile: "Conservative",
    copies: 1_240,
    totalVolume: 5_400_000,
    apy: 4.1,
    createdAt: new Date(1_720_000_000 * 1000).toISOString(),
  },
  {
    id: "strategy-2",
    agentId: "agent-2",
    agentName: "Uni-Unicorn",
    title: "Low-Gas ETH Swaps",
    description:
      "Otter official strategy. Swap USDC to ETH on Uniswap only when base fee is below 20 gwei.",
    rawText: "Swap USDC to ETH on Uniswap when gas < 20 gwei",
    riskProfile: "Balanced",
    copies: 856,
    totalVolume: 2_100_000,
    apy: 0,
    createdAt: new Date(1_720_500_000 * 1000).toISOString(),
  },
  {
    id: "strategy-3",
    agentId: "agent-4",
    agentName: "Cross-Chain Carl",
    title: "Arbitrum Yield Chase",
    description:
      "Otter official strategy. Move USDC to the highest yielding Aave or Compound market across chains.",
    rawText: "Lend USDC on highest yield market across Ethereum and Arbitrum",
    riskProfile: "Advanced",
    copies: 643,
    totalVolume: 1_800_000,
    apy: 5.2,
    createdAt: new Date(1_720_900_000 * 1000).toISOString(),
  },
];

// Ranked by proof count, same rule as the backend leaderboard endpoint.
export const demoLeaderboard: LeaderboardEntry[] = [...demoAgents]
  .sort((a, b) => b.proofsSubmitted - a.proofsSubmitted)
  .map((agent, index) => ({
    rank: index + 1,
    agentId: agent.id,
    agentName: agent.name,
    proofsSubmitted: agent.proofsSubmitted,
    yieldGenerated: agent.yieldGenerated,
    mevCaptured: agent.mevCaptured,
    uptime: agent.uptime,
  }));

// ---------------------------------------------------------------------------
// Proofs: verifier names and constraint counts mirror the backend response.
// ---------------------------------------------------------------------------

export const demoProofs: Proof[] = [
  {
    id: "demo-proof-solvency-1",
    type: "solvency",
    verifier: "SolvencyVerifier",
    constraints: 1240,
    proofTime: 2.4,
    timestamp: hoursAgo(1),
    verified: true,
  },
  {
    id: "demo-proof-exec-1",
    type: "execution",
    intentId: "demo-intent-4",
    verifier: "ExecutionVerifier",
    constraints: 512,
    proofTime: 0.9,
    timestamp: hoursAgo(48),
    verified: true,
    txHash: "0x8f2c4a1b9d3e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcd",
  },
  {
    id: "demo-proof-exec-2",
    type: "execution",
    intentId: "demo-intent-3",
    verifier: "ExecutionVerifier",
    constraints: 512,
    proofTime: 0.9,
    timestamp: hoursAgo(25),
    verified: true,
  },
];

// ---------------------------------------------------------------------------
// Activity: derived from the demo intents and delegations, newest first.
// ---------------------------------------------------------------------------

function buildDemoActivity(): ActivityItem[] {
  const items: ActivityItem[] = [];

  for (const intent of demoIntents) {
    items.push({
      id: `demo-activity-created-${intent.id}`,
      type: "intent_created",
      title: `Intent created: ${intent.rawText}`,
      timestamp: intent.createdAt,
    });
    if (intent.executedAt) {
      items.push({
        id: `demo-activity-executed-${intent.id}`,
        type: "intent_executed",
        title: `Intent executed: ${intent.rawText}`,
        timestamp: intent.executedAt,
        txHash: intent.txHash,
      });
    }
  }

  for (const delegation of demoDelegations) {
    items.push({
      id: `demo-activity-delegation-${delegation.id}`,
      type: "delegation_created",
      title: `Delegation signed: ${delegation.agentName ?? delegation.agentId}`,
      timestamp: delegation.createdAt,
    });
  }

  return items
    .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
    .slice(0, 10);
}

export const demoActivity: ActivityItem[] = buildDemoActivity();
