import type {
  ActivityItem,
  Agent,
  Delegation,
  ExecutionStatus,
  ExecutionStep,
  Intent,
  IntentStatus,
  IntentType,
  LeaderboardEntry,
  Portfolio,
  Position,
  Proof,
  Strategy,
} from "@/types/app";

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const randomDelay = () => delay(300 + Math.random() * 500);

const MOCK_USER = "0x7aA6A69d5b861C4cB859E64757f8Da6b91B9978F";

const AGENTS: Agent[] = [
  {
    id: "agent-1",
    name: "Aave Ace",
    operatedBy: "Otter",
    riskProfile: "Conservative",
    bond: 50000,
    reputation: 4.9,
    proofsSubmitted: 12403,
    yieldGenerated: 2450000,
    mevCaptured: 18400,
    uptime: 99.98,
    strategies: 12,
    followers: 3420,
    description:
      "Otter-operated lending specialist. Executes conservative lending strategies across Aave markets with steady, audited yields.",
  },
  {
    id: "agent-2",
    name: "Uni-Unicorn",
    operatedBy: "Otter",
    riskProfile: "Balanced",
    bond: 75000,
    reputation: 4.7,
    proofsSubmitted: 8932,
    yieldGenerated: 4120000,
    mevCaptured: 52300,
    uptime: 99.91,
    strategies: 8,
    followers: 2180,
    description:
      "Otter-operated liquidity execution agent. Runs protected swap and LP flows, capturing MEV rebates for depositors.",
  },
  {
    id: "agent-3",
    name: "Compound King",
    operatedBy: "Otter",
    riskProfile: "Conservative",
    bond: 32000,
    reputation: 4.5,
    proofsSubmitted: 5611,
    yieldGenerated: 980000,
    mevCaptured: 6100,
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
    bond: 100000,
    reputation: 4.8,
    proofsSubmitted: 3420,
    yieldGenerated: 1890000,
    mevCaptured: 12400,
    uptime: 99.72,
    strategies: 6,
    followers: 1560,
    description:
      "Otter-operated multi-chain strategist. Chases the best risk-adjusted yields across Ethereum and Arbitrum.",
  },
];

const POSITIONS: Position[] = [
  { asset: "USDC", protocol: "Aave", chain: "Ethereum", amount: 42000, value: 42000, apy: 4.2 },
  { asset: "ETH", protocol: "Aave", chain: "Ethereum", amount: 12.5, value: 28500, apy: 1.8 },
  { asset: "USDC", protocol: "Compound", chain: "Arbitrum", amount: 18000, value: 18000, apy: 3.9 },
];

const PORTFOLIO: Portfolio = {
  address: MOCK_USER,
  totalBalance: 88500,
  allocated: 88500,
  available: 11500,
  yieldEarned: 1240.5,
  mevRebates: 84.25,
  positions: POSITIONS,
};

const INTENTS: Intent[] = [
  {
    id: "intent-1",
    userAddress: MOCK_USER,
    rawText: "Lend 1000 USDC on Aave if yield > 3%",
    parsed: {
      type: "lend",
      amount: 1000,
      asset: "USDC",
      protocol: "Aave",
      condition: "yield > 3%",
      chain: "Ethereum",
    },
    status: "confirmed",
    createdAt: "2026-07-06T10:00:00Z",
    delegationId: "delegation-1",
    executedAt: "2026-07-06T10:02:14Z",
    txHash: "0xabc123def456",
    mevRebate: 0.45,
  },
  {
    id: "intent-2",
    userAddress: MOCK_USER,
    rawText: "Swap 500 USDC to ETH on Uniswap when gas < 20 gwei",
    parsed: {
      type: "swap",
      amount: 500,
      asset: "USDC",
      protocol: "Uniswap",
      condition: "gas < 20 gwei",
      chain: "Ethereum",
    },
    status: "monitoring",
    createdAt: "2026-07-07T08:30:00Z",
    delegationId: "delegation-2",
  },
  {
    id: "intent-3",
    userAddress: MOCK_USER,
    rawText: "Claim Aave rewards every Monday",
    parsed: {
      type: "claim",
      amount: 0,
      asset: "AAVE",
      protocol: "Aave",
      chain: "Ethereum",
    },
    status: "proving",
    createdAt: "2026-07-07T09:15:00Z",
    delegationId: "delegation-1",
  },
  {
    id: "intent-4",
    userAddress: MOCK_USER,
    rawText: "Withdraw 2000 USDC from Compound if utilization > 85%",
    parsed: {
      type: "withdraw",
      amount: 2000,
      asset: "USDC",
      protocol: "Compound",
      condition: "utilization > 85%",
      chain: "Arbitrum",
    },
    status: "condition_met",
    createdAt: "2026-07-07T09:45:00Z",
    delegationId: "delegation-3",
  },
];

const DELEGATIONS: Delegation[] = [
  {
    id: "delegation-1",
    userAddress: MOCK_USER,
    agentId: "agent-1",
    agentName: "Aave Ace",
    maxAmounts: { lend: 5000, swap: 2000, withdraw: 3000, claim: 1000 },
    allowedProtocols: ["Aave"],
    allowedChains: ["Ethereum"],
    expiry: "2026-08-06T10:00:00Z",
    createdAt: "2026-07-06T10:00:00Z",
    status: "active",
  },
  {
    id: "delegation-2",
    userAddress: MOCK_USER,
    agentId: "agent-2",
    agentName: "Uni-Unicorn",
    maxAmounts: { lend: 2000, swap: 5000, withdraw: 2000, claim: 500 },
    allowedProtocols: ["Uniswap"],
    allowedChains: ["Ethereum"],
    expiry: "2026-08-07T08:30:00Z",
    createdAt: "2026-07-07T08:30:00Z",
    status: "active",
  },
  {
    id: "delegation-3",
    userAddress: MOCK_USER,
    agentId: "agent-3",
    agentName: "Compound King",
    maxAmounts: { lend: 3000, swap: 1000, withdraw: 4000, claim: 800 },
    allowedProtocols: ["Compound"],
    allowedChains: ["Arbitrum"],
    expiry: "2026-08-07T09:45:00Z",
    createdAt: "2026-07-07T09:45:00Z",
    status: "active",
  },
];

const STRATEGIES: Strategy[] = [
  {
    id: "strategy-1",
    agentId: "agent-1",
    agentName: "Aave Ace",
    title: "Steady USDC Lending",
    description: "Otter official strategy. Lend USDC on Aave Ethereum whenever supply APY exceeds 3%.",
    rawText: "Lend USDC on Aave if yield > 3%",
    riskProfile: "Conservative",
    copies: 1240,
    totalVolume: 5400000,
    apy: 4.1,
    createdAt: "2026-06-15T00:00:00Z",
  },
  {
    id: "strategy-2",
    agentId: "agent-2",
    agentName: "Uni-Unicorn",
    title: "Low-Gas ETH Swaps",
    description: "Otter official strategy. Swap USDC to ETH on Uniswap only when base fee is below 20 gwei.",
    rawText: "Swap USDC to ETH on Uniswap when gas < 20 gwei",
    riskProfile: "Balanced",
    copies: 856,
    totalVolume: 2100000,
    apy: 0,
    createdAt: "2026-06-20T00:00:00Z",
  },
  {
    id: "strategy-3",
    agentId: "agent-4",
    agentName: "Cross-Chain Carl",
    title: "Arbitrum Yield Chase",
    description: "Otter official strategy. Move USDC to the highest yielding Aave or Compound market across chains.",
    rawText: "Lend USDC on highest yield market across Ethereum and Arbitrum",
    riskProfile: "Advanced",
    copies: 643,
    totalVolume: 1800000,
    apy: 5.2,
    createdAt: "2026-06-28T00:00:00Z",
  },
];

const PROOFS: Proof[] = [
  {
    id: "proof-1",
    type: "delegation",
    intentId: "intent-1",
    verifier: "DelegationVerifier",
    constraints: 847,
    proofTime: 1.2,
    timestamp: "2026-07-06T10:02:12Z",
    verified: true,
    txHash: "0xabc123def456",
  },
  {
    id: "proof-2",
    type: "solvency",
    verifier: "SolvencyVerifier",
    constraints: 1240,
    proofTime: 2.4,
    timestamp: "2026-07-07T00:00:00Z",
    verified: true,
  },
  {
    id: "proof-3",
    type: "execution",
    intentId: "intent-3",
    verifier: "ExecutionVerifier",
    constraints: 512,
    proofTime: 0.9,
    timestamp: "2026-07-07T09:15:02Z",
    verified: true,
  },
];

const ACTIVITY: ActivityItem[] = [
  {
    id: "act-1",
    type: "deposit",
    title: "Deposited 50,000 USDC into StrategyVault",
    amount: 50000,
    asset: "USDC",
    timestamp: "2026-07-05T14:00:00Z",
    txHash: "0xdep0001",
  },
  {
    id: "act-2",
    type: "intent_executed",
    title: "Lent 1,000 USDC on Aave",
    amount: 1000,
    asset: "USDC",
    timestamp: "2026-07-06T10:02:14Z",
    txHash: "0xabc123def456",
  },
  {
    id: "act-3",
    type: "mev_rebate",
    title: "Received MEV rebate",
    amount: 0.45,
    asset: "USDC",
    timestamp: "2026-07-06T10:02:14Z",
  },
  {
    id: "act-4",
    type: "delegation_created",
    title: "Created delegation to Uni-Unicorn",
    timestamp: "2026-07-07T08:30:00Z",
  },
  {
    id: "act-5",
    type: "intent_created",
    title: "Created intent: Swap 500 USDC to ETH",
    timestamp: "2026-07-07T08:30:05Z",
  },
];

const EXECUTION_STEPS: Record<IntentStatus, ExecutionStep> = {
  monitoring: {
    status: "monitoring",
    label: "Monitoring",
    detail: "Checking yield every 60s",
  },
  condition_met: {
    status: "condition_met",
    label: "Condition Met",
    detail: "Yield is now 3.2%",
  },
  proving: {
    status: "proving",
    label: "Generating Proof",
    detail: "847 constraints",
  },
  submitted: {
    status: "submitted",
    label: "Submitted",
    detail: "Transaction sent to vault",
  },
  confirmed: {
    status: "confirmed",
    label: "Confirmed",
    detail: "Action executed successfully",
  },
  failed: {
    status: "failed",
    label: "Failed",
    detail: "Proof verification failed",
  },
  revoked: {
    status: "revoked",
    label: "Revoked",
    detail: "Delegation revoked before execution",
  },
};

export async function getPortfolio(_address: string): Promise<Portfolio> {
  await randomDelay();
  return { ...PORTFOLIO };
}

export async function getIntents(
  _address: string,
  filters?: { status?: IntentStatus; type?: IntentType }
): Promise<Intent[]> {
  await randomDelay();
  let intents = [...INTENTS];
  if (filters?.status) {
    intents = intents.filter((i) => i.status === filters.status);
  }
  if (filters?.type) {
    intents = intents.filter((i) => i.parsed.type === filters.type);
  }
  return intents;
}

export async function getIntent(id: string): Promise<Intent | null> {
  await randomDelay();
  const intent = INTENTS.find((i) => i.id === id);
  return intent ? { ...intent } : null;
}

export async function parseIntent(text: string): Promise<{
  type: IntentType;
  amount: number;
  asset: string;
  protocol: string;
  condition?: string;
  chain: string;
}> {
  await randomDelay();
  const lower = text.toLowerCase();
  let type: IntentType = "lend";
  if (lower.includes("swap")) type = "swap";
  else if (lower.includes("withdraw")) type = "withdraw";
  else if (lower.includes("claim")) type = "claim";

  const amountMatch = text.match(/(\d+(?:\.\d+)?)/);
  const amount = amountMatch ? Number.parseFloat(amountMatch[1]) : 0;

  let asset = "USDC";
  if (lower.includes("eth")) asset = "ETH";
  else if (lower.includes("aave")) asset = "AAVE";

  let protocol = "Aave";
  if (lower.includes("uniswap")) protocol = "Uniswap";
  else if (lower.includes("compound")) protocol = "Compound";

  let condition: string | undefined;
  if (lower.includes("if")) condition = text.split("if")[1]?.trim();
  else if (lower.includes("when")) condition = text.split("when")[1]?.trim();

  const chain = lower.includes("arbitrum") ? "Arbitrum" : "Ethereum";

  return { type, amount, asset, protocol, condition, chain };
}

export async function createIntent(data: {
  rawText: string;
  parsed: {
    type: IntentType;
    amount: number;
    asset: string;
    protocol: string;
    condition?: string;
    chain: string;
  };
  delegationId: string;
}): Promise<Intent> {
  await randomDelay();
  const intent: Intent = {
    id: `intent-${Date.now()}`,
    userAddress: MOCK_USER,
    rawText: data.rawText,
    parsed: data.parsed,
    status: "monitoring",
    createdAt: new Date().toISOString(),
    delegationId: data.delegationId,
  };
  INTENTS.unshift(intent);
  return { ...intent };
}

export async function getDelegations(_address: string): Promise<Delegation[]> {
  await randomDelay();
  return [...DELEGATIONS];
}

export async function createDelegation(data: {
  agentId: string;
  maxAmounts: Record<IntentType, number>;
  allowedProtocols: string[];
  allowedChains: string[];
  expiryDays: number;
}): Promise<Delegation> {
  await randomDelay();
  const agent = AGENTS.find((a) => a.id === data.agentId);
  const expiry = new Date();
  expiry.setDate(expiry.getDate() + data.expiryDays);
  const delegation: Delegation = {
    id: `delegation-${Date.now()}`,
    userAddress: MOCK_USER,
    agentId: data.agentId,
    agentName: agent?.name ?? "Unknown Agent",
    maxAmounts: data.maxAmounts,
    allowedProtocols: data.allowedProtocols,
    allowedChains: data.allowedChains,
    expiry: expiry.toISOString(),
    createdAt: new Date().toISOString(),
    status: "active",
  };
  DELEGATIONS.unshift(delegation);
  return { ...delegation };
}

export async function revokeDelegation(id: string): Promise<Delegation | null> {
  await randomDelay();
  const delegation = DELEGATIONS.find((d) => d.id === id);
  if (!delegation) return null;
  delegation.status = "revoked";
  return { ...delegation };
}

export async function getAgents(): Promise<Agent[]> {
  await randomDelay();
  return [...AGENTS];
}

export async function getAgent(id: string): Promise<Agent | null> {
  await randomDelay();
  const agent = AGENTS.find((a) => a.id === id);
  return agent ? { ...agent } : null;
}

export async function getStrategies(): Promise<Strategy[]> {
  await randomDelay();
  return [...STRATEGIES];
}

export async function getLeaderboard(): Promise<LeaderboardEntry[]> {
  await randomDelay();
  const entries: LeaderboardEntry[] = AGENTS.map((agent, index) => ({
    rank: index + 1,
    agentId: agent.id,
    agentName: agent.name,
    proofsSubmitted: agent.proofsSubmitted,
    yieldGenerated: agent.yieldGenerated,
    mevCaptured: agent.mevCaptured,
    uptime: agent.uptime,
  }));
  return entries.sort((a, b) => b.proofsSubmitted - a.proofsSubmitted);
}

export async function getProofs(): Promise<Proof[]> {
  await randomDelay();
  return [...PROOFS];
}

export async function getExecutionStatus(intentId: string): Promise<ExecutionStatus> {
  await randomDelay();
  const intent = INTENTS.find((i) => i.id === intentId);
  const currentStep = intent?.status ?? "monitoring";
  const steps: ExecutionStep[] = [];
  const stepOrder: IntentStatus[] = [
    "monitoring",
    "condition_met",
    "proving",
    "submitted",
    "confirmed",
  ];
  const currentIndex = stepOrder.indexOf(currentStep);
  for (let i = 0; i <= currentIndex; i++) {
    const status = stepOrder[i];
    steps.push({
      ...EXECUTION_STEPS[status],
      status,
      timestamp: new Date(Date.now() - (currentIndex - i) * 60000).toISOString(),
    });
  }
  if (currentStep === "failed" || currentStep === "revoked") {
    steps.push({
      ...EXECUTION_STEPS[currentStep],
      status: currentStep,
      timestamp: new Date().toISOString(),
    });
  }
  return {
    intentId,
    currentStep,
    steps,
    startedAt: intent?.createdAt ?? new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}

export async function getActivity(_address: string): Promise<ActivityItem[]> {
  await randomDelay();
  return [...ACTIVITY];
}

export async function copyStrategy(strategyId: string): Promise<Strategy> {
  await randomDelay();
  const strategy = STRATEGIES.find((s) => s.id === strategyId);
  if (!strategy) throw new Error("Strategy not found");
  strategy.copies += 1;
  return { ...strategy };
}

export async function followAgent(agentId: string): Promise<Agent> {
  await randomDelay();
  const agent = AGENTS.find((a) => a.id === agentId);
  if (!agent) throw new Error("Agent not found");
  agent.followers += 1;
  return { ...agent };
}
