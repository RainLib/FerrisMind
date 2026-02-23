export function CollapsedRightSidebar({ onExpand }: { onExpand: () => void }) {
  return (
    <aside className="w-14 flex flex-col bg-bg-studio nb-border-l items-center py-4 gap-6 shrink-0 transition-all duration-300 relative z-10">
      <button
        onClick={onExpand}
        className="w-9 h-9 flex items-center justify-center text-gray-500 hover:bg-black hover:text-white border border-transparent hover:border-black transition-all rounded-sm"
        title="Expand Studio"
      >
        <span className="material-symbols-outlined icon-sm">dock_to_left</span>
      </button>

      <div className="w-8 h-px bg-gray-300"></div>

      <div className="flex flex-col gap-3 w-full items-center">
        <button
          onClick={onExpand}
          className="w-9 h-9 flex items-center justify-center bg-white border border-transparent hover:border-black hover:shadow-hard-sm text-gray-500 hover:text-black transition-all rounded-sm group relative"
          title="Audio"
        >
          <span className="material-symbols-outlined icon-sm">graphic_eq</span>
          <div className="absolute w-2 h-2 bg-accent-main rounded-full top-1 right-1 border border-white"></div>
        </button>
        <button
          onClick={onExpand}
          className="w-9 h-9 flex items-center justify-center bg-white border border-transparent hover:border-black hover:shadow-hard-sm text-gray-500 hover:text-black transition-all rounded-sm"
          title="Video"
        >
          <span className="material-symbols-outlined icon-sm">
            smart_display
          </span>
        </button>
        <button
          onClick={onExpand}
          className="w-9 h-9 flex items-center justify-center bg-white border border-transparent hover:border-black hover:shadow-hard-sm text-gray-500 hover:text-black transition-all rounded-sm"
          title="Brief"
        >
          <span className="material-symbols-outlined icon-sm">summarize</span>
        </button>
        <button
          onClick={onExpand}
          className="w-9 h-9 flex items-center justify-center bg-white border border-transparent hover:border-black hover:shadow-hard-sm text-gray-500 hover:text-black transition-all rounded-sm"
          title="Cards"
        >
          <span className="material-symbols-outlined icon-sm">style</span>
        </button>
      </div>

      <div className="mt-auto flex flex-col items-center gap-4 w-full">
        <div className="w-8 h-px bg-gray-300"></div>
        <button
          className="w-9 h-9 flex items-center justify-center bg-accent-main border border-black shadow-hard-sm text-white hover:-translate-y-0.5 transition-all rounded-sm"
          title="New Output"
        >
          <span className="material-symbols-outlined icon-sm">add</span>
        </button>
        <div className="writing-vertical-rl text-[10px] font-bold uppercase tracking-widest text-gray-400 py-4 cursor-default select-none">
          Studio
        </div>
      </div>
    </aside>
  );
}
