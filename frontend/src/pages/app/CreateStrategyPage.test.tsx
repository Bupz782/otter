import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { CreateStrategyPage } from "./CreateStrategyPage";

// The page data hooks hit the API; the form contract does not need them.
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

describe("CreateStrategyPage", () => {
  it("renders no agent picker and no risk profile selector", () => {
    renderPage();

    // Single protocol-operated agent: the user publishes an intent template,
    // risk is bounded by their signed delegation, never by a label here.
    expect(screen.queryByText("Agent")).not.toBeInTheDocument();
    expect(screen.queryByText("Risk profile")).not.toBeInTheDocument();
    expect(screen.queryByRole("radio")).not.toBeInTheDocument();
  });

  it("keeps publish disabled until the intent text is parsed", () => {
    renderPage();

    expect(screen.getByRole("button", { name: /publish strategy/i })).toBeDisabled();
  });
});
