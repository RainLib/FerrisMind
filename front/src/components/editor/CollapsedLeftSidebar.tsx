import { useState } from "react";
import { useNotebookStore } from "@/store/notebookStore";

export function CollapsedLeftSidebar({ onExpand }: { onExpand: () => void }) {
  const { sources, selectedIds, setIsAddSourceModalOpen } = useNotebookStore();
  const [hoveredSource, setHoveredSource] = useState<{
    title: string;
    top: number;
  } | null>(null);

  return (
    <aside className="w-14 flex flex-col bg-bg-sources nb-border-r items-center py-4 gap-6 shrink-0 transition-all duration-300 z-10 relative">
      <button
        onClick={onExpand}
        className="w-9 h-9 flex items-center justify-center text-gray-500 hover:text-black transition-colors rounded-sm group relative"
      >
        <span className="material-symbols-outlined icon-md">view_sidebar</span>
        <div className="absolute left-full top-1/2 -translate-y-1/2 ml-2 bg-black text-white text-xs font-bold px-2.5 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm hidden group-hover:block rounded-[2px]">
          Expand Sidebar
        </div>
      </button>

      <div className="w-8 h-px bg-gray-300"></div>

      <div className="flex flex-col gap-3 items-center w-full">
        <button
          onClick={() => {
            onExpand();
            setIsAddSourceModalOpen(true);
          }}
          className="w-9 h-9 bg-white border-2 border-black flex items-center justify-center hover:bg-gray-100 transition-colors group relative rounded-sm"
        >
          <span className="material-symbols-outlined icon-sm font-bold text-black">
            add
          </span>
          <div className="absolute left-full top-1/2 -translate-y-1/2 ml-2 bg-black text-white text-xs font-bold px-2.5 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm hidden group-hover:block rounded-[2px]">
            Add Source
          </div>
        </button>
        <div className="relative group cursor-pointer" onClick={onExpand}>
          <div
            className={`w-9 h-9 flex items-center justify-center text-sm border-2 rounded-sm transition-colors ${
              selectedIds.size > 0
                ? "bg-accent-main text-white font-extrabold border-black"
                : "bg-gray-100 text-gray-400 border-gray-300 font-bold"
            }`}
          >
            {selectedIds.size}
          </div>
          <div className="absolute left-full top-1/2 -translate-y-1/2 ml-2 bg-black text-white text-xs font-bold px-2.5 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm hidden group-hover:block rounded-[2px]">
            Active Sources
          </div>
        </div>
      </div>

      <div className="flex-1 flex flex-col gap-3 items-center mt-2 w-full overflow-y-auto hide-scrollbar pb-4 max-h-[60vh]">
        {sources.slice(0, 15).map((source) => (
          <button
            key={source.id}
            onClick={onExpand}
            onMouseEnter={(e) => {
              const rect = e.currentTarget.getBoundingClientRect();
              setHoveredSource({
                title: source.title,
                top: rect.top + rect.height / 2,
              });
            }}
            onMouseLeave={() => setHoveredSource(null)}
            className="flex items-center justify-center w-9 h-9 text-gray-500 hover:text-black hover:bg-white border-2 border-transparent hover:border-black transition-colors rounded-sm shrink-0"
          >
            <span className="material-symbols-outlined icon-md">
              {source.icon}
            </span>
          </button>
        ))}
        {sources.length > 15 && (
          <div className="w-8 h-8 flex items-center justify-center text-xs font-bold text-gray-400">
            +{sources.length - 15}
          </div>
        )}
      </div>

      {hoveredSource && (
        <div
          style={{ top: hoveredSource.top }}
          className="fixed left-[56px] -translate-y-1/2 ml-2 bg-black text-white text-xs font-bold px-2.5 py-1 whitespace-nowrap z-[100] shadow-sm rounded-[2px] max-w-[200px] truncate pointer-events-none"
        >
          {hoveredSource.title}
        </div>
      )}

      <div className="mt-auto">
        <div className="writing-vertical-rl transform rotate-180 text-[10px] font-bold uppercase tracking-widest text-gray-400 py-4 cursor-default select-none">
          Sources
        </div>
      </div>
    </aside>
  );
}
