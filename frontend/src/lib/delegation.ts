import { hexToBytes, toHex } from "viem";

export const INTENT_TYPE_COUNT = 10;
export const ALLOWED_PROTOCOL_COUNT = 5;

export const INTENT_TYPE_INDICES: Record<string, number> = {
  swap: 0,
  stake: 1,
  borrow: 2,
  lend: 3,
  // Frontend-only action types stored in reserved slots until the circuit supports them.
  withdraw: 8,
  claim: 9,
};

export const PROTOCOL_INDICES: Record<string, number> = {
  Uniswap: 1,
  Sushiswap: 2,
  Balancer: 3,
  Aave: 4,
  Compound: 5,
};

export function fieldFromU256(value: bigint): string {
  return toHex(value, { size: 32 });
}

export function fieldFromU128(value: bigint): string {
  const bytes = new Uint8Array(32);
  const valueBytes = bigIntToBeBytes(value, 16);
  bytes.set(valueBytes, 16);
  return toHex(bytes, { size: 32 });
}

export function fieldFromU64(value: bigint): string {
  const bytes = new Uint8Array(32);
  const valueBytes = bigIntToBeBytes(value, 8);
  bytes.set(valueBytes, 24);
  return toHex(bytes, { size: 32 });
}

export function fieldFromU32(value: number): string {
  const bytes = new Uint8Array(32);
  const valueBytes = bigIntToBeBytes(BigInt(value), 4);
  bytes.set(valueBytes, 28);
  return toHex(bytes, { size: 32 });
}

export function fieldFromU8(value: number): string {
  const bytes = new Uint8Array(32);
  bytes[31] = value;
  return toHex(bytes, { size: 32 });
}

export function padHex32(value: string): string {
  return toHex(hexToBytes(value as `0x${string}`), { size: 32 });
}

function bigIntToBeBytes(value: bigint, length: number): Uint8Array {
  const hex = value.toString(16).padStart(length * 2, "0");
  return hexToBytes(`0x${hex}`);
}

export interface DelegationMessage {
  pubkey_x: string;
  pubkey_y: string;
  allowed_intents: string;
  max_amounts: string[];
  allowed_protocols: string[];
  expiry: string;
  nonce: string;
  target_contract: string;
}

export interface DelegationLimits {
  lend: number;
  swap: number;
  withdraw: number;
  claim: number;
}

export function buildDelegationMessage(
  pubkeyX: string,
  pubkeyY: string,
  limits: DelegationLimits,
  allowedProtocols: string[],
  expirySeconds: number,
  nonce: bigint,
  targetContract: string
): DelegationMessage {
  const allowedIntentsBitmap = Object.entries(INTENT_TYPE_INDICES).reduce(
    (bitmap, [key, index]) => {
      if (key in limits && limits[key as keyof DelegationLimits] > 0) {
        bitmap |= 1 << index;
      }
      return bitmap;
    },
    0
  );

  const maxAmounts: string[] = Array(INTENT_TYPE_COUNT).fill(fieldFromU256(0n));
  for (const [key, index] of Object.entries(INTENT_TYPE_INDICES)) {
    if (index < INTENT_TYPE_COUNT) {
      maxAmounts[index] = fieldFromU256(BigInt(limits[key as keyof DelegationLimits] ?? 0));
    }
  }

  const allowedProtocolsArray: string[] = Array(ALLOWED_PROTOCOL_COUNT).fill(fieldFromU256(0n));
  for (const protocol of allowedProtocols) {
    const index = PROTOCOL_INDICES[protocol];
    if (index !== undefined) {
      allowedProtocolsArray[index] = padHex32(
        "0x000000000000000000000000" + protocol.toLowerCase().padEnd(8, "0")
      );
    }
  }

  return {
    pubkey_x: padHex32(pubkeyX),
    pubkey_y: padHex32(pubkeyY),
    allowed_intents: fieldFromU8(allowedIntentsBitmap),
    max_amounts: maxAmounts,
    allowed_protocols: allowedProtocolsArray,
    expiry: fieldFromU64(BigInt(expirySeconds)),
    nonce: fieldFromU256(nonce),
    target_contract: padHex32(targetContract),
  };
}

export function splitSignatureForBackend(signature: `0x${string}`): string[] {
  // Backend expects 64 1-byte hex strings (r || s, no recovery id).
  const bytes = hexToBytes(signature);
  const sig64 = bytes.length === 65 ? bytes.slice(0, 64) : bytes;
  if (sig64.length !== 64) {
    throw new Error(`Expected 64-byte signature, got ${sig64.length}`);
  }
  return Array.from(sig64).map((b) => `0x${b.toString(16).padStart(2, "0")}`);
}

export function generateNonce(): bigint {
  const bytes = crypto.getRandomValues(new Uint8Array(32));
  let value = 0n;
  for (const byte of bytes) {
    value = (value << 8n) | BigInt(byte);
  }
  return value;
}
