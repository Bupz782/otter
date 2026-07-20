import { useEffect } from "react";

/**
 * Sets document.title to "<Page> — Otter" so every routed page has a
 * distinct, meaningful title (RGAA 8.5 / WCAG 2.4.2).
 */
export function useDocumentTitle(title: string) {
  useEffect(() => {
    document.title = `${title} — Otter`;
  }, [title]);
}
