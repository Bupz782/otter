import { NavLink, useLocation } from "react-router-dom";
import {
  LayoutDashboard,
  Lightbulb,
  FileSignature,
  Bot,
  BookOpen,
  ShieldCheck,
  Settings,
  Plus,
} from "lucide-react";
import { cn } from "@/lib/utils";

interface NavItem {
  to: string;
  label: string;
  icon: React.ReactNode;
  children?: { to: string; label: string; icon?: React.ReactNode }[];
}

interface NavGroup {
  label: string;
  items: NavItem[];
}

const navGroups: NavGroup[] = [
  {
    label: "Manage",
    items: [
      { to: "/app/dashboard", label: "Dashboard", icon: <LayoutDashboard className="h-5 w-5" /> },
      {
        to: "/app/intents",
        label: "Intents",
        icon: <Lightbulb className="h-5 w-5" />,
        children: [{ to: "/app/intents/new", label: "Create", icon: <Plus className="h-4 w-4" /> }],
      },
      {
        to: "/app/delegations",
        label: "Delegations",
        icon: <FileSignature className="h-5 w-5" />,
        children: [{ to: "/app/delegations/new", label: "New", icon: <Plus className="h-4 w-4" /> }],
      },
    ],
  },
  {
    label: "Discover",
    items: [
      { to: "/app/agents", label: "Otter Agents", icon: <Bot className="h-5 w-5" /> },
      { to: "/app/strategies", label: "Strategies", icon: <BookOpen className="h-5 w-5" /> },
    ],
  },
  {
    label: "Verify",
    items: [{ to: "/app/proofs", label: "Proofs", icon: <ShieldCheck className="h-5 w-5" /> }],
  },
  {
    label: "System",
    items: [{ to: "/app/settings", label: "Settings", icon: <Settings className="h-5 w-5" /> }],
  },
];

function SidebarNavItem({ item }: { item: NavItem }) {
  const location = useLocation();
  const active = location.pathname === item.to || location.pathname.startsWith(`${item.to}/`);

  return (
    <div key={item.to}>
      <NavLink
        id={`onboarding-${item.to.replace("/app/", "").replace("/", "-")}`}
        to={item.to}
        className={cn(
          "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors",
          active
            ? "bg-accent-subtle text-accent"
            : "text-muted-foreground hover:bg-secondary hover:text-foreground"
        )}
      >
        {item.icon}
        {item.label}
      </NavLink>
      {item.children && (
        <div className="mt-1 space-y-0.5 pl-10">
          {item.children.map((child) => (
            <NavLink
              key={child.to}
              to={child.to}
              className={cn(
                "flex items-center gap-2 rounded-md px-3 py-2 text-xs font-medium transition-colors",
                location.pathname === child.to
                  ? "text-accent"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              {child.icon}
              {child.label}
            </NavLink>
          ))}
        </div>
      )}
    </div>
  );
}

export function AppSidebar({ mobile = false }: { mobile?: boolean }) {
  return (
    <aside
      className={cn(
        "flex flex-col border-r border-border/50 bg-background/60 backdrop-blur-xl",
        mobile ? "h-full" : "fixed left-0 top-0 hidden h-screen w-64 md:flex"
      )}
    >
      <div className="flex h-16 items-center border-b border-border/50 px-6">
        <NavLink
          to="/app/dashboard"
          className="font-heading text-xl font-bold tracking-tight text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent rounded"
        >
          otter
        </NavLink>
      </div>

      <nav className="flex-1 space-y-6 overflow-y-auto px-3 py-5" aria-label="App navigation">
        {navGroups.map((group) => (
          <div key={group.label}>
            <p className="mb-2 px-3 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/60">
              {group.label}
            </p>
            <div className="space-y-1">
              {group.items.map((item) => (
                <SidebarNavItem key={item.to} item={item} />
              ))}
            </div>
          </div>
        ))}
      </nav>

      <div className="border-t border-border/50 p-4">
        <p className="text-xs text-muted-foreground">Mock mode — no real transactions.</p>
      </div>
    </aside>
  );
}
