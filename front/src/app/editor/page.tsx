import { LeftSidebar } from "@/components/editor/LeftSidebar";
import { ChatPanel } from "@/components/editor/ChatPanel";
import { RightSidebar } from "@/components/editor/RightSidebar";

export default function Editor() {
  return (
    <>
      <header className="h-16 shrink-0 border-b border-border-bold flex items-center justify-between px-6 bg-white z-20 relative overflow-hidden">
        <div
          className="absolute top-0 right-0 bottom-0 w-64 pointer-events-none opacity-10"
          style={{
            backgroundImage:
              "repeating-linear-gradient(45deg, #171717, #171717 1px, transparent 1px, transparent 6px)",
          }}
        ></div>
        <div className="flex items-center gap-4 relative z-10">
          <div className="w-10 h-10 bg-black text-white flex items-center justify-center shadow-hard-sm border border-black transform transition-transform hover:-translate-y-0.5">
            <span className="material-symbols-outlined icon-lg">
              auto_awesome
            </span>
          </div>
          <h1 className="font-bold text-xl tracking-tight uppercase">
            Neo Workspace
          </h1>
        </div>
        <div className="flex items-center gap-3 relative z-10">
          <button className="flex items-center gap-2 px-4 py-2 bg-accent-main text-white border border-black shadow-hard-sm hover:shadow-hard hover:-translate-y-0.5 active:translate-y-0 active:shadow-none transition-all rounded-none font-bold text-sm">
            <span className="material-symbols-outlined icon-sm">add</span>
            Notebook
          </button>
          <div className="h-6 w-px bg-gray-300 mx-2"></div>
          <button
            className="w-10 h-10 flex items-center justify-center border border-transparent hover:border-black hover:bg-gray-50 transition-all rounded-none text-gray-600 hover:text-black"
            title="Analytics"
          >
            <span className="material-symbols-outlined icon-sm">analytics</span>
          </button>
          <button
            className="w-10 h-10 flex items-center justify-center border border-transparent hover:border-black hover:bg-gray-50 transition-all rounded-none text-gray-600 hover:text-black"
            title="Share"
          >
            <span className="material-symbols-outlined icon-sm">share</span>
          </button>
          <button
            className="w-10 h-10 flex items-center justify-center border border-transparent hover:border-black hover:bg-gray-50 transition-all rounded-none text-gray-600 hover:text-black"
            title="Settings"
          >
            <span className="material-symbols-outlined icon-sm">settings</span>
          </button>
          <div className="w-10 h-10 border border-black overflow-hidden shadow-hard-sm ml-2">
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img
              alt="User Avatar"
              className="w-full h-full object-cover grayscale hover:grayscale-0 transition-all"
              src="https://lh3.googleusercontent.com/aida-public/AB6AXuDU-DdYBfrFz-om3NR3ti3vMwzqmnGUGZLKIiUxgjgXeggfaNKkY4I7KzszndsvY7r90cccF3eWELBKnYVytB6PDtTlC9zAwd6ULKKLUmvHlt76S9XdpTsG_v3MgdW5thM63xoMm-gknjo3UFZkCpDmnYnerCiDaIGG4_5FjTWyrXPqf5Z_UMWcgXrWelxirf9_Ne6wWI52X_af3MNcsIOQe-tBE9EeO01HQX6mLI9Ovlagabo_xz1alYPg0osyOjcZMQFRlhTLo83t"
            />
          </div>
        </div>
      </header>
      <main className="flex-1 flex overflow-hidden">
        <LeftSidebar />
        <ChatPanel />
        <RightSidebar />
      </main>
    </>
  );
}
