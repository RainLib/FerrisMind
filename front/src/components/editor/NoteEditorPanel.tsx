"use client";

import React, { useState } from "react";
import dynamic from "next/dynamic";
import { useNotebookStore, ActivityItem } from "@/store/notebookStore";

const MDEditor = dynamic(() => import("@uiw/react-md-editor"), { ssr: false });

export function NoteEditorPanel({ onToggle }: { onToggle?: () => void }) {
  const { activities, activeActivity, setActivities, setActiveActivity } =
    useNotebookStore();

  const currentNote = activities.find((a) => a.id === activeActivity?.id);
  const [content, setContent] = useState(currentNote?.content || "");
  const [title, setTitle] = useState(currentNote?.title || "Untitled Note");

  if (!currentNote) return null;

  const handleSave = (newContent: string) => {
    setContent(newContent);
    setActivities((prev) =>
      prev.map((a) =>
        a.id === currentNote.id ? { ...a, content: newContent, title } : a,
      ),
    );
  };

  const handleTitleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newTitle = e.target.value;
    setTitle(newTitle);
    setActivities((prev) =>
      prev.map((a) =>
        a.id === currentNote.id ? { ...a, title: newTitle } : a,
      ),
    );
  };

  const handleConvertToSource = async () => {
    // Basic logic mapping note content to a File object.
    const blob = new Blob([content], { type: "text/markdown" });
    const file = new File([blob], `${title}.md`, { type: "text/markdown" });

    // In a full implementation, this should hit POST /api/upload and then insert to sources.
    // For now, we mock the transition by indicating it's processing.
    alert(
      `Converting "${title}" to source! This will hit /api/upload shortly.`,
    );

    // Switch away from editor to view the source upload
    setActiveActivity(null);
  };

  return (
    <div className="flex flex-col h-full w-full bg-white relative">
      <div className="h-14 px-4 border-b border-border-main flex items-center gap-2 justify-between shrink-0 bg-stone-50">
        <div className="flex items-center gap-2 text-sm">
          <button
            onClick={() => setActiveActivity(null)}
            className="text-gray-500 hover:text-black hover:bg-gray-200 px-2 py-1 rounded transition-colors flex items-center gap-1 font-semibold"
          >
            <span className="material-symbols-outlined icon-sm">
              arrow_back
            </span>
            Studio
          </button>
          <span className="material-symbols-outlined icon-sm text-gray-400">
            chevron_right
          </span>
          <span className="text-gray-800 font-semibold truncate max-w-[200px]">
            Note
          </span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => {
              setActivities((prev) =>
                prev.filter((a) => a.id !== currentNote.id),
              );
              setActiveActivity(null);
            }}
            className="w-8 h-8 flex items-center justify-center text-gray-500 hover:text-red-600 hover:bg-red-50 rounded transition-colors"
            title="Delete Note"
          >
            <span className="material-symbols-outlined icon-sm">delete</span>
          </button>
          {onToggle && (
            <button
              onClick={onToggle}
              className="w-8 h-8 flex items-center justify-center text-gray-500 hover:bg-black hover:text-white rounded transition-colors group relative border border-transparent hover:border-black"
              title="Collapse Sidebar"
            >
              <span className="material-symbols-outlined icon-sm">
                dock_to_right
              </span>
            </button>
          )}
        </div>
      </div>

      <div className="flex-1 overflow-hidden flex flex-col pt-6 px-10 pb-4">
        <input
          type="text"
          value={title}
          onChange={handleTitleChange}
          className="text-3xl font-black text-gray-800 border-none outline-none focus:ring-0 bg-transparent mb-6 placeholder-gray-300 w-full"
          placeholder="Note Title..."
        />
        <div
          data-color-mode="light"
          className="flex-1 w-full overflow-hidden mb-4"
        >
          <MDEditor
            value={content}
            onChange={(val) => handleSave(val || "")}
            height="100%"
            preview="edit"
            className="w-full border-none! shadow-none!"
            style={{ backgroundColor: "transparent" }}
          />
        </div>
      </div>

      <div className="h-16 border-t border-border-main flex items-center px-6 shrink-0 bg-stone-50 justify-between">
        <span className="text-xs text-gray-400 font-medium tracking-wide">
          Auto-saved locally
        </span>
        <button
          onClick={handleConvertToSource}
          className="flex items-center gap-2 px-4 py-2.5 bg-black text-white text-xs font-bold rounded-[2px] shadow-hard-sm hover:shadow-hard transition-all hover:-translate-y-0.5 group"
        >
          <span className="material-symbols-outlined icon-sm group-hover:rotate-12 transition-transform">
            post_add
          </span>
          Convert to source
        </button>
      </div>
    </div>
  );
}
