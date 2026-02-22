import * as React from "react";
import { Button } from "@/components/ui/button";

interface LeftSidebarProps {
  isMobile?: boolean;
  onToggle?: () => void;
}

export function LeftSidebar({ isMobile, onToggle }: LeftSidebarProps) {
  return (
    <aside className="w-full h-full flex flex-col bg-bg-sources border-r border-border-bold">
      <div className="p-5 flex items-center justify-between border-b border-border-bold bg-bg-sources">
        <h2 className="text-xs font-bold text-gray-500 uppercase tracking-widest overflow-hidden whitespace-nowrap text-ellipsis">
          Sources
        </h2>
        <div className="flex gap-2">
          {isMobile && (
            <Button variant="icon" title="Close Sidebar" onClick={onToggle}>
              <span className="material-symbols-outlined icon-sm">close</span>
            </Button>
          )}
          <Button variant="icon" title="Space Dashboard">
            <span className="material-symbols-outlined icon-sm">
              space_dashboard
            </span>
          </Button>
        </div>
      </div>
      <div className="px-5 py-5 space-y-4">
        <button className="w-full py-3 px-4 bg-white border border-dashed border-black shadow-sm hover:shadow-md hover:border-solid hover:-translate-y-0.5 active:translate-y-0 active:shadow-none transition-all flex items-center justify-center gap-2 text-sm font-bold group text-gray-600 hover:text-black">
          <span className="material-symbols-outlined icon-sm text-accent-main group-hover:text-black transition-colors">
            add
          </span>
          Add source
        </button>
        <div className="relative group">
          <span className="material-symbols-outlined absolute left-3 top-2.5 text-gray-400 z-10 icon-sm">
            search
          </span>
          <input
            className="w-full bg-white border border-gray-300 py-2 pl-10 pr-4 text-sm font-medium placeholder-gray-400 focus:border-black focus:ring-0 focus:shadow-hard-sm transition-all outline-none"
            placeholder="Filter sources..."
            type="text"
          />
        </div>
        <div className="flex gap-2 text-xs font-bold w-full overflow-x-auto pb-1 hide-scrollbar">
          <button className="px-3 py-1.5 bg-black text-white border border-black shadow-hard-sm shrink-0">
            All
          </button>
          <button className="px-3 py-1.5 bg-transparent text-gray-600 border border-gray-300 hover:border-black hover:text-black transition-colors shrink-0">
            PDF
          </button>
          <button className="px-3 py-1.5 bg-transparent text-gray-600 border border-gray-300 hover:border-black hover:text-black transition-colors shrink-0">
            Web
          </button>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto px-5 pb-4 space-y-2">
        <div className="px-1 py-2 text-[10px] font-bold uppercase tracking-wider text-gray-400 mb-2">
          Selected (6)
        </div>
        {/* Source items placeholder mapping */}
        {[
          {
            icon: "picture_as_pdf",
            title: "AWS Agentic AI",
            sub: "Framework guidance",
            active: true,
          },
          { icon: "article", title: "Best Practices", sub: "Building Systems" },
          {
            icon: "code",
            title: "Dhenara Agent DSL",
            sub: "GitHub Repository",
          },
          {
            icon: "article",
            title: "Agent Skills",
            sub: "Implementation Guide",
          },
          {
            icon: "picture_as_pdf",
            title: "LLM RecSys",
            sub: "Multi-agent Arch",
          },
        ].map((source, idx) => (
          <div
            key={idx}
            className={`group flex items-center gap-3 p-3 bg-white shadow-sm cursor-pointer transition-all ${source.active ? "border-l-2 border-l-accent-main border-y border-r border-gray-200 hover:border-black" : "border border-gray-200 hover:border-black"}`}
          >
            <div
              className={`p-1.5 rounded-sm shrink-0 ${source.active ? "bg-accent-light text-accent-secondary" : "bg-gray-50 text-gray-400 group-hover:text-black transition-colors"}`}
            >
              <span className="material-symbols-outlined icon-sm">
                {source.icon}
              </span>
            </div>
            <div className="flex-1 min-w-0">
              <p
                className={`text-sm tracking-tight truncate ${source.active ? "font-bold" : "font-semibold text-gray-700 group-hover:text-black"}`}
              >
                {source.title}
              </p>
              <p className="text-xs text-gray-500 truncate font-medium">
                {source.sub}
              </p>
            </div>
            {source.active && (
              <div className="w-2 h-2 rounded-full bg-accent-main shrink-0"></div>
            )}
          </div>
        ))}
      </div>
    </aside>
  );
}
