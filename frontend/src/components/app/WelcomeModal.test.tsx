import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { WelcomeModal } from "./WelcomeModal";

describe("WelcomeModal focus management", () => {
  it("moves focus inside the dialog on open", () => {
    render(<WelcomeModal onStart={vi.fn()} onSkip={vi.fn()} />);

    const dialog = screen.getByRole("dialog");
    expect(dialog.contains(document.activeElement)).toBe(true);
  });

  it("traps Tab within the dialog", () => {
    render(<WelcomeModal onStart={vi.fn()} onSkip={vi.fn()} />);

    const focusable = screen
      .getByRole("dialog")
      .querySelectorAll("button, a[href]");
    const first = focusable[0] as HTMLElement;
    const last = focusable[focusable.length - 1] as HTMLElement;

    // Focus starts on the first focusable element (the close button).
    expect(document.activeElement).toBe(first);

    // Tab on the last element wraps back to the first.
    last.focus();
    fireEvent.keyDown(document, { key: "Tab" });
    expect(document.activeElement).toBe(first);

    // Shift+Tab on the first element wraps to the last.
    fireEvent.keyDown(document, { key: "Tab", shiftKey: true });
    expect(document.activeElement).toBe(last);
  });

  it("restores focus to the trigger when it closes", () => {
    const trigger = document.createElement("button");
    document.body.appendChild(trigger);
    trigger.focus();

    const { unmount } = render(<WelcomeModal onStart={vi.fn()} onSkip={vi.fn()} />);
    expect(document.activeElement).not.toBe(trigger);

    unmount();
    expect(document.activeElement).toBe(trigger);

    trigger.remove();
  });

  it("closes on Escape", () => {
    const onSkip = vi.fn();
    render(<WelcomeModal onStart={vi.fn()} onSkip={onSkip} />);

    fireEvent.keyDown(window, { key: "Escape" });

    expect(onSkip).toHaveBeenCalledTimes(1);
  });
});
