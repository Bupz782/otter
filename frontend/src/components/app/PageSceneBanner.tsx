import type { ReactNode } from "react";
import { PageHeader } from "./PageHeader";

/**
 * Painterly scene banner for app pages: the painting covers the header zone
 * (subject right, dark void left), page title/subtitle overlay the void.
 */
export function PageSceneBanner({
  src,
  title,
  subtitle,
  action,
}: {
  src: string;
  title: string;
  subtitle?: string;
  action?: ReactNode;
}) {
  return (
    <div className="relative overflow-hidden rounded-2xl border border-border/50">
      {/* In-flow image: the painting defines the banner height, never cropped */}
      <img src={src} alt="" aria-hidden="true" className="block w-full" />
      <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-background via-background/55 to-transparent" />
      <div className="absolute inset-0 flex flex-col justify-center px-6">
        <PageHeader title={title} subtitle={subtitle} action={action} />
      </div>
    </div>
  );
}
