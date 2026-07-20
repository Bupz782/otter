export interface ParsedIntent {
  action: string;
  amount: string;
  asset: string;
  condition: string;
  target: string;
  chain: string;
}

export interface ExecutionResult {
  txHash: string;
  gasUsed: string;
  blockNumber: number;
  status: "confirmed" | "pending";
}

export interface MockIntent {
  id: string;
  prompt: string;
  parsed: ParsedIntent;
  delegationHash: string;
  proofHash: string;
  result: ExecutionResult;
}

export const mockIntents: MockIntent[] = [
  {
    id: "1",
    prompt: "Buy 1 ETH if the price drops under 1,800 USDC",
    parsed: {
      action: "buy",
      amount: "1 ETH",
      asset: "ETH",
      condition: "ETH/USDC < 1,800",
      target: "Uniswap V3",
      chain: "Ethereum",
    },
    delegationHash: "0x7a3f...9e2c",
    proofHash: "0x4b8d...1f4a",
    result: {
      txHash: "0x9c2a...b71d",
      gasUsed: "124,300",
      blockNumber: 18294021,
      status: "confirmed",
    },
  },
  {
    id: "2",
    prompt: "Lend 1,000 USDC on Aave if APY is above 5%",
    parsed: {
      action: "lend",
      amount: "1,000 USDC",
      asset: "USDC",
      condition: "Aave USDC APY > 5%",
      target: "Aave V3",
      chain: "Base",
    },
    delegationHash: "0x2e9b...6d1f",
    proofHash: "0x8c5e...3a7b",
    result: {
      txHash: "0x1d4f...e82a",
      gasUsed: "98,700",
      blockNumber: 14230567,
      status: "confirmed",
    },
  },
  {
    id: "3",
    prompt: "Swap ARB to USDC if the price goes above $1.20",
    parsed: {
      action: "swap",
      amount: "All ARB",
      asset: "ARB",
      condition: "ARB/USDC > 1.20",
      target: "Camelot",
      chain: "Arbitrum",
    },
    delegationHash: "0x5f1c...8a3e",
    proofHash: "0x9d2a...7c5b",
    result: {
      txHash: "0x3b7e...d91c",
      gasUsed: "156,200",
      blockNumber: 98450231,
      status: "confirmed",
    },
  },
];

// Candidate tokens per intent: its traded asset plus the tickers in its condition pair.
function intentTokens(intent: MockIntent): string[] {
  const tokens = new Set<string>([intent.parsed.asset.toLowerCase()]);
  for (const token of intent.parsed.condition.match(/[A-Za-z]{2,}/g) ?? []) {
    tokens.add(token.toLowerCase());
  }
  return [...tokens];
}

// Score every intent by how many of its tokens appear in the prompt (word-boundary
// match, so "ARB" does not fire inside "barbell"). Most tokens wins, then longest
// total match, then array order, so the demo stays deterministic. Falls back to the
// first intent when nothing matches.
export function matchMockIntent(prompt: string): MockIntent {
  const scored = mockIntents.map((intent, index) => {
    const matched = intentTokens(intent).filter((token) =>
      new RegExp(`\\b${token}\\b`, "i").test(prompt)
    );
    return {
      intent,
      index,
      count: matched.length,
      length: matched.reduce((sum, token) => sum + token.length, 0),
    };
  });
  scored.sort((a, b) => b.count - a.count || b.length - a.length || a.index - b.index);
  const best = scored[0];
  return best.count > 0 ? best.intent : mockIntents[0];
}

export const reasoningSteps = [
  "Parsing natural language intent into structured condition",
  "Resolving target asset, protocol, and chain",
  "Verifying delegation limits against the Vault registry",
  "Generating zero-knowledge proof of authorization",
  "Simulating on-chain execution and gas estimate",
  "Intent ready for trustless execution",
];

export const promptSuggestions = [
  "Buy 1 ETH if it drops under 1,800 USDC",
  "Lend 1,000 USDC on Aave if APY > 5%",
  "Swap ARB to USDC if price goes above $1.20",
  "Exit ETH/USDC LP if impermanent loss > 2%",
];
