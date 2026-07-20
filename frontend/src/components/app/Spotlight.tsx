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

  const padding = 6;
  const style: React.CSSProperties = {
    position: "fixed",
    top: rect.top - padding,
    left: rect.left - padding,
    width: rect.width + padding * 2,
    height: rect.height + padding * 2,
    zIndex: 125,
    pointerEvents: "none",
    borderRadius: 16,
  };

  return createPortal(
    <div style={style} className="rounded-2xl ring-2 ring-accent/60 ring-offset-0" />,
    document.body
  );
}
