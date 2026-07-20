import { describe, it, expect } from "vitest";
import { normalizeStatus, getStatusPresentation, STATUS_PRESENTATION } from "@/lib/status";

describe("normalizeStatus", () => {
  it("normalizes pending to monitoring", () => {
    expect(normalizeStatus("pending")).toBe("monitoring");
  });

  it("passes through every known status unchanged", () => {
    for (const status of Object.keys(STATUS_PRESENTATION)) {
      expect(normalizeStatus(status)).toBe(status);
    }
  });

  it("falls back to monitoring for unknown statuses", () => {
    expect(normalizeStatus("exploded")).toBe("monitoring");
    expect(normalizeStatus("")).toBe("monitoring");
  });
});

describe("getStatusPresentation", () => {
  it("distinguishes in-flight statuses from terminal ones", () => {
    for (const status of ["monitoring", "condition_met", "proving", "submitted"] as const) {
      expect(getStatusPresentation(status).active).toBe(true);
    }
    for (const status of ["confirmed", "failed", "revoked"] as const) {
      expect(getStatusPresentation(status).active).toBe(false);
    }
  });

  it("maps confirmed to the emerald positive tone", () => {
    const presentation = getStatusPresentation("confirmed");

    expect(presentation.tone).toBe("emerald");
    expect(presentation.label).toBe("Confirmed");
  });
});
