export function CollapsedLeftSidebar({ onExpand }: { onExpand: () => void }) {
  return (
    <aside className="w-14 flex flex-col bg-bg-sources nb-border-r items-center py-4 gap-6 shrink-0 transition-all duration-300 z-10">
      <button
        onClick={onExpand}
        className="w-9 h-9 flex items-center justify-center text-gray-500 hover:bg-black hover:text-white border border-transparent hover:border-black transition-all rounded-sm"
        title="Expand Sources"
      >
        <span className="material-symbols-outlined icon-sm">dock_to_right</span>
      </button>

      <div className="w-8 h-px bg-gray-300"></div>

      <div className="flex flex-col gap-4 items-center w-full">
        <button
          className="w-9 h-9 bg-white border border-black shadow-hard-sm flex items-center justify-center hover:bg-accent-light hover:text-accent-secondary transition-all group"
          title="Add Source"
        >
          <span className="material-symbols-outlined icon-sm">add</span>
        </button>
        <div className="relative group cursor-pointer" onClick={onExpand}>
          <div className="w-9 h-9 flex items-center justify-center bg-accent-main text-white font-bold text-xs border border-black shadow-sm">
            9
          </div>
          <div className="absolute left-full top-0 ml-2 bg-black text-white text-xs px-2 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
            Active Sources
          </div>
        </div>
      </div>

      <div className="flex-1 flex flex-col gap-3 items-center mt-2 w-full">
        <button
          className="w-8 h-8 flex items-center justify-center text-gray-400 hover:text-black hover:bg-white hover:border hover:border-black transition-all rounded-sm"
          title="PDFs"
        >
          <span className="material-symbols-outlined icon-sm">
            picture_as_pdf
          </span>
        </button>
        <button
          className="w-8 h-8 flex items-center justify-center text-gray-400 hover:text-black hover:bg-white hover:border hover:border-black transition-all rounded-sm"
          title="Web"
        >
          <span className="material-symbols-outlined icon-sm">public</span>
        </button>
      </div>

      <div className="mt-auto">
        <div className="writing-vertical-rl transform rotate-180 text-[10px] font-bold uppercase tracking-widest text-gray-400 py-4 cursor-default select-none">
          Sources
        </div>
      </div>
    </aside>
  );
}
