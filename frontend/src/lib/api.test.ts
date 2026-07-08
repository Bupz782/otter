import { describe, it, expect } from "vitest";
import { mapBackendConditionalIntent, type BackendConditionalIntent } from "@/lib/api";

describe("mapBackendConditionalIntent", () => {
  it("maps a Lend intent with a yield condition", () => {
    const backend: BackendConditionalIntent = {
      intent: {
        Lend: {
          asset: "Usdc",
          amount: "1500.50",
          protocol: "Aave",
        },
      },
      condition: {
        Comparison: {
          metric: "Yield",
          comparator: "GreaterThan",
          value: "5",
        },
      },
    };

    const parsed = mapBackendConditionalIntent(backend);

    expect(parsed).toEqual({
      type: "lend",
      amount: 1500.5,
      asset: "USDC",
      protocol: "Aave",
      condition: "Yield > 5",
      chain: "Ethereum",
    });
  });

  it("maps a Swap intent without a condition", () => {
    const backend: BackendConditionalIntent = {
      intent: {
        Swap: {
          from_asset: "Eth",
          to_asset: "Dai",
          amount: "2",
          protocol: "Uniswap",
        },
      },
      condition: null,
    };

    const parsed = mapBackendConditionalIntent(backend);

    expect(parsed).toEqual({
      type: "swap",
      amount: 2,
      asset: "ETH",
      protocol: "Uniswap",
      chain: "Ethereum",
    });
  });

  it("falls back to a default lend intent for an empty Composite", () => {
    const backend: BackendConditionalIntent = {
      intent: {
        Composite: {
          intents: [],
        },
      },
      condition: null,
    };

    const parsed = mapBackendConditionalIntent(backend);

    expect(parsed).toEqual({
      type: "lend",
      amount: 0,
      asset: "USDC",
      protocol: "Otter",
      chain: "Ethereum",
    });
  });
});
