import React from "react";
import { cn } from "@/utils/cn";

export interface GlassInputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  error?: string;
  label?: string;
}

export const GlassInput = React.forwardRef<HTMLInputElement, GlassInputProps>(
  ({ className, error, label, id, ...props }, ref) => {
    return (
      <div className="w-full space-y-1.5">
        {label && (
          <label htmlFor={id} className="block text-xs font-medium text-slate-300">
            {label}
          </label>
        )}
        <input
          ref={ref}
          id={id}
          className={cn(
            "w-full rounded-xl px-3.5 py-2.5 text-sm text-white placeholder:text-slate-500 transition-all duration-150 outline-none",
            "bg-white/[0.05] border border-white/10 backdrop-blur-md shadow-[inset_0_1px_0_0_rgba(255,255,255,0.06)]",
            "focus:border-blue-500/80 focus:bg-white/[0.08] focus:ring-2 focus:ring-blue-500/20",
            error && "border-red-500/80 focus:border-red-500 focus:ring-red-500/20",
            className
          )}
          {...props}
        />
        {error && <p className="text-xs text-red-400 font-medium">{error}</p>}
      </div>
    );
  }
);
GlassInput.displayName = "GlassInput";