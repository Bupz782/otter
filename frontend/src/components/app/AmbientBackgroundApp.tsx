export function AmbientBackgroundApp() {
  return (
    <div className="pointer-events-none fixed inset-0 z-0 overflow-hidden">
      <div className="animate-ambient-drift absolute -left-[10%] top-0 h-[50vmax] w-[50vmax] rounded-full bg-accent/5 blur-[100px]" />
      <div className="animate-ambient-drift absolute -bottom-[10%] -right-[10%] h-[40vmax] w-[40vmax] rounded-full bg-white/[0.03] blur-[80px] [animation-delay:-7s]" />
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,transparent_0%,var(--otter-black)_80%)]" />
    </div>
  );
}
