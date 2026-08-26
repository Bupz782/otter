import { useEffect, useState } from "react";
import { ConnectButton } from "@rainbow-me/rainbowkit";
import { useAccount, useSignMessage } from "wagmi";
import { Button } from "@/components/ui/button";
import { api, setAuthTokens } from "@/lib/api";
import { useAuthToken } from "@/hooks/useAuthToken";
import { Loader2, LogOut, ShieldCheck } from "lucide-react";

export function AppConnectButton() {
  const { address, isConnected, status } = useAccount();
  const { signMessageAsync } = useSignMessage();
  const { isAuthenticated: authenticated } = useAuthToken();
  const [authLoading, setAuthLoading] = useState(false);
  const [authError, setAuthError] = useState<string | null>(null);

  // Wallet disconnected: drop the session token too.
  // Keyed on status === "disconnected" so a page reload in "reconnecting"
  // state does not clear the session before the wallet comes back.
  useEffect(() => {
    if (status === "disconnected") {
      setAuthTokens(null, null);
      setAuthError(null);
    }
    }, [status]);

    const handleAuth = async () => {
    if (!address) return;
    setAuthLoading(true);
    setAuthError(null);
    try {
      const { message } = await api.auth.challenge(address);
      const signature = await signMessageAsync({ message });
      const { access_token, refresh_token } = await api.auth.verify(message, signature);
      setAuthTokens(access_token, refresh_token);
    } catch (err) {
      setAuthError(err instanceof Error ? err.message : "Sign-in failed. Try again.");
    } finally {
      setAuthLoading(false);
    }
    };

    const handleSignOut = () => {
    setAuthTokens(null, null);
    setAuthError(null);
    };

  if (!isConnected) {
    return <ConnectButton accountStatus="address" chainStatus="icon" showBalance={false} />;
  }

  if (!authenticated) {
    return (
      <div className="flex items-center gap-2">
        <Button size="sm" onClick={handleAuth} disabled={authLoading} className="rounded-full">
          {authLoading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <ShieldCheck className="mr-2 h-4 w-4" />
          )}
          Sign In
        </Button>
        {authError && (
          <p role="alert" className="max-w-48 text-xs text-rose-400">
            {authError}
          </p>
        )}
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2">
      <ConnectButton accountStatus="address" chainStatus="icon" showBalance={false} />
      <Button
        size="sm"
        variant="ghost"
        onClick={handleSignOut}
        className="rounded-full text-muted-foreground hover:text-foreground"
      >
        <LogOut className="mr-2 h-4 w-4" />
        Sign out
      </Button>
    </div>
  );
}
