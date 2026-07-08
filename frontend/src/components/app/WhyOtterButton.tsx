import { Sparkles } from "lucide-react";
import { Button } from "@/components/ui/button";

export function WhyOtterButton({ onClick }: { onClick: () => void }) {
  return (
    <Button
      variant="outline"
      size="sm"
      onClick={onClick}
      className="hidden rounded-full border-accent/40 text-accent hover:bg-accent-subtle sm:inline-flex"
    >
      <Sparkles className="mr-2 h-3.5 w-3.5" />
      Why Otter?
    </Button>
  );
}
