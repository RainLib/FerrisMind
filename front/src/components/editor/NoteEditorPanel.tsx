"use client";

import React, { useState } from "react";
import { useNotebookStore } from "@/store/notebookStore";
import { LexicalEditor } from "./LexicalEditor";

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
    // In a full implementation, this should hit POST /api/upload and then insert to sources.
    // For now, we mock the transition by indicating it's processing.
    alert(
      `Converting "${title}" to source! This will hit /api/upload shortly.`,
    );

    // Switch away from editor to view the source upload
    setActiveActivity(null);
  };

  return (
    <div className="flex flex-col h-full w-full bg-bg-main relative">
      <div className="h-14 px-4 border-b border-border-bold flex items-center gap-2 justify-between shrink-0 bg-bg-studio">
        <div className="flex items-center gap-2 text-sm">
          <button
            onClick={() => setActiveActivity(null)}
            className="text-gray-500 hover:text-primary hover:bg-bg-sources px-2 py-1 rounded transition-colors flex items-center gap-1 font-semibold border border-transparent hover:border-border-light"
          >
            <span className="material-symbols-outlined icon-sm">
              arrow_back
            </span>
            Studio
          </button>
          <span className="material-symbols-outlined icon-sm text-gray-400">
            chevron_right
          </span>
          <span className="text-primary font-semibold truncate max-w-[200px]">
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
              className="w-8 h-8 flex items-center justify-center text-gray-500 hover:bg-primary hover:text-bg-main rounded transition-colors group relative border border-transparent hover:border-border-bold"
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
          className="text-3xl font-black text-primary border-none outline-none focus:ring-0 bg-transparent mb-6 placeholder-gray-400 w-full"
          placeholder="Note Title..."
        />
        <div className="flex-1 w-full overflow-hidden mb-4 relative z-0">
          <LexicalEditor
            initialMarkdown={content}
            onChange={(val) => handleSave(val)}
          />
        </div>
      </div>

      <div className="h-16 border-t border-border-bold flex items-center px-6 shrink-0 bg-bg-studio justify-between">
        <span className="text-xs text-gray-400 font-medium tracking-wide">
          Auto-saved locally
        </span>
        <button
          onClick={handleConvertToSource}
          className="flex items-center gap-2 px-4 py-2.5 bg-primary text-bg-main text-xs font-bold rounded-[2px] shadow-hard-sm hover:shadow-hard transition-all hover:-translate-y-0.5 group border border-border-bold"
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
