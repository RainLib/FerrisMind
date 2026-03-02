export function CollapsedRightSidebar({ onExpand }: { onExpand: () => void }) {
  return (
    <aside className="w-14 flex flex-col bg-bg-studio border-l border-border-bold items-center py-4 gap-6 shrink-0 transition-all duration-300 relative z-10">
      <button
        onClick={onExpand}
        className="w-9 h-9 flex items-center justify-center text-gray-500 hover:bg-primary hover:text-bg-main border border-transparent hover:border-border-bold transition-all rounded-sm group relative"
      >
        <span className="material-symbols-outlined icon-sm">dock_to_left</span>
        <div className="absolute right-full top-1/2 -translate-y-1/2 mr-2 bg-primary text-bg-main text-[10px] font-bold px-2 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm border border-border-bold hidden group-hover:block">
          Expand Studio
        </div>
      </button>

      <div className="w-8 h-px bg-gray-300"></div>

      <div className="flex flex-col gap-3 w-full items-center">
        <button
          onClick={onExpand}
          className="w-9 h-9 flex items-center justify-center bg-bg-main border border-transparent hover:border-border-bold hover:shadow-hard-sm text-gray-500 hover:text-primary transition-all rounded-sm group relative"
        >
          <span className="material-symbols-outlined icon-sm">graphic_eq</span>
          <div className="absolute w-2 h-2 bg-accent-main rounded-full top-1 right-1 border border-bg-main"></div>
          <div className="absolute right-full top-1/2 -translate-y-1/2 mr-2 bg-primary text-bg-main text-[10px] font-bold px-2 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm border border-border-bold hidden group-hover:block">
            Audio
          </div>
        </button>
        <button
          onClick={onExpand}
          className="w-9 h-9 flex items-center justify-center bg-bg-main border border-transparent hover:border-border-bold hover:shadow-hard-sm text-gray-500 hover:text-primary transition-all rounded-sm group relative"
        >
          <span className="material-symbols-outlined icon-sm">
            smart_display
          </span>
          <div className="absolute right-full top-1/2 -translate-y-1/2 mr-2 bg-primary text-bg-main text-[10px] font-bold px-2 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm border border-border-bold hidden group-hover:block">
            Video
          </div>
        </button>
        <button
          onClick={onExpand}
          className="w-9 h-9 flex items-center justify-center bg-bg-main border border-transparent hover:border-border-bold hover:shadow-hard-sm text-gray-500 hover:text-primary transition-all rounded-sm group relative"
        >
          <span className="material-symbols-outlined icon-sm">summarize</span>
          <div className="absolute right-full top-1/2 -translate-y-1/2 mr-2 bg-primary text-bg-main text-[10px] font-bold px-2 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm border border-border-bold hidden group-hover:block">
            Brief
          </div>
        </button>
        <button
          onClick={onExpand}
          className="w-9 h-9 flex items-center justify-center bg-bg-main border border-transparent hover:border-border-bold hover:shadow-hard-sm text-gray-500 hover:text-primary transition-all rounded-sm group relative"
        >
          <span className="material-symbols-outlined icon-sm">style</span>
          <div className="absolute right-full top-1/2 -translate-y-1/2 mr-2 bg-primary text-bg-main text-[10px] font-bold px-2 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm border border-border-bold hidden group-hover:block">
            Cards
          </div>
        </button>
      </div>

      <div className="mt-auto flex flex-col items-center gap-4 w-full">
        <div className="w-8 h-px bg-border-light"></div>
        <button className="w-9 h-9 flex items-center justify-center bg-accent-main border border-border-bold shadow-hard-sm text-bg-main hover:-translate-y-0.5 transition-all rounded-sm group relative">
          <span className="material-symbols-outlined icon-sm">add</span>
          <div className="absolute right-full top-1/2 -translate-y-1/2 mr-2 bg-primary text-bg-main text-[10px] font-bold px-2 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm border border-border-bold hidden group-hover:block">
            New Output
          </div>
        </button>
        <div className="writing-vertical-rl text-[10px] font-bold uppercase tracking-widest text-gray-400 py-4 cursor-default select-none">
          Studio
        </div>
      </div>
    </aside>
  );
}
