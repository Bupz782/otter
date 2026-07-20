import type { LucideIcon } from "lucide-react";
import { Search, CheckCircle2, Loader2, Clock, XCircle } from "lucide-react";
import type { IntentStatus } from "@/types/app";

/**
 * Single source of truth for intent status presentation.
 * Consumed by StatusOrb, IntentStatusBadge, and KineticTimeline.
 *
 * Color mapping (DESIGN.md palette: accent #c8a46c, emerald, rose, amber):
 * - confirmed -> emerald (positive terminal state)
 * - monitoring -> amber (waiting, watching; "pending" normalizes here)
 * - submitted -> accent
 * - condition_met -> accent
 * - proving -> accent (in-flight work under Otter control, like submitted and
 *   condition_met; amber stays reserved for the monitoring wait state)
 * - failed -> rose (negative terminal state)
 * - revoked -> muted (neutral terminal state, outside the brief's mapping)
 */
export interface StatusPresentation {
  label: string;
  tone: "accent" | "emerald" | "amber" | "rose" | "muted";
  /** Solid dot color (StatusOrb). */
  dotClass: string;
  /** Subtle tinted badge surface (IntentStatusBadge). */
  badgeClass: string;
  /** Status-colored text. */
  textClass: string;
  /** True while the intent is in flight (pulse cue on the orb). */
  active: boolean;
  icon: LucideIcon;
}

const ACCENT_BADGE = "border-accent/30 bg-accent-subtle text-accent";

export const STATUS_PRESENTATION: Record<IntentStatus, StatusPresentation> = {
  monitoring: {
    label: "Monitoring",
    tone: "amber",
    dotClass: "bg-amber-400",
    badgeClass: "border-amber-400/30 bg-amber-400/10 text-amber-400",
    textClass: "text-amber-400",
    active: true,
    icon: Search,
  },
  condition_met: {
    label: "Condition Met",
    tone: "accent",
    dotClass: "bg-accent",
    badgeClass: ACCENT_BADGE,
    textClass: "text-accent",
    active: true,
    icon: CheckCircle2,
  },
  proving: {
    label: "Proving",
    tone: "accent",
    dotClass: "bg-accent",
    badgeClass: ACCENT_BADGE,
    textClass: "text-accent",
    active: true,
    icon: Loader2,
  },
  submitted: {
    label: "Submitted",
    tone: "accent",
    dotClass: "bg-accent",
    badgeClass: ACCENT_BADGE,
    textClass: "text-accent",
    active: true,
    icon: Clock,
  },
  confirmed: {
    label: "Confirmed",
    tone: "emerald",
    dotClass: "bg-emerald-400",
    badgeClass: "border-emerald-400/30 bg-emerald-400/10 text-emerald-400",
    textClass: "text-emerald-400",
    active: false,
    icon: CheckCircle2,
  },
  failed: {
    label: "Failed",
    tone: "rose",
    dotClass: "bg-rose-400",
    badgeClass: "border-rose-400/30 bg-rose-400/10 text-rose-400",
    textClass: "text-rose-400",
    active: false,
    icon: XCircle,
  },
  revoked: {
    label: "Revoked",
    tone: "muted",
    dotClass: "bg-muted-foreground",
    badgeClass: "border-border bg-secondary text-muted-foreground",
    textClass: "text-muted-foreground",
    active: false,
    icon: XCircle,
  },
};

const VALID_STATUSES = new Set<string>(Object.keys(STATUS_PRESENTATION));

/**
 * Normalize a raw status string to a known IntentStatus.
 * "pending" is treated like "monitoring" (both mean waiting on a condition).
 * Unknown values fall back to "monitoring", the safest non-terminal state.
 */
export function normalizeStatus(status: string): IntentStatus {
  if (status === "pending") return "monitoring";
  if (VALID_STATUSES.has(status)) return status as IntentStatus;
  return "monitoring";
}

export function getStatusPresentation(status: IntentStatus | string): StatusPresentation {
  return STATUS_PRESENTATION[normalizeStatus(status)];
}
