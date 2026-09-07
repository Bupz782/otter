import type {
  ActivityItem,
  Agent,
  Delegation,
  Intent,
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
    agentId: "otter-agent",
    agentName: "Otter Agent",
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
    agentId: "otter-agent",
    agentName: "Otter Agent",
    maxAmounts: { lend: 1000, swap: 2500, withdraw: 1000, claim: 500 },
    allowedProtocols: ["Uniswap"],
    allowedChains: ["Ethereum"],
    expiry: daysFromNow(14),
    status: "active",
  },
];

// ---------------------------------------------------------------------------
// Agent and strategies: exact mirrors of the backend seeds. Single
// protocol-operated agent — users delegate to it, there is no marketplace.
// ---------------------------------------------------------------------------

export const demoAgents: Agent[] = [
  {
    id: "otter-agent",
    name: "Otter Agent",
    operatedBy: "Otter",
    bond: 100_000,
    proofsSubmitted: 30_366,
    yieldGenerated: 9_440_000,
    mevCaptured: 89_200,
    uptime: 99.9,
    description:
      "The protocol-operated execution agent. It executes user intents inside the limits of their signed delegation, and every execution carries a ZK proof verified on-chain before funds move.",
  },
];

export const demoStrategies: Strategy[] = [
  {
    id: "strategy-1",
    title: "Steady USDC Lending",
    description:
      "Otter official strategy. Lend USDC on Aave Ethereum whenever supply APY exceeds 3%.",
    rawText: "Lend 1000 USDC on Aave if yield > 3%",
    copies: 1_240,
    totalVolume: 5_400_000,
    apy: 4.1,
    createdAt: new Date(1_720_000_000 * 1000).toISOString(),
    updatedAt: new Date(1_720_000_000 * 1000).toISOString(),
  },
  {
    id: "strategy-2",
    title: "Low-Gas ETH Swaps",
    description:
      "Otter official strategy. Swap USDC to ETH on Uniswap only when base fee is below 20 gwei.",
    rawText: "Swap 1000 USDC for ETH on Uniswap if gas < 20",
    copies: 856,
    totalVolume: 2_100_000,
    apy: 0,
    createdAt: new Date(1_720_500_000 * 1000).toISOString(),
    updatedAt: new Date(1_720_500_000 * 1000).toISOString(),
  },
  {
    id: "strategy-3",
    title: "Compound Rate Hunter",
    description:
      "Otter official strategy. Lend USDC on Compound whenever the supply APY exceeds 5%.",
    rawText: "Lend 1000 USDC on Compound if yield > 5%",
    copies: 643,
    totalVolume: 1_800_000,
    apy: 5.2,
    createdAt: new Date(1_720_900_000 * 1000).toISOString(),
    updatedAt: new Date(1_720_900_000 * 1000).toISOString(),
  },
];

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
