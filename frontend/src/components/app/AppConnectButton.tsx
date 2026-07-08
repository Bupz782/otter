import { useState } from "react";
import { ConnectButton } from "@rainbow-me/rainbowkit";
import { useAccount, useSignMessage } from "wagmi";
import { Button } from "@/components/ui/button";
import { api, setAuthToken } from "@/lib/api";
import { Loader2, ShieldCheck } from "lucide-react";

export function AppConnectButton() {
  const { address, isConnected } = useAccount();
  const { signMessageAsync } = useSignMessage();
  const [authLoading, setAuthLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [authenticated, setAuthenticated] = useState(() => !!localStorage.getItem("otter_token"));

  const handleAuth = async () => {
    if (!address) return;
    setAuthLoading(true);
    setError(null);
    try {
      const { message } = await api.auth.challenge(address);
      const signature = await signMessageAsync({ message });
      const { token } = await api.auth.verify(message, signature);
      setAuthToken(token);
      setAuthenticated(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Authentication failed");
    } finally {
      setAuthLoading(false);
    }
  };

  if (!isConnected) {
    return <ConnectButton accountStatus="address" chainStatus="icon" showBalance={false} />;
  }

  if (!authenticated) {
    return (
      <Button
        size="sm"
        onClick={handleAuth}
        disabled={authLoading}
        className="rounded-full"
      >
        {authLoading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <ShieldCheck className="mr-2 h-4 w-4" />}
        Sign In
      </Button>
    );
  }

  return <ConnectButton accountStatus="address" chainStatus="icon" showBalance={false} />;
}
