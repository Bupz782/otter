import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  api,
  ApiClientError,
  mapBackendConditionalIntent,
  mapBackendIntent,
  type BackendConditionalIntent,
  type BackendIntentRecord,
} from "@/lib/api";

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
    });
  });

  it("maps a Stake intent to a lend parsed intent", () => {
    const backend: BackendConditionalIntent = {
      intent: {
        Stake: {
          asset: "Eth",
          amount: "0.25",
          protocol: "Compound",
        },
      },
      condition: null,
    };

    const parsed = mapBackendConditionalIntent(backend);

    expect(parsed).toEqual({
      type: "lend",
      amount: 0.25,
      asset: "ETH",
      protocol: "Compound",
    });
  });

  it("maps a Borrow intent to a withdraw parsed intent", () => {
    const backend: BackendConditionalIntent = {
      intent: {
        Borrow: {
          asset: "Dai",
          amount: "800",
          collateral: "Wbtc",
          collateral_amount: "0.05",
          protocol: "Aave",
        },
      },
      condition: null,
    };

    const parsed = mapBackendConditionalIntent(backend);

    expect(parsed).toEqual({
      type: "withdraw",
      amount: 800,
      asset: "DAI",
      protocol: "Aave",
    });
  });

  it("formats multi-character comparators in conditions", () => {
    const backend: BackendConditionalIntent = {
      intent: {
        Lend: {
          asset: "Link",
          amount: "10",
          protocol: "Aave",
        },
      },
      condition: {
        Comparison: {
          metric: "GasCost",
          comparator: "LessThanOrEqualTo",
          value: "30",
        },
      },
    };

    const parsed = mapBackendConditionalIntent(backend);

    expect(parsed.condition).toBe("GasCost <= 30");
  });
});

function intentRecord(state: string): BackendIntentRecord {
  return {
    id: "intent-1",
    text: "lend 100 USDC on Aave",
    intent: {
      intent: {
        Lend: {
          asset: "Usdc",
          amount: "100",
          protocol: "Aave",
        },
      },
      condition: null,
    },
    state,
    created_at: 1_700_000_000,
    updated_at: 1_700_000_600,
  };
}

describe("mapBackendIntent", () => {
  it("maps an active record to monitoring without a tx hash", () => {
    const intent = mapBackendIntent(intentRecord("active"));

    expect(intent.status).toBe("monitoring");
    expect(intent.txHash).toBeUndefined();
    expect(intent.executedAt).toBeUndefined();
    expect(intent.createdAt).toBe(new Date(1_700_000_000 * 1000).toISOString());
  });

  it("extracts the tx hash from a submitted state", () => {
    const intent = mapBackendIntent(intentRecord("submitted:0xabc123"));

    expect(intent.status).toBe("submitted");
    expect(intent.txHash).toBe("0xabc123");
    expect(intent.executedAt).toBeUndefined();
  });

  it("maps an executed state to confirmed with tx hash and execution date", () => {
    const intent = mapBackendIntent(intentRecord("executed:0xdeadbeef"));

    expect(intent.status).toBe("confirmed");
    expect(intent.txHash).toBe("0xdeadbeef");
    expect(intent.executedAt).toBe(new Date(1_700_000_600 * 1000).toISOString());
  });

  it("maps a cancelled state to revoked", () => {
    const intent = mapBackendIntent(intentRecord("cancelled"));

    expect(intent.status).toBe("revoked");
    expect(intent.txHash).toBeUndefined();
  });
});

describe("api request handling", () => {
  beforeEach(() => {
    // Node's experimental localStorage global shadows jsdom's in this
    // environment; stub the surface request() relies on.
    vi.stubGlobal("localStorage", {
      getItem: () => null,
      setItem: () => {},
      removeItem: () => {},
    });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("throws an ApiClientError with the backend message and status on failure", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        status: 401,
        statusText: "Unauthorized",
        json: () => Promise.resolve({ error: "invalid signature" }),
      })
    );

    const promise = api.intents.list();

    await expect(promise).rejects.toBeInstanceOf(ApiClientError);
    await expect(promise).rejects.toMatchObject({
      message: "invalid signature",
      status: 401,
    });
  });

  it("falls back to the HTTP status text when the error body has no message", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        status: 500,
        statusText: "Internal Server Error",
        json: () => Promise.reject(new Error("not json")),
      })
    );

    await expect(api.intents.list()).rejects.toMatchObject({
      message: "Internal Server Error",
      status: 500,
    });
  });

  it("resolves to undefined for a 204 No Content response", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        status: 204,
        statusText: "No Content",
        json: () => Promise.reject(new Error("no body")),
      })
    );

    await expect(api.intents.cancel("intent-1")).resolves.toBeUndefined();
  });
});
