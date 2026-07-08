import { useEffect, useState } from "react";
import { createPortal } from "react-dom";

export function Spotlight({ targetId }: { targetId: string }) {
  const [rect, setRect] = useState<DOMRect | null>(null);

  useEffect(() => {
    const update = () => {
      const target = document.getElementById(targetId);
      if (!target) return;
      setRect(target.getBoundingClientRect());
    };
    update();
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  }, [targetId]);

  if (!rect) return null;

  const padding = 8;
  const style: React.CSSProperties = {
    position: "fixed",
    top: rect.top - padding,
    left: rect.left - padding,
    width: rect.width + padding * 2,
    height: rect.height + padding * 2,
    zIndex: 125,
    pointerEvents: "none",
  };

  return createPortal(
    <div style={style} className="rounded-xl">
      <div className="animate-spotlight-rotate absolute -inset-px rounded-xl bg-[conic-gradient(from_0deg,transparent_0_340deg,var(--otter-amber-300)_360deg)] opacity-60" />
      <div className="absolute inset-[2px] rounded-xl bg-background/20 backdrop-blur-[1px]" />
    </div>,
    document.body
  );
}
