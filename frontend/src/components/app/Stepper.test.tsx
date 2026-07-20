import { describe, it, expect } from "vitest";
import { render, screen, within } from "@testing-library/react";
import { Stepper } from "@/components/app/Stepper";

const steps = [
  { label: "Define limits", description: "Set amounts" },
  { label: "Sign", description: "Wallet signature" },
  { label: "Activate", description: "Agent takes over" },
];

describe("Stepper", () => {
  it("renders every step label", () => {
    render(<Stepper steps={steps} currentStep={0} />);

    expect(screen.getByText("Define limits")).toBeInTheDocument();
    expect(screen.getByText("Sign")).toBeInTheDocument();
    expect(screen.getByText("Activate")).toBeInTheDocument();
  });

  it("marks only the current step with aria-current=step", () => {
    render(<Stepper steps={steps} currentStep={1} />);

    expect(screen.getByText("Sign").closest("[aria-current]")).toHaveAttribute(
      "aria-current",
      "step"
    );
    expect(screen.getByText("Define limits").closest("[aria-current]")).toBeNull();
    expect(screen.getByText("Activate").closest("[aria-current]")).toBeNull();
  });

  it("shows a check instead of the number on completed steps", () => {
    render(<Stepper steps={steps} currentStep={2} />);

    const items = screen.getAllByRole("listitem");

    // Completed steps show the Check icon (svg) rather than their number.
    expect(within(items[0]).queryByText("1")).not.toBeInTheDocument();
    expect(items[0].querySelector("svg")).not.toBeNull();
    expect(within(items[1]).queryByText("2")).not.toBeInTheDocument();

    // The current step still shows its number.
    expect(within(items[2]).getByText("3")).toBeInTheDocument();
  });

  it("shows numbers on every step when nothing is completed yet", () => {
    render(<Stepper steps={steps} currentStep={0} />);

    const items = screen.getAllByRole("listitem");

    expect(within(items[0]).getByText("1")).toBeInTheDocument();
    expect(within(items[1]).getByText("2")).toBeInTheDocument();
    expect(within(items[2]).getByText("3")).toBeInTheDocument();
  });
});
