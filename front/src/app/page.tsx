"use client";

import Link from "next/link";
import { useState, useMemo, useEffect } from "react";
import { useRouter } from "next/navigation";
import { Logo } from "@/components/ui/logo";
import {
  fetchGraphQL,
  GET_NOTEBOOKS,
  CREATE_NOTEBOOK,
  UPDATE_NOTEBOOK,
  DELETE_NOTEBOOK,
  Notebook,
} from "@/lib/graphql";

// UI-only properties that aren't in the DB yet
interface NotebookUI extends Notebook {
  role: string;
  icon: string;
  iconColor: string;
  iconBg: string;
  type: string;
  sources: number;
}

// Mock UI properties helper
const mapToUI = (n: Notebook): NotebookUI => ({
  ...n,
  role: "Owner",
  icon: "description",
  iconColor: "text-blue-600",
  iconBg: "bg-blue-50",
  type: "My notebooks",
  sources: 0,
});

export default function Home() {
  const router = useRouter();
  const [searchTerm, setSearchTerm] = useState("");
  const [activeTab, setActiveTab] = useState("All");
  const [viewMode, setViewMode] = useState<"list" | "grid">("list");
  const [notebooks, setNotebooks] = useState<NotebookUI[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isCreating, setIsCreating] = useState(false);

  // Modal states
  const [editModalOpen, setEditModalOpen] = useState(false);
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);
  const [selectedNotebook, setSelectedNotebook] = useState<NotebookUI | null>(
    null,
  );

  // Form states
  const [formName, setFormName] = useState("");
  const [formDescription, setFormDescription] = useState("");

  // Dropdown state
  const [openDropdownId, setOpenDropdownId] = useState<string | null>(null);

  const loadNotebooks = async () => {
    setIsLoading(true);
    const { data, errors } = await fetchGraphQL<{ notebooks: Notebook[] }>(
      GET_NOTEBOOKS,
    );
    if (data) {
      setNotebooks(data.notebooks.map(mapToUI));
    } else if (errors) {
      console.error("Failed to load notebooks:", errors);
    }
    setIsLoading(false);
  };

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    loadNotebooks();
  }, []);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = () => setOpenDropdownId(null);
    document.addEventListener("click", handleClickOutside);
    return () => document.removeEventListener("click", handleClickOutside);
  }, []);

  const handleCreateNotebook = async () => {
    setIsCreating(true);
    try {
      const { data, errors } = await fetchGraphQL<{
        createNotebook: { id: string };
      }>(CREATE_NOTEBOOK, { name: "Untitled Notebook", description: "" });
      if (data) {
        router.push(`/notebook/${data.createNotebook.id}`);
      } else if (errors) {
        alert("Failed to create notebook: " + errors[0].message);
        setIsCreating(false);
      }
    } catch (e) {
      console.error("Failed to create notebook:", e);
      setIsCreating(false);
    }
  };

  const openEditModal = (notebook: NotebookUI, e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setSelectedNotebook(notebook);
    setFormName(notebook.name);
    setFormDescription(notebook.description || "");
    setOpenDropdownId(null);
    setEditModalOpen(true);
  };

  const openDeleteModal = (notebook: NotebookUI, e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setSelectedNotebook(notebook);
    setOpenDropdownId(null);
    setDeleteModalOpen(true);
  };

  const submitEditNotebook = async () => {
    if (!selectedNotebook) return;
    if (!formName.trim()) return alert("Name is required");
    const { data, errors } = await fetchGraphQL<{
      updateNotebook: { id: string };
    }>(UPDATE_NOTEBOOK, {
      id: selectedNotebook.id,
      name: formName,
      description: formDescription,
    });
    if (data) {
      setEditModalOpen(false);
      loadNotebooks();
    } else if (errors) {
      alert("Failed to update notebook: " + errors[0].message);
    }
  };

  const submitDeleteNotebook = async () => {
    if (!selectedNotebook) return;
    const { data, errors } = await fetchGraphQL<{
      deleteNotebook: boolean;
    }>(DELETE_NOTEBOOK, { id: selectedNotebook.id });
    if (data) {
      setDeleteModalOpen(false);
      loadNotebooks();
    } else if (errors) {
      alert("Failed to delete notebook: " + errors[0].message);
    }
  };

  const filteredNotebooks = useMemo(() => {
    return notebooks.filter((n: NotebookUI) => {
      const matchesSearch = n.name
        .toLowerCase()
        .includes(searchTerm.toLowerCase());
      const matchesTab = activeTab === "All" || n.type === activeTab;
      return matchesSearch && matchesTab;
    });
  }, [notebooks, searchTerm, activeTab]);

  return (
    <>
      <header className="h-16 shrink-0 border-b border-border-bold flex items-center justify-between px-6 bg-bg-main z-20 relative">
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 flex items-center justify-center">
            <Logo className="w-8 h-8 text-primary" />
          </div>
          <h1 className="font-bold text-lg tracking-tight uppercase text-primary">
            Neo Workspace
          </h1>
        </div>
        <div className="flex items-center gap-4">
          <div className="relative hidden md:block">
            <span className="material-symbols-outlined absolute left-3 top-2 text-gray-400 icon-sm">
              search
            </span>
            <input
              className="bg-bg-sources border border-border-light rounded-lg py-1.5 pl-9 pr-4 text-sm focus:border-border-bold focus:ring-0 transition-all w-64 placeholder-gray-400 text-primary"
              placeholder="Search notebooks..."
              type="text"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
            />
          </div>
          <button className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-bg-sources text-gray-500 hover:text-primary transition-colors">
            <span className="material-symbols-outlined icon-sm">
              notifications
            </span>
          </button>
          <button className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-bg-sources text-gray-500 hover:text-primary transition-colors">
            <span className="material-symbols-outlined icon-sm">help</span>
          </button>
          <div className="w-8 h-8 rounded-full border border-border-light overflow-hidden ml-2">
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
            <h2 className="text-3xl font-bold text-primary tracking-tight mb-1">
              My notebooks
            </h2>
            <p className="text-gray-500 text-sm">
              Manage and organize your AI research projects.
            </p>
          </div>
          <button
            onClick={handleCreateNotebook}
            disabled={isCreating}
            className="flex items-center gap-2 px-5 py-2.5 bg-accent-main text-bg-main border border-border-bold shadow-hard-sm hover:shadow-hard hover:-translate-y-0.5 active:translate-y-0 active:shadow-none transition-all rounded-lg font-semibold text-sm cursor-pointer disabled:opacity-70 disabled:hover:translate-y-0 disabled:hover:shadow-hard-sm"
          >
            <span
              className={`material-symbols-outlined icon-sm ${isCreating ? "animate-spin" : ""}`}
            >
              {isCreating ? "progress_activity" : "add"}
            </span>
            {isCreating ? "Creating..." : "Create new"}
          </button>
        </div>
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6 border-b border-border-light pb-1">
          <div className="flex gap-6">
            {["All", "My notebooks", "Featured", "Shared with me"].map(
              (tab) => (
                <button
                  key={tab}
                  onClick={() => setActiveTab(tab)}
                  className={`pb-3 border-b-2 text-sm transition-colors ${
                    activeTab === tab
                      ? "border-accent-main text-primary font-semibold"
                      : "border-transparent hover:border-border-light text-gray-500 hover:text-primary font-medium"
                  }`}
                >
                  {tab}
                </button>
              ),
            )}
          </div>
          <div className="flex items-center gap-2 pb-2">
            <div className="flex bg-bg-sources p-1 rounded-lg border border-border-light">
              <button
                onClick={() => setViewMode("list")}
                className={`p-1 rounded transition-all ${
                  viewMode === "list"
                    ? "bg-bg-main shadow-sm text-primary border border-border-light"
                    : "text-gray-500 hover:text-primary"
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
                    ? "bg-bg-main shadow-sm text-primary border border-border-light"
                    : "text-gray-500 hover:text-primary"
                }`}
              >
                <span className="material-symbols-outlined icon-sm">
                  grid_view
                </span>
              </button>
            </div>
            <button className="flex items-center gap-1 px-3 py-1.5 border border-border-light rounded-lg text-xs font-medium text-gray-600 hover:border-border-bold hover:text-primary bg-bg-main transition-colors">
              <span>Most recent</span>
              <span className="material-symbols-outlined icon-sm text-[16px]">
                expand_more
              </span>
            </button>
          </div>
        </div>
        {viewMode === "list" ? (
          <div className="flex-1 overflow-auto bg-bg-main border border-border-light rounded-xl shadow-sm">
            <table className="w-full text-left border-collapse">
              <thead className="bg-bg-sources text-xs uppercase text-gray-500 font-semibold tracking-wider sticky top-0 z-10">
                <tr>
                  <th className="px-6 py-4 border-b border-border-light font-medium w-1/2">
                    Title
                  </th>
                  <th className="px-6 py-4 border-b border-border-light font-medium">
                    Sources
                  </th>
                  <th className="px-6 py-4 border-b border-border-light font-medium">
                    Created
                  </th>
                  <th className="px-6 py-4 border-b border-border-light font-medium">
                    Role
                  </th>
                  <th className="px-6 py-4 border-b border-border-light font-medium text-right">
                    Action
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border-light/50 text-sm">
                {isLoading ? (
                  <tr>
                    <td colSpan={5} className="px-6 py-20 text-center">
                      <div className="flex flex-col items-center gap-2 text-gray-400">
                        <span className="material-symbols-outlined text-4xl animate-spin">
                          progress_activity
                        </span>
                        <p>Loading notebooks...</p>
                      </div>
                    </td>
                  </tr>
                ) : filteredNotebooks.length === 0 ? (
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
                ) : (
                  filteredNotebooks.map((n) => (
                    <tr
                      key={n.id}
                      className="group hover:bg-bg-sources/50 transition-colors cursor-pointer"
                      onClick={() => router.push(`/notebook/${n.id}`)}
                    >
                      <td className="px-6 py-4">
                        <div className="flex items-center gap-3 w-full h-full">
                          <div
                            className={`w-8 h-8 rounded ${n.iconBg} ${n.iconColor} flex items-center justify-center border border-border-light`}
                          >
                            <span className="material-symbols-outlined icon-sm">
                              {n.icon}
                            </span>
                          </div>
                          <span className="font-semibold text-primary">
                            {n.name}
                          </span>
                        </div>
                      </td>
                      <td className="px-6 py-4 text-gray-500 font-mono text-xs">
                        {n.sources} Sources
                      </td>
                      <td className="px-6 py-4 text-gray-500">
                        {new Date(n.createdAt).toLocaleDateString()}
                      </td>
                      <td className="px-6 py-4">
                        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-bg-sources text-primary border border-border-light">
                          {n.role}
                        </span>
                      </td>
                      <td className="px-6 py-4 text-right relative">
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            setOpenDropdownId(
                              openDropdownId === n.id ? null : n.id,
                            );
                          }}
                          className="text-gray-400 hover:text-primary p-1 rounded hover:bg-bg-sources opacity-0 group-hover:opacity-100 transition-all border border-transparent hover:border-border-light"
                        >
                          <span className="material-symbols-outlined icon-sm">
                            more_vert
                          </span>
                        </button>
                        {openDropdownId === n.id && (
                          <div className="absolute right-6 top-10 w-32 bg-bg-main border border-border-bold shadow-hard-sm rounded-lg z-50 py-1 flex flex-col items-start overflow-hidden">
                            <button
                              onClick={(e) => openEditModal(n, e)}
                              className="w-full text-left px-4 py-2 text-sm hover:bg-bg-sources transition-colors text-primary font-medium"
                            >
                              Rename
                            </button>
                            <button
                              onClick={(e) => openDeleteModal(n, e)}
                              className="w-full text-left px-4 py-2 text-sm hover:bg-red-500/10 hover:text-red-500 transition-colors text-red-600 font-medium"
                            >
                              Delete
                            </button>
                          </div>
                        )}
                      </td>
                    </tr>
                  ))
                )}
                <tr className="h-16">
                  <td className="hatch-pattern opacity-30" colSpan={5}></td>
                </tr>
              </tbody>
            </table>
          </div>
        ) : (
          <div className="flex-1 overflow-auto grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
            {isLoading ? (
              Array.from({ length: 6 }).map((_, i) => (
                <div
                  key={i}
                  className="h-48 bg-bg-sources border border-border-light rounded-xl animate-pulse"
                />
              ))
            ) : filteredNotebooks.length === 0 ? (
              <div className="col-span-full h-64 flex flex-col items-center justify-center gap-2 text-gray-400 border-2 border-dashed border-border-light rounded-xl bg-bg-sources/20">
                <span className="material-symbols-outlined text-4xl">
                  inbox
                </span>
                <p>No notebooks found</p>
              </div>
            ) : (
              filteredNotebooks.map((n) => (
                <Link
                  href={`/notebook/${n.id}`}
                  key={n.id}
                  className="group p-5 bg-bg-main border border-border-light rounded-xl shadow-sm hover:shadow-hard-sm hover:border-border-bold hover:-translate-y-1 transition-all flex flex-col justify-between"
                >
                  <div className="flex items-start justify-between mb-4">
                    <div
                      className={`w-12 h-12 rounded-lg ${n.iconBg} ${n.iconColor} flex items-center justify-center border border-border-light shadow-sm bg-bg-sources/30`}
                    >
                      <span className="material-symbols-outlined text-2xl">
                        {n.icon}
                      </span>
                    </div>
                    <div className="relative">
                      <button
                        onClick={(e) => {
                          e.preventDefault();
                          e.stopPropagation();
                          setOpenDropdownId(
                            openDropdownId === n.id ? null : n.id,
                          );
                        }}
                        className="text-gray-400 hover:text-primary p-1 rounded hover:bg-bg-sources transition-colors border border-transparent hover:border-border-light"
                      >
                        <span className="material-symbols-outlined icon-sm">
                          more_vert
                        </span>
                      </button>
                      {openDropdownId === n.id && (
                        <div className="absolute right-0 top-8 w-32 bg-bg-main border border-border-bold shadow-hard-sm rounded-lg z-50 py-1 flex flex-col items-start overflow-hidden">
                          <button
                            onClick={(e) => openEditModal(n, e)}
                            className="w-full text-left px-4 py-2 text-sm hover:bg-bg-sources transition-colors text-primary font-medium"
                          >
                            Rename
                          </button>
                          <button
                            onClick={(e) => openDeleteModal(n, e)}
                            className="w-full text-left px-4 py-2 text-sm hover:bg-red-500/10 hover:text-red-500 transition-colors text-red-600 font-medium"
                          >
                            Delete
                          </button>
                        </div>
                      )}
                    </div>
                  </div>
                  <div>
                    <h3 className="font-bold text-primary text-lg mb-1">
                      {n.name}
                    </h3>
                    <p className="text-xs text-gray-500 font-mono">
                      {n.sources} SOURCES •{" "}
                      {new Date(n.createdAt).toLocaleDateString()}
                    </p>
                  </div>
                </Link>
              ))
            )}
          </div>
        )}
        <div className="mt-4 flex justify-between items-center text-xs text-gray-400">
          <p>Showing {filteredNotebooks.length} notebooks</p>
          <div className="flex gap-2">
            <button className="hover:text-primary transition-colors">
              Privacy
            </button>
            <span>•</span>
            <button className="hover:text-primary transition-colors">
              Terms
            </button>
          </div>
        </div>
      </main>

      {/* RENAME MODAL */}
      {editModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-bg-main/60 backdrop-blur-sm p-4">
          <div className="bg-bg-main border border-border-bold shadow-modal rounded-xl w-full max-w-md overflow-hidden">
            <div className="px-6 py-4 border-b border-border-light">
              <h3 className="font-bold text-lg text-primary tracking-tight">
                Rename Notebook
              </h3>
            </div>
            <div className="p-6 flex flex-col gap-4">
              <div>
                <label className="text-[10px] font-bold uppercase tracking-widest text-gray-500 mb-1 block">
                  Notebook Name *
                </label>
                <input
                  type="text"
                  value={formName}
                  onChange={(e) => setFormName(e.target.value)}
                  className="w-full border border-border-light rounded-lg px-3 py-2 text-sm focus:border-border-bold bg-bg-sources text-primary focus:ring-0 transition-colors"
                  placeholder="e.g. Q4 Financial Research"
                  autoFocus
                />
              </div>
              <div>
                <label className="text-[10px] font-bold uppercase tracking-widest text-gray-500 mb-1 block">
                  Description (Optional)
                </label>
                <textarea
                  value={formDescription}
                  onChange={(e) => setFormDescription(e.target.value)}
                  className="w-full border border-border-light rounded-lg px-3 py-2 text-sm focus:border-border-bold bg-bg-sources text-primary focus:ring-0 transition-colors h-24 resize-none"
                  placeholder="What is this notebook about?"
                />
              </div>
            </div>
            <div className="px-6 py-4 bg-bg-sources border-t border-border-light flex justify-end gap-3">
              <button
                onClick={() => setEditModalOpen(false)}
                className="px-4 py-2 border border-border-light rounded-lg text-sm font-medium hover:bg-bg-main transition-colors text-primary"
              >
                Cancel
              </button>
              <button
                onClick={submitEditNotebook}
                disabled={!formName.trim()}
                className="px-4 py-2 border border-border-bold shadow-hard-sm bg-accent-main text-bg-main rounded-lg text-sm font-semibold hover:-translate-y-0.5 hover:shadow-hard transition-all disabled:opacity-50 disabled:hover:translate-y-0 disabled:hover:shadow-hard-sm"
              >
                Save Changes
              </button>
            </div>
          </div>
        </div>
      )}

      {/* DELETE MODAL */}
      {deleteModalOpen && selectedNotebook && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-bg-main/60 backdrop-blur-sm p-4">
          <div className="bg-bg-main border border-red-500 shadow-modal rounded-xl w-full max-w-md overflow-hidden">
            <div className="px-6 py-4 border-b border-red-500/20 bg-red-500/10">
              <h3 className="font-bold text-lg text-red-500 tracking-tight flex items-center gap-2">
                <span className="material-symbols-outlined icon-sm">
                  warning
                </span>
                Delete Notebook
              </h3>
            </div>
            <div className="p-6">
              <p className="text-gray-500 text-sm mb-4">
                Are you sure you want to delete{" "}
                <span className="font-bold text-primary border border-border-light px-1 py-0.5 rounded bg-bg-sources">
                  {selectedNotebook.name}
                </span>
                ?
              </p>
              <p className="text-xs text-red-500/80">
                This action cannot be undone. All documents and chat histories
                inside this notebook will be immediately removed.
              </p>
            </div>
            <div className="px-6 py-4 bg-bg-sources border-t border-border-light flex justify-end gap-3">
              <button
                onClick={() => setDeleteModalOpen(false)}
                className="px-4 py-2 border border-border-light rounded-lg text-sm font-medium hover:bg-bg-main transition-colors text-primary"
              >
                Cancel
              </button>
              <button
                onClick={submitDeleteNotebook}
                className="px-4 py-2 border border-border-bold shadow-hard-sm bg-red-600 text-bg-main rounded-lg text-sm font-semibold hover:-translate-y-0.5 hover:shadow-hard hover:bg-red-700 transition-all"
              >
                Yes, Delete It
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
