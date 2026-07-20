import { useEffect, useState, Suspense } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { X } from "lucide-react";
import { AppSidebar } from "./AppSidebar";
import { AppHeader } from "./AppHeader";
import { AmbientBackgroundApp } from "./AmbientBackgroundApp";
import { OnboardingProvider } from "./OnboardingProvider";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";

function RouteFallback() {
  return (
    <div className="space-y-6" aria-busy="true" aria-label="Loading page">
      <div className="space-y-2">
        <Skeleton className="h-9 w-48" />
        <Skeleton className="h-4 w-64" />
      </div>
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Skeleton className="h-28 w-full rounded-2xl" />
        <Skeleton className="h-28 w-full rounded-2xl" />
        <Skeleton className="h-28 w-full rounded-2xl" />
        <Skeleton className="h-28 w-full rounded-2xl" />
      </div>
      <Skeleton className="h-64 w-full rounded-2xl" />
    </div>
  );
}

export function AppLayout() {
  const [mobileOpen, setMobileOpen] = useState(false);
  const location = useLocation();

  // Close the mobile drawer whenever the route changes.
  useEffect(() => {
    setMobileOpen(false);
  }, [location.pathname]);

  useEffect(() => {
    if (!mobileOpen) return;
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setMobileOpen(false);
    };
    document.addEventListener("keydown", handleKey);
    return () => document.removeEventListener("keydown", handleKey);
  }, [mobileOpen]);

  return (
    <OnboardingProvider>
      <div className="relative flex min-h-screen bg-background text-foreground">
        <AmbientBackgroundApp />
        <AppSidebar />

        {mobileOpen && (
          <div
            className="fixed inset-0 z-50 flex md:hidden"
            role="dialog"
            aria-modal="true"
            aria-label="Navigation"
          >
            <div
              className="flex-1 bg-background/80 backdrop-blur-sm"
              onClick={() => setMobileOpen(false)}
              aria-hidden="true"
            />
            <div className="relative w-64">
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setMobileOpen(false)}
                aria-label="Close navigation"
                className="absolute right-2 top-3 z-10"
              >
                <X className="h-5 w-5" />
              </Button>
              <AppSidebar mobile />
            </div>
          </div>
        )}

        <div className="relative z-10 flex flex-1 flex-col md:ml-64">
          <AppHeader onMenuClick={() => setMobileOpen(true)} />
          <main className="mx-auto w-full max-w-6xl flex-1 px-4 py-8 md:px-8">
            <Suspense fallback={<RouteFallback />}>
              <Outlet />
            </Suspense>
          </main>
        </div>
      </div>
    </OnboardingProvider>
  );
}
