import * as React from "react";

interface AddSourceModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function AddSourceModal({ isOpen, onClose }: AddSourceModalProps) {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-stone-100/60 backdrop-blur-[2px]">
      <div className="bg-white w-[600px] border border-black shadow-modal relative">
        <button
          onClick={onClose}
          className="absolute top-4 right-4 p-1 hover:bg-gray-100 rounded-sm transition-colors z-10"
        >
          <span className="material-symbols-outlined text-gray-400 hover:text-black">
            close
          </span>
        </button>
        <div className="absolute top-0 left-0 px-2 py-1 bg-black text-white text-[10px] font-mono font-bold">
          M-01: UPLOAD
        </div>
        <div className="p-8 pt-10">
          <h2 className="text-xl font-bold mb-1 text-center">
            Create Overview from your documents
          </h2>
          <p className="text-center text-sm text-gray-500 mb-8 font-medium">
            Add sources to generate insights
          </p>
          <div className="relative group mb-8">
            <span className="material-symbols-outlined absolute left-4 top-3.5 text-gray-400 z-10 icon-sm">
              search
            </span>
            <input
              className="w-full bg-white border border-gray-300 py-3 pl-12 pr-12 text-sm font-medium placeholder-gray-400 focus:border-black focus:ring-0 focus:shadow-hard-sm transition-all"
              placeholder="Search the web for new sources"
              type="text"
            />
            <button className="absolute right-3 top-2 px-2 py-1 bg-gray-100 hover:bg-gray-200 border border-gray-200 text-[10px] font-bold uppercase tracking-wider text-gray-600 rounded-sm transition-colors flex items-center gap-1">
              <span className="material-symbols-outlined text-[14px]">
                language
              </span>{" "}
              Web
            </button>
          </div>
          <div className="relative h-64 border-2 border-dashed border-gray-300 bg-stone-50/50 hover:bg-white hover:border-black transition-all group flex flex-col items-center justify-center gap-4 mb-4 overflow-hidden">
            <div className="absolute inset-0 bg-background-image-diagonal-hatch opacity-30 pointer-events-none"></div>
            <div className="relative z-10 flex flex-col items-center">
              <p className="text-base font-medium text-gray-800">
                or drop your files
              </p>
              <p className="text-xs text-gray-400 mt-1">
                pdf, images, docs, audio,{" "}
                <span className="underline cursor-pointer hover:text-black">
                  and more
                </span>
              </p>
            </div>
            <div className="relative z-10 flex gap-3 mt-4">
              <button className="flex items-center gap-2 px-4 py-2 bg-black text-white border border-black shadow-hard-sm hover:shadow-hard hover:-translate-y-0.5 transition-all text-xs font-bold">
                <span className="material-symbols-outlined icon-sm">
                  upload_file
                </span>
                Upload files
              </button>
              <button className="flex items-center gap-2 px-4 py-2 bg-white text-gray-700 border border-gray-300 hover:border-black shadow-sm hover:shadow-hard-sm hover:-translate-y-0.5 transition-all text-xs font-bold">
                <span className="material-symbols-outlined icon-sm">link</span>
                Websites
              </button>
              <button className="flex items-center gap-2 px-4 py-2 bg-white text-gray-700 border border-gray-300 hover:border-black shadow-sm hover:shadow-hard-sm hover:-translate-y-0.5 transition-all text-xs font-bold">
                <span className="material-symbols-outlined icon-sm">
                  add_to_drive
                </span>
                Drive
              </button>
              <button className="flex items-center gap-2 px-4 py-2 bg-white text-gray-700 border border-gray-300 hover:border-black shadow-sm hover:shadow-hard-sm hover:-translate-y-0.5 transition-all text-xs font-bold">
                <span className="material-symbols-outlined icon-sm">
                  content_paste
                </span>
                Copied text
              </button>
            </div>
          </div>
          <div className="flex items-center gap-3 mt-6">
            <div className="flex-1 h-1.5 bg-gray-100 rounded-full overflow-hidden border border-gray-200">
              <div className="h-full bg-accent-main w-[65%] rounded-full"></div>
            </div>
            <span className="text-[10px] font-mono font-bold text-gray-400">
              11 / 300
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
