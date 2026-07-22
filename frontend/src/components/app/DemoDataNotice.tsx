// Shown when the API flags its payload as demonstration data (demo: true /
// X-Demo-Data header, anomaly A2). Reuses the amber "Demo data" styling from
// AppHeader so both demo modes read the same way.
export function DemoDataNotice() {
  return (
    <div
      role="status"
      className="flex items-center gap-2 rounded-xl border border-amber-400/30 bg-amber-400/10 px-4 py-3 text-sm font-medium text-amber-400"
    >
      <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-amber-400" aria-hidden="true" />
      Demo data — the API is currently serving built-in demonstration data for this section.
    </div>
  );
}
