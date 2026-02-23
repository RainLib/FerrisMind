"use client";

import Link from "next/link";
import { useState, useMemo } from "react";
import { Logo } from "@/components/ui/logo";

interface Notebook {
  id: string;
  title: string;
  sources: number;
  created: string;
  role: string;
  icon: string;
  iconColor: string;
  iconBg: string;
  type: string;
}

const INITIAL_NOTEBOOKS: Notebook[] = [
  {
    id: "1",
    title: "Elasticsearch Report",
    sources: 16,
    created: "Feb 7, 2026",
    role: "Owner",
    icon: "search",
    iconColor: "text-blue-600",
    iconBg: "bg-blue-50",
    type: "My notebooks",
  },
  {
    id: "2",
    title: "Agentic AI Overview",
    sources: 9,
    created: "Feb 6, 2026",
    role: "Owner",
    icon: "smart_toy",
    iconColor: "text-accent-secondary",
    iconBg: "bg-accent-light",
    type: "My notebooks",
  },
  {
    id: "3",
    title: "Search Learn Knowledge",
    sources: 56,
    created: "Feb 6, 2026",
    role: "Owner",
    icon: "neurology",
    iconColor: "text-purple-600",
    iconBg: "bg-purple-50",
    type: "Featured",
  },
  {
    id: "4",
    title: "Game VFX",
    sources: 1,
    created: "Jan 28, 2026",
    role: "Owner",
    icon: "local_fire_department",
    iconColor: "text-orange-600",
    iconBg: "bg-orange-50",
    type: "My notebooks",
  },
  {
    id: "5",
    title: "Cognitive Frameworks",
    sources: 12,
    created: "Jan 15, 2026",
    role: "Member",
    icon: "psychology",
    iconColor: "text-emerald-600",
    iconBg: "bg-emerald-50",
    type: "Shared with me",
  },
  {
    id: "6",
    title: "Untitled Notebook",
    sources: 0,
    created: "Jan 10, 2026",
    role: "Owner",
    icon: "draft",
    iconColor: "text-slate-600",
    iconBg: "bg-slate-50",
    type: "My notebooks",
  },
];

export default function Home() {
  const [searchTerm, setSearchTerm] = useState("");
  const [activeTab, setActiveTab] = useState("My notebooks");
  const [viewMode, setViewMode] = useState<"list" | "grid">("list");
  const [notebooks] = useState<Notebook[]>(INITIAL_NOTEBOOKS);

  const filteredNotebooks = useMemo(() => {
    return notebooks.filter((n) => {
      const matchesSearch = n.title
        .toLowerCase()
        .includes(searchTerm.toLowerCase());
      const matchesTab = activeTab === "All" || n.type === activeTab;
      return matchesSearch && matchesTab;
    });
  }, [notebooks, searchTerm, activeTab]);

  return (
    <>
      <header className="h-16 shrink-0 border-b border-border-bold flex items-center justify-between px-6 bg-white z-20 relative">
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 flex items-center justify-center">
            <Logo className="w-8 h-8 text-black" />
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
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
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
            {["All", "My notebooks", "Featured", "Shared with me"].map(
              (tab) => (
                <button
                  key={tab}
                  onClick={() => setActiveTab(tab)}
                  className={`pb-3 border-b-2 text-sm transition-colors ${
                    activeTab === tab
                      ? "border-accent-main text-black font-semibold"
                      : "border-transparent hover:border-gray-300 text-gray-500 hover:text-black font-medium"
                  }`}
                >
                  {tab}
                </button>
              ),
            )}
          </div>
          <div className="flex items-center gap-2 pb-2">
            <div className="flex bg-gray-100 p-1 rounded-lg border border-gray-200">
              <button
                onClick={() => setViewMode("list")}
                className={`p-1 rounded transition-all ${
                  viewMode === "list"
                    ? "bg-white shadow-sm text-black"
                    : "text-gray-500 hover:text-black"
                }`}
              >
                <span className="material-symbols-outlined icon-sm">
                  view_list
                </span>
              </button>
              <button
                onClick={() => setViewMode("grid")}
                className={`p-1 rounded transition-all ${
                  viewMode === "grid"
                    ? "bg-white shadow-sm text-black"
                    : "text-gray-500 hover:text-black"
                }`}
              >
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
        {viewMode === "list" ? (
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
                {filteredNotebooks.map((n) => (
                  <tr
                    key={n.id}
                    className="group table-row-hover transition-colors cursor-pointer"
                  >
                    <td className="px-6 py-4">
                      <Link
                        href="/editor"
                        className="flex items-center gap-3 w-full h-full"
                      >
                        <div
                          className={`w-8 h-8 rounded ${n.iconBg} ${n.iconColor} flex items-center justify-center border border-gray-100`}
                        >
                          <span className="material-symbols-outlined icon-sm">
                            {n.icon}
                          </span>
                        </div>
                        <span className="font-semibold text-gray-900">
                          {n.title}
                        </span>
                      </Link>
                    </td>
                    <td className="px-6 py-4 text-gray-500 font-mono text-xs">
                      {n.sources} Sources
                    </td>
                    <td className="px-6 py-4 text-gray-500">{n.created}</td>
                    <td className="px-6 py-4">
                      <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
                        {n.role}
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
                ))}
                {filteredNotebooks.length === 0 && (
                  <tr>
                    <td colSpan={5} className="px-6 py-20 text-center">
                      <div className="flex flex-col items-center gap-2 text-gray-400">
                        <span className="material-symbols-outlined text-4xl">
                          inbox
                        </span>
                        <p>No notebooks found</p>
                      </div>
                    </td>
                  </tr>
                )}
                <tr className="h-16">
                  <td className="hatch-pattern opacity-30" colSpan={5}></td>
                </tr>
              </tbody>
            </table>
          </div>
        ) : (
          <div className="flex-1 overflow-auto grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
            {filteredNotebooks.map((n) => (
              <Link
                href="/editor"
                key={n.id}
                className="group p-5 bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-hard-sm hover:border-black hover:-translate-y-1 transition-all flex flex-col justify-between"
              >
                <div className="flex items-start justify-between mb-4">
                  <div
                    className={`w-12 h-12 rounded-lg ${n.iconBg} ${n.iconColor} flex items-center justify-center border border-gray-100 shadow-sm`}
                  >
                    <span className="material-symbols-outlined text-2xl">
                      {n.icon}
                    </span>
                  </div>
                  <button className="text-gray-400 hover:text-black">
                    <span className="material-symbols-outlined icon-sm">
                      more_vert
                    </span>
                  </button>
                </div>
                <div>
                  <h3 className="font-bold text-black text-lg mb-1">
                    {n.title}
                  </h3>
                  <p className="text-xs text-gray-500 font-mono">
                    {n.sources} SOURCES • {n.created}
                  </p>
                </div>
              </Link>
            ))}
            {filteredNotebooks.length === 0 && (
              <div className="col-span-full h-64 flex flex-col items-center justify-center gap-2 text-gray-400 border-2 border-dashed border-gray-100 rounded-xl">
                <span className="material-symbols-outlined text-4xl">
                  inbox
                </span>
                <p>No notebooks found</p>
              </div>
            )}
          </div>
        )}
        <div className="mt-4 flex justify-between items-center text-xs text-gray-400">
          <p>
            Showing {filteredNotebooks.length} of {notebooks.length} notebooks
          </p>
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
