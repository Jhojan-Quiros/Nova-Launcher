import React from "react";
import { cn } from "@/utils/cn";

export interface GlassBadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: "default" | "success" | "warning" | "danger" | "info";
  size?: "sm" | "md";
}

export const GlassBadge: React.FC<GlassBadgeProps> = ({
  className,
  variant = "default",
  size = "sm",
  children,
  ...props
}) => {
  return (
    <span
      className={cn(
        "inline-flex items-center font-medium rounded-full border backdrop-blur-md",
        size === "sm" && "px-2.5 py-0.5 text-[11px]",
        size === "md" && "px-3 py-1 text-xs",
        variant === "default" && "bg-white/10 text-slate-300 border-white/10",
        variant === "success" && "bg-emerald-500/15 text-emerald-400 border-emerald-500/30",
        variant === "warning" && "bg-amber-500/15 text-amber-400 border-amber-500/30",
        variant === "danger" && "bg-rose-500/15 text-rose-400 border-rose-500/30",
        variant === "info" && "bg-blue-500/15 text-blue-400 border-blue-500/30",
        className
      )}
      {...props}
    >
      {children}
    </span>
  );
};