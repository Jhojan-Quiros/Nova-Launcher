import React from "react";
import { cn } from "@/utils/cn";

export interface GlassProgressProps extends React.HTMLAttributes<HTMLDivElement> {
  value: number; // 0 to 100
  max?: number;
  height?: "sm" | "md" | "lg";
}

export const GlassProgress: React.FC<GlassProgressProps> = ({
  className,
  value,
  max = 100,
  height = "md",
  ...props
}) => {
  const percentage = Math.min(Math.max((value / max) * 100, 0), 100);

  return (
    <div
      className={cn(
        "w-full overflow-hidden rounded-full bg-white/[0.08] border border-white/10 backdrop-blur-md p-0.5 shadow-inner",
        height === "sm" && "h-2",
        height === "md" && "h-3",
        height === "lg" && "h-4",
        className
      )}
      {...props}
    >
      <div
        className="h-full rounded-full bg-gradient-to-r from-blue-500 to-indigo-500 transition-all duration-300 ease-out shadow-[0_0_12px_rgba(59,130,246,0.6)]"
        style={{ width: `${percentage}%` }}
      />
    </div>
  );
};