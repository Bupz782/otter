import { describe, it, expect } from "vitest";
import {
  buildDelegationMessage,
  fieldFromU256,
  splitSignatureForBackend,
} from "@/lib/delegation";

describe("splitSignatureForBackend", () => {
  it("strips the recovery id from a 65-byte signature", () => {
    const signature = `0x${"ab".repeat(64)}1b` as `0x${string}`;

    const bytes = splitSignatureForBackend(signature);

    expect(bytes).toHaveLength(64);
    expect(bytes.every((b) => b === "0xab")).toBe(true);
  });

  it("accepts an already 64-byte signature unchanged", () => {
    const signature = `0x${"cd".repeat(64)}` as `0x${string}`;

    const bytes = splitSignatureForBackend(signature);

    expect(bytes).toHaveLength(64);
    expect(bytes[0]).toBe("0xcd");
    expect(bytes[63]).toBe("0xcd");
  });

  it("rejects signatures that are not 64 or 65 bytes", () => {
    const signature = `0x${"ab".repeat(10)}` as `0x${string}`;

    expect(() => splitSignatureForBackend(signature)).toThrow(
      "Expected 64-byte signature, got 10"
    );
  });
});

describe("buildDelegationMessage", () => {
  const pubkeyX = `0x${"11".repeat(32)}`;
  const pubkeyY = `0x${"22".repeat(32)}`;
  const targetContract = `0x${"33".repeat(20)}`;

  it("sets the allowed-intents bitmap bits for each positive limit", () => {
    // swap -> bit 0, lend -> bit 3: bitmap = 0b1001 = 9
    const message = buildDelegationMessage(
      pubkeyX,
      pubkeyY,
      { lend: 100, swap: 50, withdraw: 0, claim: 0 },
      ["Aave"],
      3600,
      1n,
      targetContract
    );

    expect(message.allowed_intents).toBe(`0x${"0".repeat(56)}00000009`);
  });

  it("places max amounts at the intent-type indices", () => {
    const message = buildDelegationMessage(
      pubkeyX,
      pubkeyY,
      { lend: 100, swap: 50, withdraw: 7, claim: 0 },
      [],
      3600,
      1n,
      targetContract
    );

    expect(message.max_amounts).toHaveLength(10);
    expect(message.max_amounts[0]).toBe(fieldFromU256(50n)); // swap
    expect(message.max_amounts[3]).toBe(fieldFromU256(100n)); // lend
    expect(message.max_amounts[8]).toBe(fieldFromU256(7n)); // withdraw
    expect(message.max_amounts[9]).toBe(fieldFromU256(0n)); // claim
  });

  it("packs protocol ids into consecutive slots and zero-fills the rest", () => {
    // Aave -> 4, Compound -> 5, Uniswap -> 1
    const message = buildDelegationMessage(
      pubkeyX,
      pubkeyY,
      { lend: 100, swap: 0, withdraw: 0, claim: 0 },
      ["Aave", "Compound", "Uniswap"],
      3600,
      1n,
      targetContract
    );

    expect(message.allowed_protocols).toEqual([
      `0x${"0".repeat(56)}00000004`,
      `0x${"0".repeat(56)}00000005`,
      `0x${"0".repeat(56)}00000001`,
      fieldFromU256(0n),
      fieldFromU256(0n),
    ]);
  });
});
