import React from "react";
import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { Header } from "./Header";

export const MainLayout: React.FC = () => {
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-slate-100 antialiased">
      {/* Dynamic ambient glass background illumination */}
      <div className="fixed inset-0 pointer-events-none z-0 overflow-hidden">
        <div className="absolute -top-40 -left-40 h-96 w-96 rounded-full bg-blue-600/15 blur-[128px]" />
        <div className="absolute top-1/2 -right-40 h-[500px] w-[500px] rounded-full bg-indigo-600/10 blur-[140px]" />
        <div className="absolute -bottom-40 left-1/3 h-80 w-80 rounded-full bg-sky-500/10 blur-[120px]" />
      </div>

      {/* Sidebar navigation */}
      <Sidebar />

      {/* Main viewport */}
      <div className="relative flex flex-col flex-1 h-full overflow-hidden z-10">
        <Header />
        <main className="flex-1 overflow-y-auto p-6 md:p-8">
          <Outlet />
        </main>
      </div>
    </div>
  );
};