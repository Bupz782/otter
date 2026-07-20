import { describe, it, expect } from "vitest";
import { renderHook } from "@testing-library/react";
import { useDocumentTitle } from "./useDocumentTitle";

describe("useDocumentTitle", () => {
  it("sets document.title with the Otter suffix", () => {
    renderHook(() => useDocumentTitle("Dashboard"));

    expect(document.title).toBe("Dashboard — Otter");
  });

  it("updates the title when the page name changes", () => {
    const { rerender } = renderHook(({ title }) => useDocumentTitle(title), {
      initialProps: { title: "Intents" },
    });

    expect(document.title).toBe("Intents — Otter");

    rerender({ title: "Proofs" });

    expect(document.title).toBe("Proofs — Otter");
  });
});
