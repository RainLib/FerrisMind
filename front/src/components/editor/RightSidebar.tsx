"use client";

import * as React from "react";
import { useState } from "react";
import { cn } from "@/lib/utils";
import { PanelDetailView } from "./PanelDetailView";

interface RightSidebarProps {
  isExpanded?: boolean;
  onToggle?: () => void;
  isPanel?: boolean;
}

export function RightSidebar({
  isExpanded = true,
  onToggle,
  isPanel,
}: RightSidebarProps) {
  const [activeTool, setActiveTool] = useState<string | null>(null);

  const handleToolClick = (toolId: string) => {
    if (!isExpanded && onToggle) onToggle();
    setActiveTool(toolId);
  };

  return (
    <aside
      className={cn(
        "flex flex-col h-full bg-bg-studio border-l border-border-bold relative",
        !isPanel &&
          (isExpanded
            ? "w-80"
            : "w-16 transition-all duration-300 ease-in-out"),
        isPanel && "w-full",
      )}
    >
      <div className="h-14 px-4 flex items-center justify-between border-b border-border-bold bg-bg-studio shrink-0">
        {isExpanded && (
          <h2 className="text-xs font-bold text-gray-500 uppercase tracking-widest whitespace-nowrap overflow-hidden">
            Studio
          </h2>
        )}
        <button
          onClick={onToggle}
          className={cn(
            "text-gray-500 hover:bg-black hover:text-white border border-transparent hover:border-black transition-all p-1 relative group",
            !isExpanded && "mx-auto",
          )}
        >
          <span className="material-symbols-outlined icon-sm">
            {isExpanded ? "dock_to_right" : "dock_to_left"}
          </span>
          <div className="absolute top-full mt-2 right-0 bg-black text-white text-[10px] font-bold px-2 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm border border-black hidden group-hover:block">
            {isExpanded ? "Collapse Sidebar" : "Expand Sidebar"}
          </div>
        </button>
      </div>

      <div
        className={cn(
          "p-4 border-b border-gray-200 bg-stone-50/50 shrink-0 transition-opacity",
          !isExpanded && "px-2 py-4",
        )}
      >
        <div
          className={cn(
            "grid gap-3 transition-all",
            isExpanded ? "grid-cols-2" : "grid-cols-1",
          )}
        >
          {/* Audio Tool */}
          <button
            onClick={() => handleToolClick("audio")}
            className={cn(
              "relative flex p-3 bg-orange-50/60 border border-orange-200 rounded-sm hover:border-orange-500 hover:shadow-md transition-all text-xs font-bold text-gray-800 hatch-pattern-orange group overflow-hidden items-center justify-center",
              isExpanded ? "flex-col gap-2" : "",
            )}
            style={{
              backgroundImage:
                "repeating-linear-gradient(45deg, rgba(249, 115, 22, 0.1) 0, rgba(249, 115, 22, 0.1) 1px, transparent 0, transparent 8px)",
            }}
          >
            {isExpanded && (
              <div className="absolute top-0 right-0 border-b border-l border-orange-200 bg-white px-1 py-0.5 text-[8px] text-orange-400 font-mono tracking-tighter opacity-70 group-hover:opacity-100 transition-opacity">
                [AU]
              </div>
            )}
            <div className="w-8 h-8 flex items-center justify-center bg-white border border-orange-100 rounded-full shadow-sm group-hover:scale-105 transition-transform shrink-0">
              <span className="material-symbols-outlined icon-sm text-orange-500">
                graphic_eq
              </span>
            </div>
            {isExpanded && (
              <span className="z-10 text-left pt-1 whitespace-nowrap overflow-hidden text-ellipsis">
                Audio
              </span>
            )}
            <div className="absolute bottom-0 left-0 w-full h-0.5 bg-orange-400 opacity-60 group-hover:opacity-100 transition-opacity"></div>
          </button>

          {/* Video Tool */}
          <button
            onClick={() => handleToolClick("video")}
            className={cn(
              "relative flex p-3 bg-cyan-50/60 border border-cyan-200 rounded-sm hover:border-cyan-500 hover:shadow-md transition-all text-xs font-bold text-gray-800 hatch-pattern-blue group overflow-hidden items-center justify-center",
              isExpanded ? "flex-col gap-2" : "",
            )}
            style={{
              backgroundImage:
                "repeating-linear-gradient(45deg, rgba(6, 182, 212, 0.1) 0, rgba(6, 182, 212, 0.1) 1px, transparent 0, transparent 8px)",
            }}
          >
            {isExpanded && (
              <div className="absolute top-0 right-0 border-b border-l border-cyan-200 bg-white px-1 py-0.5 text-[8px] text-cyan-500 font-mono tracking-tighter opacity-70 group-hover:opacity-100 transition-opacity">
                [VI]
              </div>
            )}
            <div className="w-8 h-8 flex items-center justify-center bg-white border border-cyan-100 rounded-full shadow-sm group-hover:scale-105 transition-transform shrink-0">
              <span className="material-symbols-outlined icon-sm text-cyan-500">
                smart_display
              </span>
            </div>
            {isExpanded && (
              <span className="z-10 text-left pt-1 whitespace-nowrap overflow-hidden text-ellipsis">
                Video
              </span>
            )}
            <div className="absolute bottom-0 left-0 w-full h-0.5 bg-cyan-500 opacity-60 group-hover:opacity-100 transition-opacity"></div>
          </button>

          {/* Brief Tool */}
          <button
            onClick={() => handleToolClick("brief")}
            className={cn(
              "relative flex p-3 bg-emerald-50/60 border border-emerald-200 rounded-sm hover:border-emerald-600 hover:shadow-md transition-all text-xs font-bold text-gray-800 hatch-pattern-green group overflow-hidden items-center justify-center",
              isExpanded ? "flex-col gap-2" : "",
            )}
            style={{
              backgroundImage:
                "repeating-linear-gradient(45deg, rgba(16, 185, 129, 0.1) 0, rgba(16, 185, 129, 0.1) 1px, transparent 0, transparent 8px)",
            }}
          >
            {isExpanded && (
              <div className="absolute top-0 right-0 border-b border-l border-emerald-200 bg-white px-1 py-0.5 text-[8px] text-emerald-600 font-mono tracking-tighter opacity-70 group-hover:opacity-100 transition-opacity">
                [BR]
              </div>
            )}
            <div className="w-8 h-8 flex items-center justify-center bg-white border border-emerald-100 rounded-full shadow-sm group-hover:scale-105 transition-transform shrink-0">
              <span className="material-symbols-outlined icon-sm text-emerald-600">
                summarize
              </span>
            </div>
            {isExpanded && (
              <span className="z-10 text-left pt-1 whitespace-nowrap overflow-hidden text-ellipsis">
                Brief
              </span>
            )}
            <div className="absolute bottom-0 left-0 w-full h-0.5 bg-emerald-600 opacity-60 group-hover:opacity-100 transition-opacity"></div>
          </button>

          {/* Cards Tool */}
          <button
            onClick={() => handleToolClick("cards")}
            className={cn(
              "relative flex p-3 bg-violet-50/60 border border-violet-200 rounded-sm hover:border-violet-600 hover:shadow-md transition-all text-xs font-bold text-gray-800 hatch-pattern-purple group overflow-hidden items-center justify-center",
              isExpanded ? "flex-col gap-2" : "",
            )}
            style={{
              backgroundImage:
                "repeating-linear-gradient(45deg, rgba(139, 92, 246, 0.1) 0, rgba(139, 92, 246, 0.1) 1px, transparent 0, transparent 8px)",
            }}
          >
            {isExpanded && (
              <div className="absolute top-0 right-0 border-b border-l border-violet-200 bg-white px-1 py-0.5 text-[8px] text-violet-600 font-mono tracking-tighter opacity-70 group-hover:opacity-100 transition-opacity">
                [CA]
              </div>
            )}
            <div className="w-8 h-8 flex items-center justify-center bg-white border border-violet-100 rounded-full shadow-sm group-hover:scale-105 transition-transform shrink-0">
              <span className="material-symbols-outlined icon-sm text-violet-600">
                style
              </span>
            </div>
            {isExpanded && (
              <span className="z-10 text-left pt-1 whitespace-nowrap overflow-hidden text-ellipsis">
                Cards
              </span>
            )}
            <div className="absolute bottom-0 left-0 w-full h-0.5 bg-violet-600 opacity-60 group-hover:opacity-100 transition-opacity"></div>
          </button>
        </div>
      </div>

      <div className="flex-1 px-5 py-5 flex flex-col items-center justify-start overflow-y-auto">
        {isExpanded ? (
          <div
            className="w-full h-48 border-2 border-dashed border-gray-300 rounded-sm relative flex flex-col items-center justify-center p-6 text-center"
            style={{
              backgroundImage:
                "repeating-linear-gradient(45deg, #e5e5e5 0, #e5e5e5 1px, transparent 0, transparent 10px)",
            }}
          >
            <div className="absolute top-2 right-2 text-[9px] font-bold text-gray-400 bg-white px-1 border border-gray-200">
              [ST-EMPTY]
            </div>
            <div className="w-12 h-12 bg-white rounded-full flex items-center justify-center border border-gray-200 mb-3 shadow-sm">
              <span className="material-symbols-outlined text-gray-300 icon-lg">
                auto_awesome
              </span>
            </div>
            <h3 className="text-sm font-bold text-gray-600 mb-1">
              No generated content yet
            </h3>
            <p className="text-xs text-gray-400 font-medium leading-relaxed">
              Select a tool above to start creating
            </p>
          </div>
        ) : (
          <div className="flex flex-col items-center gap-4 mt-4 w-full">
            <button
              title="History"
              className="text-gray-400 hover:text-black hover:bg-gray-100 p-2 rounded-sm transition-colors"
            >
              <span className="material-symbols-outlined">history</span>
            </button>
          </div>
        )}
      </div>

      <div
        className={cn(
          "absolute bottom-6 left-0 right-0 flex justify-center pointer-events-none z-10",
        )}
      >
        {isExpanded ? (
          <button className="pointer-events-auto bg-black text-white border border-black px-6 py-3 shadow-[4px_4px_0px_0px_#f59e0b] hover:shadow-[6px_6px_0px_0px_#d97706] hover:-translate-y-0.5 transition-all flex items-center gap-2 font-black text-sm uppercase tracking-wide">
            <span className="material-symbols-outlined icon-sm">note_add</span>
            New Note
          </button>
        ) : (
          <button className="pointer-events-auto bg-black text-white border border-black w-10 h-10 shadow-[2px_2px_0px_0px_#f59e0b] flex items-center justify-center hover:shadow-[4px_4px_0px_0px_#d97706] hover:-translate-y-0.5 transition-all">
            <span className="material-symbols-outlined icon-sm">note_add</span>
          </button>
        )}
      </div>

      {isExpanded && activeTool === "audio" && (
        <PanelDetailView
          title="Audio Studio"
          icon="graphic_eq"
          onBack={() => setActiveTool(null)}
        >
          <div className="p-5 flex flex-col gap-4">
            <div className="p-4 bg-orange-50/50 border border-orange-200 rounded-sm">
              <h3 className="text-sm font-bold text-orange-800 mb-2">
                Voice Generation
              </h3>
              <p className="text-xs text-orange-600/80 mb-4">
                Generate podcast-style discussions or audio summaries from your
                sources.
              </p>
              <button className="w-full py-2 bg-orange-500 hover:bg-orange-600 text-white font-bold text-xs rounded-sm transition-colors shadow-sm">
                Generate Audio
              </button>
            </div>

            <div className="flex flex-col gap-2">
              <h4 className="text-xs font-bold text-gray-500 uppercase tracking-widest px-1">
                Options
              </h4>
              <div className="p-3 bg-white border border-gray-200 shadow-sm flex items-center justify-between">
                <span className="text-xs font-medium">Voice Clone</span>
                <span className="text-[10px] bg-gray-100 text-gray-500 px-2 py-0.5 font-bold uppercase">
                  Pro
                </span>
              </div>
              <div className="p-3 bg-white border border-gray-200 shadow-sm flex items-center justify-between">
                <span className="text-xs font-medium">Background Music</span>
                <span className="material-symbols-outlined icon-sm text-gray-400">
                  toggle_off
                </span>
              </div>
            </div>
          </div>
        </PanelDetailView>
      )}

      {isExpanded && activeTool === "video" && (
        <PanelDetailView
          title="Video Studio"
          icon="smart_display"
          onBack={() => setActiveTool(null)}
        >
          <div className="p-5 flex flex-col gap-4">
            <div className="p-4 bg-cyan-50/50 border border-cyan-200 rounded-sm">
              <h3 className="text-sm font-bold text-cyan-800 mb-2">
                Video Generation
              </h3>
              <p className="text-xs text-cyan-600/80 mb-4">
                Create short explainer videos generated directly from your
                notes.
              </p>
              <button className="w-full py-2 bg-cyan-500 hover:bg-cyan-600 text-white font-bold text-xs rounded-sm transition-colors shadow-sm">
                Generate Video
              </button>
            </div>
          </div>
        </PanelDetailView>
      )}

      {isExpanded && activeTool === "brief" && (
        <PanelDetailView
          title="Brief Generator"
          icon="summarize"
          onBack={() => setActiveTool(null)}
        >
          <div className="p-5 flex flex-col gap-4">
            <div className="p-4 bg-emerald-50/50 border border-emerald-200 rounded-sm">
              <h3 className="text-sm font-bold text-emerald-800 mb-2">
                Executive Brief
              </h3>
              <p className="text-xs text-emerald-600/80 mb-4">
                Synthesize all your sources into a quick, readable summary
                document.
              </p>
              <button className="w-full py-2 bg-emerald-600 hover:bg-emerald-700 text-white font-bold text-xs rounded-sm transition-colors shadow-sm">
                Create Brief
              </button>
            </div>
          </div>
        </PanelDetailView>
      )}

      {isExpanded && activeTool === "cards" && (
        <PanelDetailView
          title="Flashcards"
          icon="style"
          onBack={() => setActiveTool(null)}
        >
          <div className="p-5 flex flex-col gap-4">
            <div className="p-4 bg-violet-50/50 border border-violet-200 rounded-sm">
              <h3 className="text-sm font-bold text-violet-800 mb-2">
                Study Cards
              </h3>
              <p className="text-xs text-violet-600/80 mb-4">
                Automatically generate flashcards and FAQs for active recall.
              </p>
              <button className="w-full py-2 bg-violet-600 hover:bg-violet-700 text-white font-bold text-xs rounded-sm transition-colors shadow-sm">
                Generate Cards
              </button>
            </div>
          </div>
        </PanelDetailView>
      )}
    </aside>
  );
}
