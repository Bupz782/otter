import type { ReactNode } from "react";
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { AppLayout } from "./AppLayout";

// Keep the layout test focused on keyboard chrome (skip link, drawer,
// focus trap): child sections pull in wagmi/RainbowKit and are mocked.
vi.mock("./AmbientBackgroundApp", () => ({
  AmbientBackgroundApp: () => null,
}));
vi.mock("./OnboardingProvider", () => ({
  OnboardingProvider: ({ children }: { children: ReactNode }) => <>{children}</>,
}));
vi.mock("./AppSidebar", () => ({
  AppSidebar: () => <nav aria-label="App navigation" />,
}));
vi.mock("./AppHeader", () => ({
  AppHeader: ({ onMenuClick }: { onMenuClick?: () => void }) => (
    <button type="button" onClick={onMenuClick}>
      Open navigation
    </button>
  ),
}));

function renderLayout() {
  return render(
    <MemoryRouter initialEntries={["/app/dashboard"]}>
      <Routes>
        <Route path="/app" element={<AppLayout />}>
          <Route path="dashboard" element={<p>Page content</p>} />
        </Route>
      </Routes>
    </MemoryRouter>
  );
}

describe("AppLayout keyboard accessibility", () => {
  it("has a skip link pointing to an existing #main-content element", () => {
    renderLayout();

    const skipLink = screen.getByRole("link", { name: /skip to main content/i });
    expect(skipLink).toHaveAttribute("href", "#main-content");
    expect(document.getElementById("main-content")).not.toBeNull();
  });

  it("contains no positive tabIndex that would break the tab order", () => {
    const { container } = renderLayout();

    const positiveTabIndex = Array.from(container.querySelectorAll("[tabindex]")).filter(
      (el) => Number(el.getAttribute("tabindex")) > 0
    );
    expect(positiveTabIndex).toHaveLength(0);
  });

  it("moves focus inside the mobile drawer when it opens", () => {
    renderLayout();

    fireEvent.click(screen.getByRole("button", { name: /open navigation/i }));

    const dialog = screen.getByRole("dialog", { name: /navigation/i });
    expect(dialog.contains(document.activeElement)).toBe(true);
  });

  it("closes the mobile drawer on Escape", () => {
    renderLayout();

    fireEvent.click(screen.getByRole("button", { name: /open navigation/i }));
    expect(screen.getByRole("dialog", { name: /navigation/i })).toBeInTheDocument();

    fireEvent.keyDown(document, { key: "Escape" });
    expect(screen.queryByRole("dialog", { name: /navigation/i })).not.toBeInTheDocument();
  });
});
