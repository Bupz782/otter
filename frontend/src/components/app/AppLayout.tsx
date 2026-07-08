import { useState } from "react";
import { Outlet } from "react-router-dom";
import { AppSidebar } from "./AppSidebar";
import { AppHeader } from "./AppHeader";
import { AmbientBackgroundApp } from "./AmbientBackgroundApp";
import { OnboardingProvider } from "./OnboardingProvider";

export function AppLayout() {
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <OnboardingProvider>
      <div className="relative flex min-h-screen bg-background text-foreground">
        <AmbientBackgroundApp />
        <AppSidebar />

        {mobileOpen && (
          <div className="fixed inset-0 z-50 flex md:hidden">
            <div
              className="flex-1 bg-background/80 backdrop-blur-sm"
              onClick={() => setMobileOpen(false)}
              aria-hidden="true"
            />
            <div className="w-64">
              <AppSidebar mobile />
            </div>
          </div>
        )}

        <div className="flex flex-1 flex-col md:ml-64">
          <AppHeader onMenuClick={() => setMobileOpen(true)} />
          <main className="flex-1 p-4 md:p-8">
            <Outlet />
          </main>
        </div>
      </div>
    </OnboardingProvider>
  );
}
