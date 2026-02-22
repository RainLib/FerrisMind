import Link from "next/link";

export default function Home() {
  return (
    <>
      <header className="h-16 shrink-0 border-b border-border-bold flex items-center justify-between px-6 bg-white z-20 relative">
        <div className="flex items-center gap-4">
          <div className="w-8 h-8 bg-black text-white flex items-center justify-center shadow-hard-sm border border-black">
            <span className="material-symbols-outlined icon-sm">
              auto_awesome
            </span>
          </div>
          <h1 className="font-bold text-lg tracking-tight uppercase">
            Neo Workspace
          </h1>
        </div>
        <div className="flex items-center gap-4">
          <div className="relative hidden md:block">
            <span className="material-symbols-outlined absolute left-3 top-2 text-gray-400 icon-sm">
              search
            </span>
            <input
              className="bg-gray-50 border border-gray-200 rounded-lg py-1.5 pl-9 pr-4 text-sm focus:border-black focus:ring-0 transition-all w-64 placeholder-gray-400"
              placeholder="Search notebooks..."
              type="text"
            />
          </div>
          <button className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-gray-100 text-gray-500 hover:text-black transition-colors">
            <span className="material-symbols-outlined icon-sm">
              notifications
            </span>
          </button>
          <button className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-gray-100 text-gray-500 hover:text-black transition-colors">
            <span className="material-symbols-outlined icon-sm">help</span>
          </button>
          <div className="w-8 h-8 rounded-full border border-gray-200 overflow-hidden ml-2">
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img
              alt="User Avatar"
              className="w-full h-full object-cover"
              src="https://lh3.googleusercontent.com/aida-public/AB6AXuDU-DdYBfrFz-om3NR3ti3vMwzqmnGUGZLKIiUxgjgXeggfaNKkY4I7KzszndsvY7r90cccF3eWELBKnYVytB6PDtTlC9zAwd6ULKKLUmvHlt76S9XdpTsG_v3MgdW5thM63xoMm-gknjo3UFZkCpDmnYnerCiDaIGG4_5FjTWyrXPqf5Z_UMWcgXrWelxirf9_Ne6wWI52X_af3MNcsIOQe-tBE9EeO01HQX6mLI9Ovlagabo_xz1alYPg0osyOjcZMQFRlhTLo83t"
            />
          </div>
        </div>
      </header>
      <main className="flex-1 flex flex-col overflow-hidden max-w-7xl mx-auto w-full px-6 lg:px-8 py-8">
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 mb-8">
          <div>
            <h2 className="text-3xl font-bold text-black tracking-tight mb-1">
              My notebooks
            </h2>
            <p className="text-gray-500 text-sm">
              Manage and organize your AI research projects.
            </p>
          </div>
          <Link
            href="/editor"
            className="flex items-center gap-2 px-5 py-2.5 bg-accent-main text-white border border-black shadow-hard-sm hover:shadow-hard hover:-translate-y-0.5 active:translate-y-0 active:shadow-none transition-all rounded-lg font-semibold text-sm"
          >
            <span className="material-symbols-outlined icon-sm">add</span>
            Create new
          </Link>
        </div>
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6 border-b border-gray-200 pb-1">
          <div className="flex gap-6">
            <button className="pb-3 border-b-2 border-transparent hover:border-gray-300 text-gray-500 hover:text-black text-sm font-medium transition-colors">
              All
            </button>
            <button className="pb-3 border-b-2 border-accent-main text-black text-sm font-semibold">
              My notebooks
            </button>
            <button className="pb-3 border-b-2 border-transparent hover:border-gray-300 text-gray-500 hover:text-black text-sm font-medium transition-colors">
              Featured
            </button>
            <button className="pb-3 border-b-2 border-transparent hover:border-gray-300 text-gray-500 hover:text-black text-sm font-medium transition-colors">
              Shared with me
            </button>
          </div>
          <div className="flex items-center gap-2 pb-2">
            <div className="flex bg-gray-100 p-1 rounded-lg border border-gray-200">
              <button className="p-1 bg-white shadow-sm rounded text-black">
                <span className="material-symbols-outlined icon-sm">
                  view_list
                </span>
              </button>
              <button className="p-1 text-gray-500 hover:text-black">
                <span className="material-symbols-outlined icon-sm">
                  grid_view
                </span>
              </button>
            </div>
            <button className="flex items-center gap-1 px-3 py-1.5 border border-gray-200 rounded-lg text-xs font-medium text-gray-600 hover:border-black hover:text-black bg-white transition-colors">
              <span>Most recent</span>
              <span className="material-symbols-outlined icon-sm text-[16px]">
                expand_more
              </span>
            </button>
          </div>
        </div>
        <div className="flex-1 overflow-auto bg-white border border-gray-200 rounded-xl shadow-sm">
          <table className="w-full text-left border-collapse">
            <thead className="bg-gray-50 text-xs uppercase text-gray-500 font-semibold tracking-wider sticky top-0 z-10">
              <tr>
                <th className="px-6 py-4 border-b border-gray-200 font-medium w-1/2">
                  Title
                </th>
                <th className="px-6 py-4 border-b border-gray-200 font-medium">
                  Sources
                </th>
                <th className="px-6 py-4 border-b border-gray-200 font-medium">
                  Created
                </th>
                <th className="px-6 py-4 border-b border-gray-200 font-medium">
                  Role
                </th>
                <th className="px-6 py-4 border-b border-gray-200 font-medium text-right">
                  Action
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100 text-sm">
              {/* Row 1 */}
              <tr className="group table-row-hover transition-colors cursor-pointer">
                <td className="px-6 py-4">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded bg-blue-50 text-blue-600 flex items-center justify-center border border-blue-100">
                      <span className="material-symbols-outlined icon-sm">
                        search
                      </span>
                    </div>
                    <span className="font-semibold text-gray-900">
                      Elasticsearch Report
                    </span>
                  </div>
                </td>
                <td className="px-6 py-4 text-gray-500 font-mono text-xs">
                  16 Sources
                </td>
                <td className="px-6 py-4 text-gray-500">Feb 7, 2026</td>
                <td className="px-6 py-4">
                  <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
                    Owner
                  </span>
                </td>
                <td className="px-6 py-4 text-right">
                  <button className="text-gray-400 hover:text-black p-1 rounded hover:bg-gray-100 opacity-0 group-hover:opacity-100 transition-all">
                    <span className="material-symbols-outlined icon-sm">
                      more_vert
                    </span>
                  </button>
                </td>
              </tr>
              {/* Row 2 */}
              <tr className="group table-row-hover transition-colors cursor-pointer bg-accent-light/30">
                <td className="px-6 py-4">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded bg-accent-light text-accent-secondary flex items-center justify-center border border-accent-main shadow-hard-sm">
                      <span className="material-symbols-outlined icon-sm">
                        smart_toy
                      </span>
                    </div>
                    <span className="font-bold text-black">
                      Agentic AI Overview
                    </span>
                    <span className="ml-2 w-2 h-2 bg-accent-main rounded-full"></span>
                  </div>
                </td>
                <td className="px-6 py-4 text-gray-900 font-bold font-mono text-xs">
                  9 Sources
                </td>
                <td className="px-6 py-4 text-gray-900 font-medium">
                  Feb 6, 2026
                </td>
                <td className="px-6 py-4">
                  <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-accent-light text-accent-secondary border border-accent-main/30">
                    Owner
                  </span>
                </td>
                <td className="px-6 py-4 text-right">
                  <button className="text-black p-1 rounded hover:bg-accent-light transition-all">
                    <span className="material-symbols-outlined icon-sm">
                      more_vert
                    </span>
                  </button>
                </td>
              </tr>
              {/* Row 3 */}
              <tr className="group table-row-hover transition-colors cursor-pointer">
                <td className="px-6 py-4">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded bg-purple-50 text-purple-600 flex items-center justify-center border border-purple-100">
                      <span className="material-symbols-outlined icon-sm">
                        neurology
                      </span>
                    </div>
                    <span className="font-semibold text-gray-900">
                      Search Learn Knowledge
                    </span>
                  </div>
                </td>
                <td className="px-6 py-4 text-gray-500 font-mono text-xs">
                  56 Sources
                </td>
                <td className="px-6 py-4 text-gray-500">Feb 6, 2026</td>
                <td className="px-6 py-4">
                  <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
                    Owner
                  </span>
                </td>
                <td className="px-6 py-4 text-right">
                  <button className="text-gray-400 hover:text-black p-1 rounded hover:bg-gray-100 opacity-0 group-hover:opacity-100 transition-all">
                    <span className="material-symbols-outlined icon-sm">
                      more_vert
                    </span>
                  </button>
                </td>
              </tr>
              {/* Row 4 */}
              <tr className="group table-row-hover transition-colors cursor-pointer">
                <td className="px-6 py-4">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded bg-orange-50 text-orange-600 flex items-center justify-center border border-orange-100">
                      <span className="material-symbols-outlined icon-sm">
                        local_fire_department
                      </span>
                    </div>
                    <span className="font-semibold text-gray-900">
                      Game VFX
                    </span>
                  </div>
                </td>
                <td className="px-6 py-4 text-gray-500 font-mono text-xs">
                  1 Source
                </td>
                <td className="px-6 py-4 text-gray-500">Jan 28, 2026</td>
                <td className="px-6 py-4">
                  <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
                    Owner
                  </span>
                </td>
                <td className="px-6 py-4 text-right">
                  <button className="text-gray-400 hover:text-black p-1 rounded hover:bg-gray-100 opacity-0 group-hover:opacity-100 transition-all">
                    <span className="material-symbols-outlined icon-sm">
                      more_vert
                    </span>
                  </button>
                </td>
              </tr>
              {/* Row 5 */}
              <tr className="group table-row-hover transition-colors cursor-pointer">
                <td className="px-6 py-4">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded bg-emerald-50 text-emerald-600 flex items-center justify-center border border-emerald-100">
                      <span className="material-symbols-outlined icon-sm">
                        psychology
                      </span>
                    </div>
                    <span className="font-semibold text-gray-900">
                      Cognitive Frameworks
                    </span>
                  </div>
                </td>
                <td className="px-6 py-4 text-gray-500 font-mono text-xs">
                  12 Sources
                </td>
                <td className="px-6 py-4 text-gray-500">Jan 15, 2026</td>
                <td className="px-6 py-4">
                  <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
                    Member
                  </span>
                </td>
                <td className="px-6 py-4 text-right">
                  <button className="text-gray-400 hover:text-black p-1 rounded hover:bg-gray-100 opacity-0 group-hover:opacity-100 transition-all">
                    <span className="material-symbols-outlined icon-sm">
                      more_vert
                    </span>
                  </button>
                </td>
              </tr>
              {/* Row 6 */}
              <tr className="group table-row-hover transition-colors cursor-pointer">
                <td className="px-6 py-4">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded bg-slate-50 text-slate-600 flex items-center justify-center border border-slate-200">
                      <span className="material-symbols-outlined icon-sm">
                        draft
                      </span>
                    </div>
                    <span className="font-semibold text-gray-900">
                      Untitled Notebook
                    </span>
                  </div>
                </td>
                <td className="px-6 py-4 text-gray-500 font-mono text-xs">
                  0 Sources
                </td>
                <td className="px-6 py-4 text-gray-500">Jan 10, 2026</td>
                <td className="px-6 py-4">
                  <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
                    Owner
                  </span>
                </td>
                <td className="px-6 py-4 text-right">
                  <button className="text-gray-400 hover:text-black p-1 rounded hover:bg-gray-100 opacity-0 group-hover:opacity-100 transition-all">
                    <span className="material-symbols-outlined icon-sm">
                      more_vert
                    </span>
                  </button>
                </td>
              </tr>
              <tr className="h-16">
                <td className="hatch-pattern opacity-30" colSpan={5}></td>
              </tr>
            </tbody>
          </table>
        </div>
        <div className="mt-4 flex justify-between items-center text-xs text-gray-400">
          <p>Showing 6 of 12 notebooks</p>
          <div className="flex gap-2">
            <button className="hover:text-black transition-colors">
              Privacy
            </button>
            <span>•</span>
            <button className="hover:text-black transition-colors">
              Terms
            </button>
          </div>
        </div>
      </main>
    </>
  );
}
