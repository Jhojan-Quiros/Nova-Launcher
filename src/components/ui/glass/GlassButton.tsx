import React from "react";
import { cn } from "@/utils/cn";

export interface GlassButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "danger" | "ghost";
  size?: "sm" | "md" | "lg" | "icon";
  isLoading?: boolean;
}

export const GlassButton = React.forwardRef<HTMLButtonElement, GlassButtonProps>(
  (
    {
      className,
      variant = "secondary",
      size = "md",
      isLoading = false,
      disabled,
      children,
      ...props
    },
    ref
  ) => {
    return (
      <button
        ref={ref}
        disabled={disabled || isLoading}
        className={cn(
          "inline-flex items-center justify-center font-medium select-none transition-all duration-150 outline-none",
          "active:scale-[0.97] disabled:opacity-50 disabled:pointer-events-none disabled:active:scale-100",
          // Sizes
          size === "sm" && "h-8 px-3 text-xs rounded-xl gap-1.5",
          size === "md" && "h-10 px-4 text-sm rounded-xl gap-2",
          size === "lg" && "h-12 px-6 text-base rounded-2xl gap-2.5 font-semibold",
          size === "icon" && "h-9 w-9 rounded-xl",
          // Variants
          variant === "primary" &&
            "bg-blue-600 hover:bg-blue-500 text-white shadow-glow-accent border border-blue-400/40 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.35)]",
          variant === "secondary" &&
            "bg-white/[0.07] hover:bg-white/[0.12] text-slate-100 border border-white/10 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.12)] backdrop-blur-md",
          variant === "danger" &&
            "bg-red-500/20 hover:bg-red-500/30 text-red-300 border border-red-500/30 shadow-[inset_0_1px_0_0_rgba(239,68,68,0.2)]",
          variant === "ghost" &&
            "bg-transparent hover:bg-white/[0.06] text-slate-300 hover:text-white border border-transparent",
          className
        )}
        {...props}
      >
        {isLoading ? (
          <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
        ) : (
          children
        )}
      </button>
    );
  }
);
GlassButton.displayName = "GlassButton";