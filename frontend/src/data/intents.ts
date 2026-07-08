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
