export function AmbientBackground() {
  return (
    <div className="pointer-events-none fixed inset-0 z-0 overflow-hidden">
      <div className="animate-ambient-drift absolute -left-[20%] -top-[20%] h-[70vmax] w-[70vmax] rounded-full bg-accent/10 blur-[120px]" />
      <div className="animate-ambient-drift absolute -bottom-[20%] -right-[20%] h-[60vmax] w-[60vmax] rounded-full bg-white/5 blur-[100px] [animation-delay:-7s]" />
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,transparent_0%,#050505_70%)]" />
    </div>
  );
}
