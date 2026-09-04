import React from "react";
import { NavLink } from "react-router-dom";
import {
  Home,
  Layers,
  Puzzle,
  Package,
  Download,
  Settings,
  Terminal,
  ChevronLeft,
  ChevronRight,
  Sparkles,
} from "lucide-react";
import { cn } from "@/utils/cn";
import { useAppStore } from "@/store/useAppStore";
import { useDownloadStore } from "@/store/useDownloadStore";

interface NavItemProps {
  to: string;
  icon: React.ElementType;
  label: string;
  badge?: string | number;
  comingSoon?: boolean;
  collapsed: boolean;
}

const NavItem: React.FC<NavItemProps> = ({
  to,
  icon: Icon,
  label,
  badge,
  comingSoon = false,
  collapsed,
}) => {
  if (comingSoon) {
    return (
      <div
        className={cn(
          "flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-sm font-medium select-none opacity-40 cursor-not-allowed",
          collapsed && "justify-center px-2"
        )}
        title={`${label} (Coming Soon)`}
      >
        <Icon className="h-4 w-4 shrink-0" />
        {!collapsed && (
          <div className="flex flex-1 items-center justify-between overflow-hidden">
            <span className="truncate">{label}</span>
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-white/10 text-slate-400">Soon</span>
          </div>
        )}
      </div>
    );
  }

  return (
    <NavLink
      to={to}
      className={({ isActive }) =>
        cn(
          "flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-sm font-medium transition-all duration-150 outline-none select-none",
          isActive
            ? "bg-blue-600/20 text-blue-400 border border-blue-500/30 shadow-[inset_0_1px_0_0_rgba(96,165,250,0.2)] shadow-glow-accent"
            : "text-slate-400 hover:text-slate-100 hover:bg-white/[0.06] border border-transparent",
          collapsed && "justify-center px-2"
        )
      }
      title={collapsed ? label : undefined}
    >
      <Icon className="h-4 w-4 shrink-0" />
      {!collapsed && (
        <div className="flex flex-1 items-center justify-between overflow-hidden">
          <span className="truncate">{label}</span>
          {badge !== undefined && (
            <span className="text-[11px] font-semibold px-2 py-0.5 rounded-full bg-blue-500/20 text-blue-300 border border-blue-500/30">
              {badge}
            </span>
          )}
        </div>
      )}
    </NavLink>
  );
};

export const Sidebar: React.FC = () => {
  const { sidebarCollapsed, toggleSidebar } = useAppStore();
  const isDownloading = useDownloadStore((s) => s.isDownloading);

  return (
    <aside
      className={cn(
        "relative flex flex-col justify-between h-full transition-all duration-300 ease-in-out select-none",
        "bg-[#0e1017]/80 backdrop-blur-2xl border-r border-white/10 shadow-2xl z-30",
        sidebarCollapsed ? "w-16 p-2.5" : "w-64 p-4"
      )}
    >
      {/* Top section */}
      <div className="space-y-6">
        {/* Brand header */}
        <div className="flex items-center justify-between px-1">
          <div className="flex items-center gap-3 overflow-hidden">
            <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-gradient-to-tr from-blue-600 to-indigo-500 shadow-glow-accent border border-blue-400/30 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.4)]">
              <Sparkles className="h-5 w-5 text-white" />
            </div>
            {!sidebarCollapsed && (
              <div className="flex flex-col">
                <span className="text-sm font-bold tracking-tight text-white">NOVA</span>
                <span className="text-[10px] tracking-wider text-slate-400 uppercase font-semibold">Launcher</span>
              </div>
            )}
          </div>
          <button
            onClick={toggleSidebar}
            className="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-white/10 transition-colors"
            aria-label="Toggle sidebar"
          >
            {sidebarCollapsed ? <ChevronRight className="h-4 w-4" /> : <ChevronLeft className="h-4 w-4" />}
          </button>
        </div>

        {/* Navigation list */}
        <nav className="space-y-1">
          <NavItem to="/" icon={Home} label="Home" collapsed={sidebarCollapsed} />
          <NavItem to="/instances" icon={Layers} label="Instances" collapsed={sidebarCollapsed} />
          <NavItem to="/mods" icon={Puzzle} label="Mods" comingSoon collapsed={sidebarCollapsed} />
          <NavItem to="/modpacks" icon={Package} label="Modpacks" comingSoon collapsed={sidebarCollapsed} />
          <NavItem
            to="/downloads"
            icon={Download}
            label="Downloads"
            badge={isDownloading ? "Active" : undefined}
            collapsed={sidebarCollapsed}
          />
        </nav>
      </div>

      {/* Bottom section */}
      <div className="space-y-1 pt-4 border-t border-white/10">
        <NavItem to="/logs" icon={Terminal} label="Logs" collapsed={sidebarCollapsed} />
        <NavItem to="/settings" icon={Settings} label="Settings" collapsed={sidebarCollapsed} />

        {!sidebarCollapsed && (
          <div className="px-3 pt-3 flex items-center justify-between text-[11px] text-slate-500">
            <span>Nova v1.0.0</span>
            <span className="flex items-center gap-1">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-400 animate-pulse" />
              Vanilla Ready
            </span>
          </div>
        )}
      </div>
    </aside>
  );
};