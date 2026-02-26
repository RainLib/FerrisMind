import { useState, useRef, useEffect } from "react";
import { cn } from "@/lib/utils";
import { PanelDetailView } from "./PanelDetailView";
import { useNotebookStore, ActivityItem } from "@/store/notebookStore";
import { NoteEditorPanel } from "./NoteEditorPanel";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";

interface RightSidebarProps {
  isExpanded?: boolean;
  onToggle?: () => void;
}

export function RightSidebar({
  isExpanded = true,
  onToggle,
}: RightSidebarProps) {
  const {
    sources,
    activities,
    activeActivity,
    addActivity,
    setActiveActivity,
    setActivities,
  } = useNotebookStore();
  const hasSources = sources.length > 0;
  const [activeTool, setActiveTool] = useState<string | null>(null);

  const [editingId, setEditingId] = useState<string | null>(null);
  const [editTitle, setEditTitle] = useState("");
  const editInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (editingId && editInputRef.current) {
      editInputRef.current.focus();
    }
  }, [editingId]);

  const handleToolClick = (toolId: string) => {
    if (!isExpanded && onToggle) onToggle();
    setActiveTool(toolId);
  };

  const handleAddNote = () => {
    if (!isExpanded && onToggle) onToggle();
    const newNoteId = `note-${Date.now()}`;
    const newNote: ActivityItem = {
      id: newNoteId,
      type: "note",
      title: `New Note ${Math.floor(Math.random() * 1000)}`,
      createdAt: new Date(),
      content: "",
    };
    addActivity(newNote);
    setActiveActivity({ id: newNoteId, type: "note" });
  };

  const saveRename = (id: string) => {
    if (editTitle.trim()) {
      setActivities((prev) =>
        prev.map((a) => (a.id === id ? { ...a, title: editTitle.trim() } : a)),
      );
    }
    setEditingId(null);
  };

  const handleKeyDown = (e: React.KeyboardEvent, id: string) => {
    if (e.key === "Enter") saveRename(id);
    if (e.key === "Escape") setEditingId(null);
  };

  if (activeActivity?.type === "note") {
    return (
      <aside
        className={cn(
          "flex flex-col h-full bg-bg-main border-l border-border-bold relative w-full",
        )}
      >
        {isExpanded ? (
          <NoteEditorPanel key={activeActivity?.id} onToggle={onToggle} />
        ) : (
          <div className="h-14 px-4 flex items-center justify-between border-b border-border-bold bg-bg-studio shrink-0">
            <button
              onClick={onToggle}
              className="mx-auto text-gray-500 hover:bg-primary hover:text-bg-main border border-transparent hover:border-border-bold transition-all p-1 relative group"
            >
              <span className="material-symbols-outlined icon-sm">
                dock_to_left
              </span>
            </button>
          </div>
        )}
      </aside>
    );
  }

  return (
    <aside
      className={cn(
        "flex flex-col h-full bg-bg-studio border-l border-border-bold relative w-full",
      )}
    >
      <div className="h-14 px-4 flex items-center justify-between border-b border-border-bold bg-bg-studio shrink-0">
        {isExpanded && (
          <h2 className="text-xs font-bold text-gray-500 uppercase tracking-widest whitespace-nowrap overflow-hidden">
            Studio
          </h2>
        )}
        <button
          onClick={onToggle}
          className={cn(
            "text-gray-500 hover:bg-primary hover:text-bg-main border border-transparent hover:border-border-bold transition-all p-1 relative group",
            !isExpanded && "mx-auto",
          )}
        >
          <span className="material-symbols-outlined icon-sm">
            {isExpanded ? "dock_to_right" : "dock_to_left"}
          </span>
          <div className="absolute top-full mt-2 right-0 bg-primary text-bg-main text-[10px] font-bold px-2 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm border border-border-bold hidden group-hover:block">
            {isExpanded ? "Collapse Sidebar" : "Expand Sidebar"}
          </div>
        </button>
      </div>

      <div
        className={cn(
          "p-4 border-b border-border-light bg-bg-main shrink-0 transition-opacity",
          !isExpanded && "px-2 py-4",
        )}
      >
        <div
          className={cn(
            "grid gap-3 transition-all",
            isExpanded ? "grid-cols-2" : "grid-cols-1",
          )}
        >
          {/* Audio Tool */}
          <button
            onClick={() => handleToolClick("audio")}
            disabled={!hasSources}
            className={cn(
              "relative flex p-3 bg-orange-500/10 border border-orange-500/30 rounded-sm transition-all text-xs font-bold text-primary group overflow-hidden items-center justify-center dark:bg-orange-500/5",
              isExpanded ? "flex-col gap-2" : "",
              hasSources
                ? "hover:border-orange-500 hover:shadow-md"
                : "opacity-40 grayscale cursor-not-allowed",
            )}
            style={{
              backgroundImage:
                "repeating-linear-gradient(45deg, rgba(249, 115, 22, 0.05) 0, rgba(249, 115, 22, 0.05) 1px, transparent 0, transparent 8px)",
            }}
          >
            {isExpanded && (
              <div className="absolute top-0 right-0 border-b border-l border-orange-500/30 bg-bg-main px-1 py-0.5 text-[8px] text-orange-400 font-mono tracking-tighter opacity-70 group-hover:opacity-100 transition-opacity">
                [AU]
              </div>
            )}
            <div className="w-8 h-8 flex items-center justify-center bg-bg-main border border-orange-500/20 rounded-full shadow-sm group-hover:scale-105 transition-transform shrink-0">
              <span className="material-symbols-outlined icon-sm text-orange-500">
                graphic_eq
              </span>
            </div>
            {isExpanded && (
              <span className="z-10 text-left pt-1 whitespace-nowrap overflow-hidden text-ellipsis">
                Audio
              </span>
            )}
            <div className="absolute bottom-0 left-0 w-full h-0.5 bg-orange-400 opacity-60 group-hover:opacity-100 transition-opacity"></div>
          </button>

          {/* Video Tool */}
          <button
            onClick={() => handleToolClick("video")}
            disabled={!hasSources}
            className={cn(
              "relative flex p-3 bg-cyan-500/10 border border-cyan-500/30 rounded-sm transition-all text-xs font-bold text-primary group overflow-hidden items-center justify-center dark:bg-cyan-500/5",
              isExpanded ? "flex-col gap-2" : "",
              hasSources
                ? "hover:border-cyan-500 hover:shadow-md"
                : "opacity-40 grayscale cursor-not-allowed",
            )}
            style={{
              backgroundImage:
                "repeating-linear-gradient(45deg, rgba(6, 182, 212, 0.05) 0, rgba(6, 182, 212, 0.05) 1px, transparent 0, transparent 8px)",
            }}
          >
            {isExpanded && (
              <div className="absolute top-0 right-0 border-b border-l border-cyan-500/30 bg-bg-main px-1 py-0.5 text-[8px] text-cyan-500 font-mono tracking-tighter opacity-70 group-hover:opacity-100 transition-opacity">
                [VI]
              </div>
            )}
            <div className="w-8 h-8 flex items-center justify-center bg-bg-main border border-cyan-500/20 rounded-full shadow-sm group-hover:scale-105 transition-transform shrink-0">
              <span className="material-symbols-outlined icon-sm text-cyan-500">
                smart_display
              </span>
            </div>
            {isExpanded && (
              <span className="z-10 text-left pt-1 whitespace-nowrap overflow-hidden text-ellipsis">
                Video
              </span>
            )}
            <div className="absolute bottom-0 left-0 w-full h-0.5 bg-cyan-500 opacity-60 group-hover:opacity-100 transition-opacity"></div>
          </button>

          {/* Brief Tool */}
          <button
            onClick={() => handleToolClick("brief")}
            disabled={!hasSources}
            className={cn(
              "relative flex p-3 bg-emerald-500/10 border border-emerald-500/30 rounded-sm transition-all text-xs font-bold text-primary group overflow-hidden items-center justify-center dark:bg-emerald-500/5",
              isExpanded ? "flex-col gap-2" : "",
              hasSources
                ? "hover:border-emerald-600 hover:shadow-md"
                : "opacity-40 grayscale cursor-not-allowed",
            )}
            style={{
              backgroundImage:
                "repeating-linear-gradient(45deg, rgba(16, 185, 129, 0.05) 0, rgba(16, 185, 129, 0.05) 1px, transparent 0, transparent 8px)",
            }}
          >
            {isExpanded && (
              <div className="absolute top-0 right-0 border-b border-l border-emerald-500/30 bg-bg-main px-1 py-0.5 text-[8px] text-emerald-600 font-mono tracking-tighter opacity-70 group-hover:opacity-100 transition-opacity">
                [BR]
              </div>
            )}
            <div className="w-8 h-8 flex items-center justify-center bg-bg-main border border-emerald-500/20 rounded-full shadow-sm group-hover:scale-105 transition-transform shrink-0">
              <span className="material-symbols-outlined icon-sm text-emerald-600">
                summarize
              </span>
            </div>
            {isExpanded && (
              <span className="z-10 text-left pt-1 whitespace-nowrap overflow-hidden text-ellipsis">
                Brief
              </span>
            )}
            <div className="absolute bottom-0 left-0 w-full h-0.5 bg-emerald-600 opacity-60 group-hover:opacity-100 transition-opacity"></div>
          </button>

          {/* Cards Tool */}
          <button
            onClick={() => handleToolClick("cards")}
            disabled={!hasSources}
            className={cn(
              "relative flex p-3 bg-violet-500/10 border border-violet-500/30 rounded-sm transition-all text-xs font-bold text-primary group overflow-hidden items-center justify-center dark:bg-violet-500/5",
              isExpanded ? "flex-col gap-2" : "",
              hasSources
                ? "hover:border-violet-600 hover:shadow-md"
                : "opacity-40 grayscale cursor-not-allowed",
            )}
            style={{
              backgroundImage:
                "repeating-linear-gradient(45deg, rgba(139, 92, 246, 0.05) 0, rgba(139, 92, 246, 0.05) 1px, transparent 0, transparent 8px)",
            }}
          >
            {isExpanded && (
              <div className="absolute top-0 right-0 border-b border-l border-violet-500/30 bg-bg-main px-1 py-0.5 text-[8px] text-violet-600 font-mono tracking-tighter opacity-70 group-hover:opacity-100 transition-opacity">
                [CA]
              </div>
            )}
            <div className="w-8 h-8 flex items-center justify-center bg-bg-main border border-violet-500/20 rounded-full shadow-sm group-hover:scale-105 transition-transform shrink-0">
              <span className="material-symbols-outlined icon-sm text-violet-600">
                style
              </span>
            </div>
            {isExpanded && (
              <span className="z-10 text-left pt-1 whitespace-nowrap overflow-hidden text-ellipsis">
                Cards
              </span>
            )}
            <div className="absolute bottom-0 left-0 w-full h-0.5 bg-violet-600 opacity-60 group-hover:opacity-100 transition-opacity"></div>
          </button>
        </div>
      </div>

      <div className="flex-1 px-5 py-5 flex flex-col items-center justify-start overflow-y-auto">
        {isExpanded ? (
          <>
            {activities.length > 0 ? (
              <div className="w-full mb-6 pb-20">
                <div className="flex flex-col gap-2 w-full mt-2">
                  {activities.map((activity) => (
                    <div
                      key={activity.id}
                      className={cn(
                        "w-full text-left p-2 rounded-sm flex items-center justify-between group transition-colors border relative cursor-pointer",
                        activeActivity?.id === activity.id
                          ? "bg-bg-main border-border-bold shadow-hard-sm"
                          : "border-transparent hover:border-border-light hover:bg-bg-sources",
                      )}
                      onClick={() => {
                        // Prevent click if editing
                        if (editingId === activity.id) return;
                        setActiveActivity({
                          id: activity.id,
                          type: activity.type,
                        });
                      }}
                    >
                      <div className="flex items-center gap-3 overflow-hidden flex-1 m-1">
                        <span className="material-symbols-outlined icon-sm text-gray-500 shrink-0">
                          {activity.type === "note"
                            ? "article"
                            : activity.type === "audio"
                              ? "graphic_eq"
                              : "smart_display"}
                        </span>
                        <div className="flex flex-col overflow-hidden flex-1">
                          {editingId === activity.id ? (
                            <input
                              ref={editInputRef}
                              value={editTitle}
                              onChange={(e) => setEditTitle(e.target.value)}
                              onBlur={() => saveRename(activity.id)}
                              onKeyDown={(e) => handleKeyDown(e, activity.id)}
                              className="text-xs font-bold text-primary bg-bg-main border border-border-light px-1 py-0.5 outline-none rounded"
                              onClick={(e) => e.stopPropagation()}
                            />
                          ) : (
                            <span className="text-xs font-bold text-primary truncate">
                              {activity.title}
                            </span>
                          )}
                          <span className="text-[10px] text-gray-500 font-medium">
                            Just now
                          </span>
                        </div>
                      </div>

                      <div
                        className="shrink-0 ml-2"
                        onClick={(e) => e.stopPropagation()}
                      >
                        <DropdownMenu.Root>
                          <DropdownMenu.Trigger asChild>
                            <button className="h-8 w-8 flex items-center justify-center rounded hover:bg-bg-sources text-gray-400 opacity-0 group-hover:opacity-100 transition-opacity">
                              <span className="material-symbols-outlined icon-sm">
                                more_vert
                              </span>
                            </button>
                          </DropdownMenu.Trigger>
                          <DropdownMenu.Portal>
                            <DropdownMenu.Content
                              align="end"
                              className="min-w-[120px] bg-bg-main rounded-md p-1 shadow-lg border border-border-bold z-50 text-xs font-medium"
                            >
                              <DropdownMenu.Item
                                className="flex items-center px-2 py-1.5 outline-none cursor-pointer hover:bg-bg-sources rounded text-primary"
                                onClick={() => {
                                  setEditTitle(activity.title);
                                  setEditingId(activity.id);
                                }}
                              >
                                <span className="material-symbols-outlined icon-sm mr-2 text-gray-500">
                                  edit
                                </span>
                                Rename
                              </DropdownMenu.Item>
                              <DropdownMenu.Item
                                className="flex items-center px-2 py-1.5 outline-none cursor-pointer hover:bg-red-50 hover:text-red-700 rounded text-red-600"
                                onClick={() => {
                                  setActivities((prev) =>
                                    prev.filter((a) => a.id !== activity.id),
                                  );
                                  if (activeActivity?.id === activity.id) {
                                    setActiveActivity(null);
                                  }
                                }}
                              >
                                <span className="material-symbols-outlined icon-sm mr-2">
                                  delete
                                </span>
                                Delete
                              </DropdownMenu.Item>
                            </DropdownMenu.Content>
                          </DropdownMenu.Portal>
                        </DropdownMenu.Root>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ) : (
              <div
                className="w-full h-48 border-2 border-dashed border-border-light rounded-sm relative flex flex-col items-center justify-center p-6 text-center"
                style={{
                  backgroundImage:
                    "repeating-linear-gradient(45deg, var(--border-light) 0, var(--border-light) 1px, transparent 0, transparent 10px)",
                }}
              >
                <div className="absolute top-2 right-2 text-[9px] font-bold text-gray-400 bg-bg-main px-1 border border-border-bold">
                  [ST-EMPTY]
                </div>
                <div className="w-12 h-12 bg-bg-main rounded-full flex items-center justify-center border border-border-light mb-3 shadow-sm">
                  <span className="material-symbols-outlined text-gray-300 icon-lg">
                    auto_awesome
                  </span>
                </div>
                <h3 className="text-sm font-bold text-gray-600 mb-1">
                  No generated content yet
                </h3>
                <p className="text-xs text-gray-400 font-medium leading-relaxed">
                  Select a tool above to start creating
                </p>
              </div>
            )}
          </>
        ) : (
          <div className="flex flex-col items-center gap-4 mt-4 w-full">
            <button
              title="History"
              className="text-gray-400 hover:text-primary hover:bg-bg-sources p-2 rounded-sm transition-colors border border-transparent hover:border-border-light"
            >
              <span className="material-symbols-outlined">history</span>
            </button>
          </div>
        )}
      </div>

      <div
        className={cn(
          "absolute bottom-6 left-0 right-0 flex justify-center pointer-events-none z-10",
        )}
      >
        {isExpanded ? (
          <button
            onClick={handleAddNote}
            className="pointer-events-auto bg-primary text-bg-main border border-border-bold px-6 py-3 shadow-hard hover:shadow-hard-hover hover:-translate-y-0.5 transition-all flex items-center gap-2 font-black text-sm uppercase tracking-wide"
          >
            <span className="material-symbols-outlined icon-sm">note_add</span>
            New Note
          </button>
        ) : (
          <button
            onClick={handleAddNote}
            className="pointer-events-auto bg-primary text-bg-main border border-border-bold w-10 h-10 shadow-hard-sm flex items-center justify-center hover:shadow-hard hover:-translate-y-0.5 transition-all"
          >
            <span className="material-symbols-outlined icon-sm">note_add</span>
          </button>
        )}
      </div>

      {isExpanded && activeTool === "audio" && (
        <PanelDetailView
          title="Audio Studio"
          icon="graphic_eq"
          onBack={() => setActiveTool(null)}
        >
          <div className="p-5 flex flex-col gap-4">
            <div className="p-4 bg-orange-500/10 border border-orange-500/30 rounded-sm">
              <h3 className="text-sm font-bold text-orange-600 mb-2">
                Voice Generation
              </h3>
              <p className="text-xs text-orange-600/80 mb-4">
                Generate podcast-style discussions or audio summaries from your
                sources.
              </p>
              <button className="w-full py-2 bg-orange-500 hover:bg-orange-600 text-white font-bold text-xs rounded-sm transition-colors shadow-sm">
                Generate Audio
              </button>
            </div>

            <div className="flex flex-col gap-2">
              <h4 className="text-xs font-bold text-gray-500 uppercase tracking-widest px-1">
                Options
              </h4>
              <div className="p-3 bg-bg-main border border-border-light shadow-sm flex items-center justify-between">
                <span className="text-xs font-medium text-primary">
                  Voice Clone
                </span>
                <span className="text-[10px] bg-bg-sources text-gray-500 px-2 py-0.5 font-bold uppercase border border-border-light transition-colors">
                  Pro
                </span>
              </div>
              <div className="p-3 bg-bg-main border border-border-light shadow-sm flex items-center justify-between">
                <span className="text-xs font-medium text-primary">
                  Background Music
                </span>
                <span className="material-symbols-outlined icon-sm text-gray-400">
                  toggle_off
                </span>
              </div>
            </div>
          </div>
        </PanelDetailView>
      )}

      {isExpanded && activeTool === "video" && (
        <PanelDetailView
          title="Video Studio"
          icon="smart_display"
          onBack={() => setActiveTool(null)}
        >
          <div className="p-5 flex flex-col gap-4">
            <div className="p-4 bg-cyan-500/10 border border-cyan-500/30 rounded-sm">
              <h3 className="text-sm font-bold text-cyan-500 mb-2">
                Video Generation
              </h3>
              <p className="text-xs text-cyan-600/80 mb-4">
                Create short explainer videos generated directly from your
                notes.
              </p>
              <button className="w-full py-2 bg-cyan-500 hover:bg-cyan-600 text-white font-bold text-xs rounded-sm transition-colors shadow-sm">
                Generate Video
              </button>
            </div>
          </div>
        </PanelDetailView>
      )}

      {isExpanded && activeTool === "brief" && (
        <PanelDetailView
          title="Brief Generator"
          icon="summarize"
          onBack={() => setActiveTool(null)}
        >
          <div className="p-5 flex flex-col gap-4">
            <div className="p-4 bg-emerald-500/10 border border-emerald-500/30 rounded-sm">
              <h3 className="text-sm font-bold text-emerald-500 mb-2">
                Executive Brief
              </h3>
              <p className="text-xs text-emerald-600/80 mb-4">
                Synthesize all your sources into a quick, readable summary
                document.
              </p>
              <button className="w-full py-2 bg-emerald-600 hover:bg-emerald-700 text-white font-bold text-xs rounded-sm transition-colors shadow-sm">
                Create Brief
              </button>
            </div>
          </div>
        </PanelDetailView>
      )}

      {isExpanded && activeTool === "cards" && (
        <PanelDetailView
          title="Flashcards"
          icon="style"
          onBack={() => setActiveTool(null)}
        >
          <div className="p-5 flex flex-col gap-4">
            <div className="p-4 bg-violet-500/10 border border-violet-500/30 rounded-sm">
              <h3 className="text-sm font-bold text-violet-500 mb-2">
                Study Cards
              </h3>
              <p className="text-xs text-violet-600/80 mb-4">
                Automatically generate flashcards and FAQs for active recall.
              </p>
              <button className="w-full py-2 bg-violet-600 hover:bg-violet-700 text-white font-bold text-xs rounded-sm transition-colors shadow-sm">
                Generate Cards
              </button>
            </div>
          </div>
        </PanelDetailView>
      )}
    </aside>
  );
}
