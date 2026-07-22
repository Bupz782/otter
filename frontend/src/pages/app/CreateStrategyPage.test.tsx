import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { CreateStrategyPage } from "./CreateStrategyPage";

// The page data hooks hit the API; the keyboard contract does not need them.
vi.mock("@/hooks/useAgents", () => ({
  useAgents: () => ({ data: [], isLoading: false }),
}));
vi.mock("@/hooks/useCreateStrategy", () => ({
  useCreateStrategy: () => ({ mutate: vi.fn(), isLoading: false, data: null }),
}));
vi.mock("@/hooks/useParseIntent", () => ({
  useParseIntent: () => ({ parse: vi.fn(), isLoading: false, data: null, reset: vi.fn() }),
}));
vi.mock("@/hooks/useDocumentTitle", () => ({
  useDocumentTitle: () => {},
}));

function renderPage() {
  return render(
    <MemoryRouter>
      <CreateStrategyPage />
    </MemoryRouter>
  );
}

describe("CreateStrategyPage keyboard accessibility", () => {
  it("renders the risk profile selector as real buttons with aria-pressed", () => {
    renderPage();

    const balanced = screen.getByRole("button", { name: "Balanced" });
    const conservative = screen.getByRole("button", { name: "Conservative" });

    expect(balanced).toHaveAttribute("aria-pressed", "true");
    expect(conservative).toHaveAttribute("aria-pressed", "false");
  });

  it("updates the selected risk profile on activation", () => {
    renderPage();

    fireEvent.click(screen.getByRole("button", { name: "Advanced" }));

    expect(screen.getByRole("button", { name: "Advanced" })).toHaveAttribute(
      "aria-pressed",
      "true"
    );
    expect(screen.getByRole("button", { name: "Balanced" })).toHaveAttribute(
      "aria-pressed",
      "false"
    );
  });
});
