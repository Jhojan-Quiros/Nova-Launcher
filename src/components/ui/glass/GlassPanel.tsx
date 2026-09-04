import React from "react";
import { cn } from "@/utils/cn";

export interface GlassPanelProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: "default" | "elevated" | "subtle";
  blur?: "sm" | "md" | "lg";
}

export const GlassPanel = React.forwardRef<HTMLDivElement, GlassPanelProps>(
  ({ className, variant = "default", blur = "md", children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          "rounded-3xl transition-all duration-200",
          // Blur variants
          blur === "sm" && "backdrop-blur-md",
          blur === "md" && "backdrop-blur-2xl",
          blur === "lg" && "backdrop-blur-3xl",
          // Visual variants
          variant === "default" &&
            "bg-surface/80 border border-white/10 shadow-glass shadow-[inset_0_1px_0_0_rgba(255,255,255,0.12)]",
          variant === "elevated" &&
            "bg-surface-elevated/90 border border-white/15 shadow-2xl shadow-black/50 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.18)]",
          variant === "subtle" &&
            "bg-white/[0.03] border border-white/5 shadow-sm shadow-[inset_0_1px_0_0_rgba(255,255,255,0.06)]",
          className
        )}
        {...props}
      >
        {children}
      </div>
    );
  }
);
GlassPanel.displayName = "GlassPanel";