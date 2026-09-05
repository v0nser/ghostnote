import { type ReactNode } from "react";

import { cn } from "@/lib/utils";

export function TiltCard({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "relative overflow-hidden rounded-3xl border border-white/10 bg-charcoal p-6",
        className,
      )}
    >
      {children}
    </div>
  );
}
