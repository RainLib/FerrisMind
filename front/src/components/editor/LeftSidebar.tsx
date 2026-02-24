import * as React from "react";
import { useState, useRef, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { AddSourceModal } from "@/components/editor/AddSourceModal";
import { PanelDetailView } from "./PanelDetailView";

interface Source {
  id: string;
  icon: string;
  title: string;
  sub: string;
}

interface SourceItemProps {
  source: Source;
  selected: boolean;
  onSelectToggle: (id: string) => void;
  onItemClick: (id: string) => void;
  onDelete: (id: string) => void;
  onRename: (id: string, newTitle: string) => void;
}

function SourceItem({
  source,
  selected,
  onSelectToggle,
  onItemClick,
  onDelete,
  onRename,
}: SourceItemProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editTitle, setEditTitle] = useState(source.title);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isEditing]);

  const handleRenameSubmit = () => {
    if (editTitle.trim() !== "") {
      onRename(source.id, editTitle.trim());
    } else {
      setEditTitle(source.title);
    }
    setIsEditing(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleRenameSubmit();
    } else if (e.key === "Escape") {
      setEditTitle(source.title);
      setIsEditing(false);
    }
  };

  return (
    <div
      className={`group flex items-center gap-3 p-3 bg-white shadow-sm cursor-pointer transition-all ${
        selected
          ? "border-l-2 border-l-accent-main border-y border-r border-gray-200 hover:border-black"
          : "border border-gray-200 hover:border-black"
      }`}
      onClick={() => onItemClick(source.id)}
    >
      {/* Checkbox */}
      <div
        className="shrink-0 flex items-center justify-center p-1 -m-1"
        onClick={(e) => {
          e.stopPropagation();
          onSelectToggle(source.id);
        }}
      >
        <div
          className={`w-4 h-4 rounded-sm border flex items-center justify-center transition-colors ${
            selected
              ? "bg-accent-main border-accent-main text-white"
              : "border-gray-300 bg-white group-hover:border-black"
          }`}
        >
          {selected && (
            <span className="material-symbols-outlined text-[12px] font-bold">
              check
            </span>
          )}
        </div>
      </div>

      <div
        className={`p-1.5 rounded-sm shrink-0 ${
          selected
            ? "bg-accent-light text-accent-secondary"
            : "bg-gray-50 text-gray-400 group-hover:text-black transition-colors"
        }`}
      >
        <span className="material-symbols-outlined icon-sm">{source.icon}</span>
      </div>

      <div className="flex-1 min-w-0">
        {isEditing ? (
          <input
            ref={inputRef}
            type="text"
            value={editTitle}
            onChange={(e) => setEditTitle(e.target.value)}
            onBlur={handleRenameSubmit}
            onKeyDown={handleKeyDown}
            onClick={(e) => e.stopPropagation()}
            className="w-full text-sm font-bold border-b border-black outline-none bg-transparent focus:border-accent-main p-0 m-0"
          />
        ) : (
          <p
            className={`text-sm tracking-tight truncate ${
              selected
                ? "font-bold"
                : "font-semibold text-gray-700 group-hover:text-black"
            }`}
            onDoubleClick={(e) => {
              e.stopPropagation();
              setIsEditing(true);
            }}
            title="Double click to rename"
          >
            {source.title}
          </p>
        )}
        <p className="text-xs text-gray-500 truncate font-medium">
          {source.sub}
        </p>
      </div>

      {/* Actions */}
      <div
        className="hidden group-hover:flex items-center gap-1 shrink-0"
        onClick={(e) => e.stopPropagation()}
      >
        <button
          onClick={() => setIsEditing(true)}
          className="p-1 text-gray-400 hover:text-black transition-colors relative group/btn"
        >
          <span className="material-symbols-outlined text-[16px]">edit</span>
          <div className="absolute top-full mt-2 left-1/2 -translate-x-1/2 bg-black text-white text-[10px] font-bold px-2 py-1 whitespace-nowrap opacity-0 group-hover/btn:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm border border-black hidden group-hover/btn:block">
            Rename
          </div>
        </button>
        <button
          onClick={() => onDelete(source.id)}
          className="p-1 text-gray-400 hover:text-red-600 transition-colors relative group/btn"
        >
          <span className="material-symbols-outlined text-[16px]">delete</span>
          <div className="absolute top-full mt-2 left-1/2 -translate-x-1/2 bg-black text-white text-[10px] font-bold px-2 py-1 whitespace-nowrap opacity-0 group-hover/btn:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm border border-black hidden group-hover/btn:block">
            Delete
          </div>
        </button>
      </div>
    </div>
  );
}

interface LeftSidebarProps {
  isMobile?: boolean;
  onToggle?: () => void;
}

export function LeftSidebar({ isMobile, onToggle }: LeftSidebarProps) {
  const [isAddSourceModalOpen, setIsAddSourceModalOpen] = useState(false);
  const [activeDetailId, setActiveDetailId] = useState<string | null>(null);
  // Default to empty for a new notebook to trigger the modal automatically
  const [sources, setSources] = useState<Source[]>([]);

  // Auto-open modal if no sources exist
  useEffect(() => {
    let timeoutId: NodeJS.Timeout;
    if (sources.length === 0) {
      // Use timeout to bypass synchronous setState in effect warning
      timeoutId = setTimeout(() => setIsAddSourceModalOpen(true), 0);
    }
    return () => clearTimeout(timeoutId);
  }, [sources.length]);

  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  const handleSelectToggle = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const handleDelete = (id: string) => {
    setSources((prev) => prev.filter((s) => s.id !== id));
    setSelectedIds((prev) => {
      const next = new Set(prev);
      next.delete(id);
      return next;
    });
  };

  const handleRename = (id: string, newTitle: string) => {
    setSources((prev) =>
      prev.map((s) => (s.id === id ? { ...s, title: newTitle } : s)),
    );
  };

  const isAllSelected =
    sources.length > 0 && selectedIds.size === sources.length;

  const handleSelectAll = () => {
    if (isAllSelected) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(sources.map((s) => s.id)));
    }
  };

  const activeSource = sources.find((s) => s.id === activeDetailId);

  return (
    <aside className="w-full h-full flex flex-col bg-bg-sources border-r border-border-bold relative overflow-hidden">
      <div className="h-14 px-4 flex items-center justify-between border-b border-border-bold bg-bg-sources shrink-0">
        <h2 className="text-xs font-bold text-gray-500 uppercase tracking-widest overflow-hidden whitespace-nowrap text-ellipsis">
          Sources
        </h2>
        <div className="flex gap-2">
          <Button variant="icon" onClick={onToggle} className="relative group">
            <span className="material-symbols-outlined icon-sm">
              {isMobile ? "close" : "dock_to_left"}
            </span>
            <div className="absolute top-full mt-2 right-0 bg-black text-white text-[10px] font-bold px-2 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm border border-black hidden group-hover:block">
              {isMobile ? "Close Sidebar" : "Collapse Sidebar"}
            </div>
          </Button>
        </div>
      </div>
      <div className="px-5 py-5 space-y-4">
        <button
          onClick={() => setIsAddSourceModalOpen(true)}
          className="w-full py-3 px-4 bg-white border border-dashed border-black shadow-sm hover:shadow-md hover:border-solid hover:-translate-y-0.5 active:translate-y-0 active:shadow-none transition-all flex items-center justify-center gap-2 text-sm font-bold group text-gray-600 hover:text-black"
        >
          <span className="material-symbols-outlined icon-sm text-accent-main group-hover:text-black transition-colors">
            add
          </span>
          Add source
        </button>
        <div className="relative group">
          <span className="material-symbols-outlined absolute left-3 top-2.5 text-gray-400 z-10 icon-sm">
            search
          </span>
          <input
            className="w-full bg-white border border-gray-300 py-2 pl-10 pr-4 text-sm font-medium placeholder-gray-400 focus:border-black focus:ring-0 focus:shadow-hard-sm transition-all outline-none"
            placeholder="Filter sources..."
            type="text"
          />
        </div>
        <div className="flex gap-2 text-xs font-bold w-full overflow-x-auto pb-1 hide-scrollbar">
          <button className="px-3 py-1.5 bg-black text-white border border-black shadow-hard-sm shrink-0">
            All
          </button>
          <button className="px-3 py-1.5 bg-transparent text-gray-600 border border-gray-300 hover:border-black hover:text-black transition-colors shrink-0">
            PDF
          </button>
          <button className="px-3 py-1.5 bg-transparent text-gray-600 border border-gray-300 hover:border-black hover:text-black transition-colors shrink-0">
            Web
          </button>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto px-5 pb-4 space-y-2">
        <div className="flex items-center gap-2 mb-2 px-1 py-2">
          <div
            onClick={handleSelectAll}
            className={`w-4 h-4 rounded-sm border flex items-center justify-center transition-colors cursor-pointer ${
              isAllSelected
                ? "bg-black border-black text-white"
                : "border-gray-300 bg-white hover:border-black"
            }`}
          >
            {isAllSelected && (
              <span className="material-symbols-outlined text-[12px] font-bold">
                check
              </span>
            )}
          </div>
          <span className="text-[10px] font-bold uppercase tracking-wider text-gray-400">
            Select All ({selectedIds.size}/{sources.length})
          </span>
        </div>
        {/* Source items mapping */}
        {sources.map((source) => (
          <SourceItem
            key={source.id}
            source={source}
            selected={selectedIds.has(source.id)}
            onSelectToggle={handleSelectToggle}
            onItemClick={(id) => setActiveDetailId(id)}
            onDelete={handleDelete}
            onRename={handleRename}
          />
        ))}
      </div>

      {activeDetailId && activeSource && (
        <PanelDetailView
          title={activeSource.title}
          icon={activeSource.icon}
          onBack={() => setActiveDetailId(null)}
          headerActions={
            <Button variant="icon" title="Source Settings">
              <span className="material-symbols-outlined icon-sm">
                settings
              </span>
            </Button>
          }
        >
          <div className="p-5 flex flex-col gap-6">
            {/* Summary Section */}
            <div>
              <h3 className="text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-3">
                Summary
              </h3>
              <p className="text-sm text-gray-800 leading-relaxed font-medium">
                This document <strong>{activeSource.title}</strong> covers
                essential concepts regarding {activeSource.sub.toLowerCase()}.
                It provides an overview of best practices, key metrics, and
                actionable insights for building robust systems based on the
                uploaded content.
              </p>
            </div>

            {/* Key Topics Section */}
            <div>
              <h3 className="text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-3">
                Key Topics
              </h3>
              <div className="flex flex-wrap gap-2">
                {[
                  "System Architecture",
                  "Best Practices",
                  "Integration",
                  "Security Considerations",
                  "Performance",
                ].map((topic, i) => (
                  <span
                    key={i}
                    className="px-3 py-1.5 bg-gray-50 border border-gray-200 text-xs font-bold text-gray-700 hover:border-black hover:text-black cursor-pointer transition-colors shadow-sm"
                  >
                    {topic}
                  </span>
                ))}
              </div>
            </div>

            {/* Suggested Actions/Questions */}
            <div className="pt-2">
              <h3 className="text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-3">
                Suggested actions
              </h3>
              <div className="flex flex-col gap-2">
                <button className="text-left px-4 py-3 bg-white border border-gray-200 hover:border-black shadow-sm hover:shadow-hard-sm transition-all text-sm font-bold text-gray-800 group flex items-center justify-between">
                  <span>Help me understand this document</span>
                  <span className="material-symbols-outlined icon-sm text-gray-400 group-hover:text-black transition-colors">
                    arrow_forward
                  </span>
                </button>
                <button className="text-left px-4 py-3 bg-white border border-gray-200 hover:border-black shadow-sm hover:shadow-hard-sm transition-all text-sm font-bold text-gray-800 group flex items-center justify-between">
                  <span>Critique the concepts proposed here</span>
                  <span className="material-symbols-outlined icon-sm text-gray-400 group-hover:text-black transition-colors">
                    arrow_forward
                  </span>
                </button>
              </div>
            </div>
          </div>
        </PanelDetailView>
      )}

      <AddSourceModal
        isOpen={isAddSourceModalOpen}
        onClose={() => setIsAddSourceModalOpen(false)}
      />
    </aside>
  );
}
