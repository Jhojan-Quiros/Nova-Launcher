import React, { useEffect } from "react";
import { X } from "lucide-react";
import { cn } from "@/utils/cn";
import { GlassButton } from "./GlassButton";

export interface GlassModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  description?: string;
  children: React.ReactNode;
  maxWidth?: "sm" | "md" | "lg" | "xl";
}

export const GlassModal: React.FC<GlassModalProps> = ({
  isOpen,
  onClose,
  title,
  description,
  children,
  maxWidth = "md",
}) => {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/60 backdrop-blur-md transition-opacity animate-in fade-in duration-200"
        onClick={onClose}
      />

      {/* Modal dialog */}
      <div
        className={cn(
          "relative z-10 w-full overflow-hidden rounded-3xl",
          "bg-[#11131c]/90 border border-white/15 backdrop-blur-2xl shadow-2xl shadow-black/80 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.15)]",
          "animate-in zoom-in-95 duration-200",
          maxWidth === "sm" && "max-w-sm",
          maxWidth === "md" && "max-w-lg",
          maxWidth === "lg" && "max-w-2xl",
          maxWidth === "xl" && "max-w-4xl"
        )}
      >
        <div className="flex items-center justify-between border-b border-white/10 px-6 py-5">
          <div>
            <h3 className="text-lg font-semibold text-white tracking-tight">{title}</h3>
            {description && (
              <p className="mt-0.5 text-xs text-slate-400">{description}</p>
            )}
          </div>
          <GlassButton
            variant="ghost"
            size="icon"
            onClick={onClose}
            aria-label="Close"
            className="text-slate-400 hover:text-white"
          >
            <X className="h-4 w-4" />
          </GlassButton>
        </div>

        <div className="p-6 max-h-[75vh] overflow-y-auto">{children}</div>
      </div>
    </div>
  );
};