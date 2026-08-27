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
  Coins,
  Zap,
  Package,
  ArrowRightLeft,
  Globe,
} from "lucide-react";
import { useAccount } from "wagmi";
import { cn, truncateHash } from "@/lib/utils";
import { useAuthToken } from "@/hooks/useAuthToken";

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
      { to: "/app/dashboard", label: "Dashboard", icon: <LayoutDashboard className="h-4 w-4" /> },
      {
        to: "/app/intents",
        label: "Intents",
        icon: <Lightbulb className="h-4 w-4" />,
        children: [{ to: "/app/intents/new", label: "Create", icon: <Plus className="h-4 w-4" /> }],
      },
      {
        to: "/app/delegations",
        label: "Delegations",
        icon: <FileSignature className="h-4 w-4" />,
        children: [
          { to: "/app/delegations/new", label: "New", icon: <Plus className="h-4 w-4" /> },
        ],
      },
    ],
  },
  {
    label: "Discover",
    items: [
      { to: "/app/agents", label: "Otter Agents", icon: <Bot className="h-4 w-4" /> },
      { to: "/app/strategies", label: "Strategies", icon: <BookOpen className="h-4 w-4" /> },
    ],
  },
  {
    label: "Verify",
    items: [
      { to: "/app/proofs", label: "Proofs", icon: <ShieldCheck className="h-4 w-4" /> },
      { to: "/app/solvency", label: "Solvency", icon: <Coins className="h-4 w-4" /> },
      { to: "/app/rebates", label: "Rebates", icon: <Zap className="h-4 w-4" /> },
      { to: "/app/mev", label: "MEV", icon: <Package className="h-4 w-4" /> },
    ],
  },
  {
    label: "Cross-chain",
    items: [
      { to: "/app/bridge", label: "Bridge", icon: <ArrowRightLeft className="h-4 w-4" /> },
      { to: "/app/solana", label: "Solana", icon: <Globe className="h-4 w-4" /> },
    ],
  },
  {
    label: "System",
    items: [{ to: "/app/settings", label: "Settings", icon: <Settings className="h-4 w-4" /> }],
  },
];

function SidebarNavItem({ item }: { item: NavItem }) {
  const location = useLocation();
  const active = location.pathname === item.to || location.pathname.startsWith(`${item.to}/`);

  return (
    <div key={item.to}>
      <NavLink
        to={item.to}
        className={cn(
          "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent",
          active
            ? "bg-accent-subtle text-accent shadow-[inset_2px_0_0_0_var(--color-accent)]"
            : "text-muted-foreground hover:bg-secondary/50 hover:text-foreground"
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
                "flex items-center gap-2 rounded-md px-3 py-2 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent",
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

function WalletStatusRow() {
  const { address, isConnected } = useAccount();
  const { isAuthenticated } = useAuthToken();

  const { dotClass, label, hint } = !isConnected
    ? {
        dotClass: "bg-muted-foreground",
        label: "Not connected",
        hint: "Connect a wallet to go live.",
      }
    : isAuthenticated
      ? {
          dotClass: "bg-emerald-400",
          label: address ? truncateHash(address) : "Connected",
          hint: "Wallet connected and signed in.",
        }
      : {
          dotClass: "bg-amber-400",
          label: address ? truncateHash(address) : "Connected",
          hint: "Wallet connected. Sign in from the header to go live.",
        };

  return (
    <div className="border-t border-border/50 px-3 py-4">
      <div
        className="flex items-center gap-2.5 rounded-lg px-3 py-2 text-xs text-muted-foreground"
        title={hint}
      >
        <span className={cn("h-1.5 w-1.5 shrink-0 rounded-full", dotClass)} aria-hidden="true" />
        <span className="truncate font-medium">{label}</span>
      </div>
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
      <div className="flex h-14 items-center border-b border-border/50 px-6">
        <NavLink
          to="/app/dashboard"
          className="flex items-center gap-1 rounded font-heading text-xl font-bold tracking-tight text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent"
        >
          otter
          <span className="mt-2 h-1.5 w-1.5 rounded-full bg-accent" aria-hidden="true" />
        </NavLink>
      </div>

      <nav className="flex-1 space-y-6 overflow-y-auto px-3 py-5" aria-label="App navigation">
        {navGroups.map((group) => (
          <div key={group.label}>
            <p className="mb-2 px-3 text-[11px] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
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

      <WalletStatusRow />
    </aside>
  );
}
