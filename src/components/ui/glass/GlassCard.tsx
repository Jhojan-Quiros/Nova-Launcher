import React from "react";
import { cn } from "@/utils/cn";

export interface GlassCardProps extends React.HTMLAttributes<HTMLDivElement> {
  interactive?: boolean;
  active?: boolean;
}

export const GlassCard = React.forwardRef<HTMLDivElement, GlassCardProps>(
  ({ className, interactive = true, active = false, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          "relative overflow-hidden rounded-2xl p-4 transition-all duration-200",
          "bg-white/[0.04] backdrop-blur-xl border border-white/10 shadow-glass-sm shadow-[inset_0_1px_0_0_rgba(255,255,255,0.09)]",
          interactive &&
            "hover:bg-white/[0.08] hover:border-white/20 hover:-translate-y-0.5 hover:shadow-lg cursor-pointer active:scale-[0.98]",
          active &&
            "border-blue-500/60 bg-blue-500/10 shadow-[0_0_24px_-4px_rgba(59,130,246,0.3)] shadow-[inset_0_1px_0_0_rgba(96,165,250,0.3)]",
          className
        )}
        {...props}
      >
        {children}
      </div>
    );
  }
);
GlassCard.displayName = "GlassCard";