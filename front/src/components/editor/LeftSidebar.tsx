import * as React from "react";
import { useState, useRef, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { AddSourceModal } from "@/components/editor/AddSourceModal";
import { PanelDetailView } from "./PanelDetailView";
import {
  fetchGraphQL,
  GET_DOCUMENT_UPLOAD_STATUSES,
  GET_DOCUMENT_CONTENT,
  SUMMARIZE_DOCUMENT,
  DELETE_DOCUMENT,
  DocumentUploadStatus,
  DocumentContent,
} from "@/lib/graphql";

import { useNotebookStore, Source } from "@/store/notebookStore";

interface SourceItemProps {
  source: Source;
  selected: boolean;
  onSelectToggle: (id: string) => void;
  onItemClick: (id: string) => void;
  onDelete: (id: string) => void;
  onRename: (id: string, newTitle: string) => void;
  onRetry?: (id: string) => void;
}

function SourceItem({
  source,
  selected,
  onSelectToggle,
  onItemClick,
  onDelete,
  onRename,
  onRetry,
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

  const isUploading =
    source.rawStatus === "uploading" ||
    source.rawStatus === "pending" ||
    source.rawStatus === "processing";
  const isFailed = source.rawStatus === "failed";
  const isReady = !isUploading && !isFailed;

  return (
    <div
      className={`group flex items-center gap-3 p-3 bg-white shadow-sm transition-all ${
        isReady ? "cursor-pointer" : ""
      } ${
        selected
          ? "border-l-2 border-l-accent-main border-y border-r border-gray-200 hover:border-black"
          : isFailed
            ? "border border-red-200 bg-red-50/30"
            : isUploading
              ? "border border-gray-200 opacity-70"
              : "border border-gray-200 hover:border-black"
      }`}
      onClick={() => {
        if (isReady) onItemClick(source.id);
      }}
    >
      {/* Checkbox */}
      {isReady && (
        <div
          className="shrink-0 flex items-center justify-center p-1 -m-1 cursor-pointer"
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
      )}

      <div
        className={`p-1.5 rounded-sm shrink-0 flex items-center justify-center ${
          selected
            ? "bg-accent-light text-accent-secondary"
            : isFailed
              ? "bg-red-100 text-red-500"
              : "bg-gray-50 text-gray-400 " +
                (isReady ? "group-hover:text-black transition-colors" : "")
        }`}
      >
        {isUploading ? (
          <span className="material-symbols-outlined icon-sm animate-spin">
            progress_activity
          </span>
        ) : isFailed ? (
          <span className="material-symbols-outlined icon-sm">error</span>
        ) : (
          <span className="material-symbols-outlined icon-sm">
            {source.icon}
          </span>
        )}
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
                : isFailed
                  ? "font-semibold text-red-700"
                  : "font-semibold text-gray-700 " +
                    (isReady ? "group-hover:text-black" : "")
            }`}
            onDoubleClick={(e) => {
              if (isReady) {
                e.stopPropagation();
                setIsEditing(true);
              }
            }}
            title={isReady ? "Double click to rename" : ""}
          >
            {source.title}
          </p>
        )}
        <p
          className={`text-xs truncate font-medium ${
            isFailed
              ? "text-red-500"
              : isUploading
                ? "text-blue-500"
                : "text-gray-500"
          }`}
        >
          {source.sub}
        </p>
      </div>

      {/* Actions */}
      <div
        className="hidden group-hover:flex items-center gap-1 shrink-0"
        onClick={(e) => e.stopPropagation()}
      >
        {isReady && (
          <button
            onClick={() => setIsEditing(true)}
            className="p-1 text-gray-400 hover:text-black transition-colors relative group/btn"
          >
            <span className="material-symbols-outlined text-[16px]">edit</span>
            <div className="absolute top-full mt-2 left-1/2 -translate-x-1/2 bg-black text-white text-[10px] font-bold px-2 py-1 whitespace-nowrap opacity-0 group-hover/btn:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm border border-black hidden group-hover/btn:block">
              Rename
            </div>
          </button>
        )}
        {isFailed && onRetry && (
          <button
            onClick={() => onRetry(source.id)}
            className="p-1 text-gray-400 hover:text-blue-600 transition-colors relative group/btn"
          >
            <span className="material-symbols-outlined text-[16px]">
              refresh
            </span>
            <div className="absolute top-full mt-2 left-1/2 -translate-x-1/2 bg-black text-white text-[10px] font-bold px-2 py-1 whitespace-nowrap opacity-0 group-hover/btn:opacity-100 transition-opacity pointer-events-none z-50 shadow-sm border border-black hidden group-hover/btn:block">
              Retry
            </div>
          </button>
        )}
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
  notebookId: string;
}

export function LeftSidebar({
  isMobile,
  onToggle,
  notebookId,
}: LeftSidebarProps) {
  const [isAddSourceModalOpen, setIsAddSourceModalOpen] = useState(false);
  const [activeDetailId, setActiveDetailId] = useState<string | null>(null);
  const [documentContent, setDocumentContent] =
    useState<DocumentContent | null>(null);
  const [isContentLoading, setIsContentLoading] = useState(false);
  const [isSummarizing, setIsSummarizing] = useState(false);
  const { sources, setSources, selectedIds, setSelectedIds, addSelectedId } =
    useNotebookStore();

  // Removed redundant loadDocuments since it's fetched in GET_NOTEBOOK_INITIAL_DATA in page.tsx

  const handleUploadFiles = async (files: File[]) => {
    const tempSources = files.map((file) => ({
      id: "temp_" + Math.random().toString(36).substring(2, 9),
      icon: file.name.endsWith(".pdf") ? "picture_as_pdf" : "description",
      title: file.name,
      sub: "uploading",
      rawStatus: "uploading",
      file,
    }));

    setSources((prev) => [...tempSources, ...prev]);

    try {
      const formData = new FormData();
      formData.append("notebook_id", notebookId);
      files.forEach((file) => formData.append("files", file));

      const response = await fetch("http://localhost:8080/api/upload", {
        method: "POST",
        body: formData,
      });

      if (!response.ok) throw new Error("Upload failed");

      const result = await response.json();

      setSources((prev) =>
        prev.map((s) => {
          const foundTempIndex = tempSources.findIndex((ts) => ts.id === s.id);
          if (foundTempIndex >= 0 && result.documents[foundTempIndex]) {
            const doc = result.documents[foundTempIndex];
            if (doc.uploadStatus === "completed") {
              addSelectedId(doc.id);
            }
            return {
              ...s,
              id: doc.id,
              rawStatus: doc.uploadStatus,
              sub:
                doc.uploadStatus === "completed"
                  ? `${Math.round(doc.chunkCount * 1.5)} words`
                  : doc.uploadStatus,
            };
          }
          return s;
        }),
      );
    } catch {
      setSources((prev) =>
        prev.map((s) => {
          if (tempSources.find((ts) => ts.id === s.id)) {
            return { ...s, rawStatus: "failed", sub: "Upload Failed" };
          }
          return s;
        }),
      );
    }
  };

  const handleUploadUrls = async (urls: string[]) => {
    const tempSources = urls.map((url) => ({
      id: "temp_" + Math.random().toString(36).substring(2, 9),
      icon: "link",
      title: url,
      sub: "uploading",
      rawStatus: "uploading",
      url,
    }));

    setSources((prev) => [...tempSources, ...prev]);

    try {
      const response = await fetch("http://localhost:8080/api/upload/url", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          notebook_id: notebookId,
          urls,
        }),
      });

      if (!response.ok) throw new Error("Upload failed");

      const result = await response.json();

      setSources((prev) =>
        prev.map((s) => {
          const foundTempIndex = tempSources.findIndex((ts) => ts.id === s.id);
          if (foundTempIndex >= 0 && result.documents[foundTempIndex]) {
            const doc = result.documents[foundTempIndex];
            if (doc.uploadStatus === "completed") {
              addSelectedId(doc.id);
            }
            return {
              ...s,
              id: doc.id,
              rawStatus: doc.uploadStatus,
              sub:
                doc.uploadStatus === "completed"
                  ? `${Math.round(doc.chunkCount * 1.5)} words`
                  : doc.uploadStatus,
            };
          }
          return s;
        }),
      );
    } catch {
      setSources((prev) =>
        prev.map((s) => {
          if (tempSources.find((ts) => ts.id === s.id)) {
            return { ...s, rawStatus: "failed", sub: "Upload Failed" };
          }
          return s;
        }),
      );
    }
  };

  // Polling for upload statuses
  useEffect(() => {
    const pendingIds = sources
      .filter(
        (s) =>
          // Types need to match the new rawStatus field added to the mapped object
          s.rawStatus === "pending" ||
          s.rawStatus === "processing" ||
          s.rawStatus === "uploading",
      )
      .map((s) => s.id);

    if (pendingIds.length === 0) return;

    const intervalId = setInterval(async () => {
      const { data } = await fetchGraphQL<{
        documentUploadStatuses: DocumentUploadStatus[];
      }>(GET_DOCUMENT_UPLOAD_STATUSES, { ids: pendingIds });

      if (data?.documentUploadStatuses) {
        const newlyCompletedIds: string[] = [];

        setSources((prev) =>
          prev.map((source) => {
            const update = data.documentUploadStatuses.find(
              (s) => s.id === source.id,
            );
            if (update) {
              if (
                source.rawStatus !== "completed" &&
                update.uploadStatus === "completed"
              ) {
                newlyCompletedIds.push(source.id);
              }
              return {
                ...source,
                sub:
                  update.uploadStatus === "completed"
                    ? `${Math.round(update.chunkCount * 1.5)} words`
                    : update.uploadStatus,
                rawStatus: update.uploadStatus,
              };
            }
            return source;
          }),
        );

        if (newlyCompletedIds.length > 0) {
          setSelectedIds((prev) => {
            const next = new Set(prev);
            newlyCompletedIds.forEach((id) => next.add(id));
            return next;
          });
        }
      }
    }, 2000); // poll every 2 seconds

    return () => clearInterval(intervalId);
  }, [sources, setSources, setSelectedIds]);

  // Load document content when detail view is opened
  useEffect(() => {
    if (!activeDetailId) {
      setDocumentContent(null);
      return;
    }

    const loadContent = async () => {
      setIsContentLoading(true);
      try {
        const { data, errors } = await fetchGraphQL<{
          documentContent: DocumentContent;
        }>(GET_DOCUMENT_CONTENT, { documentId: activeDetailId });

        if (data?.documentContent) {
          setDocumentContent(data.documentContent);
        } else if (errors) {
          console.error("Failed to load document content:", errors);
        }
      } catch (e) {
        console.error("Failed to load document content:", e);
      } finally {
        setIsContentLoading(false);
      }
    };

    loadContent();
  }, [activeDetailId]);

  const handleSummarize = async () => {
    if (!activeDetailId) return;

    setIsSummarizing(true);
    try {
      const { data, errors } = await fetchGraphQL<{
        summarizeDocument: { documentId: string; summary: string };
      }>(SUMMARIZE_DOCUMENT, { documentId: activeDetailId });

      if (data?.summarizeDocument?.summary) {
        setDocumentContent((prev) =>
          prev ? { ...prev, summary: data.summarizeDocument.summary } : null,
        );
      } else if (errors) {
        console.error("Failed to summarize document:", errors);
      }
    } catch (e) {
      console.error(e);
    } finally {
      setIsSummarizing(false);
    }
  };

  // selectedIds is now from Zustand

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

  const handleDelete = async (id: string) => {
    // Optimistic UI update and polling stop
    setSources((prev) => prev.filter((s) => s.id !== id));
    setSelectedIds((prev) => {
      const next = new Set(prev);
      next.delete(id);
      return next;
    });

    if (!id.startsWith("temp_")) {
      try {
        await fetchGraphQL(DELETE_DOCUMENT, { id });
      } catch (e) {
        console.error("Failed to delete document on backend", e);
      }
    }
  };

  const handleRename = (id: string, newTitle: string) => {
    setSources((prev) =>
      prev.map((s) => (s.id === id ? { ...s, title: newTitle } : s)),
    );
  };

  const handleRetry = (id: string) => {
    const sourceToRetry = sources.find((s) => s.id === id);
    if (!sourceToRetry) return;

    // Remove the failed item before retrying
    setSources((prev) => prev.filter((s) => s.id !== id));

    if (sourceToRetry.file) {
      handleUploadFiles([sourceToRetry.file]);
    } else if (sourceToRetry.url) {
      handleUploadUrls([sourceToRetry.url]);
    }
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
        {sources.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 px-4 text-center relative border-2 border-dashed border-gray-200 bg-gray-50/50 rounded-sm">
            <div className="absolute top-2 right-2 text-[9px] font-bold text-gray-400 bg-white px-1 border border-gray-200">
              [EMPTY]
            </div>
            <div className="w-12 h-12 bg-white rounded-full flex items-center justify-center border border-gray-200 mb-4 shadow-sm">
              <span className="material-symbols-outlined text-gray-400 icon-lg">
                upload_file
              </span>
            </div>
            <h3 className="text-sm font-bold text-gray-600 mb-2">
              No sources found
            </h3>
            <p className="text-xs text-gray-500 font-medium leading-relaxed max-w-[200px] mx-auto">
              Please click the{" "}
              <strong className="text-gray-700">Add source</strong> button above
              to upload documents or web links.
            </p>
          </div>
        ) : (
          <>
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
                onRetry={handleRetry}
              />
            ))}
          </>
        )}
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
            {isContentLoading ? (
              <div className="flex items-center justify-center p-8">
                <span className="material-symbols-outlined animate-spin icon-md text-gray-400">
                  progress_activity
                </span>
              </div>
            ) : (
              <>
                {/* Summary Section */}
                <div>
                  <h3 className="text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-3">
                    Summary
                  </h3>
                  <div className="text-sm text-gray-800 leading-relaxed font-medium whitespace-pre-wrap">
                    {documentContent?.summary ? (
                      documentContent.summary
                    ) : activeSource.rawStatus === "completed" ? (
                      <div className="flex flex-col gap-3 items-start">
                        <span className="text-gray-400 italic">
                          No summary available for this document.
                        </span>
                        <Button
                          variant="secondary"
                          onClick={handleSummarize}
                          disabled={isSummarizing}
                        >
                          <span className="material-symbols-outlined icon-sm mr-2">
                            {isSummarizing
                              ? "progress_activity"
                              : "auto_awesome"}
                          </span>
                          {isSummarizing ? "Generating..." : "Generate Summary"}
                        </Button>
                      </div>
                    ) : (
                      <span className="text-gray-400 italic">
                        Document is still processing...
                      </span>
                    )}
                  </div>
                </div>

                {/* Key Topics Section (Placeholder until API supports it) */}
                <div>
                  <h3 className="text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-3">
                    Key Topics
                  </h3>
                  <div className="flex flex-wrap gap-2">
                    {documentContent?.summary ? (
                      // Very basic placeholder extraction of capitalized words from summary for now
                      Array.from(
                        new Set(
                          documentContent.summary
                            .split(/\s+/)
                            .filter(
                              (w) =>
                                w.length > 5 &&
                                w[0] === w[0].toUpperCase() &&
                                w[0] !== w[0].toLowerCase(),
                            )
                            .slice(0, 5),
                        ),
                      ).map((topic, i) => (
                        <span
                          key={i}
                          className="px-3 py-1.5 bg-gray-50 border border-gray-200 text-xs font-bold text-gray-700 hover:border-black hover:text-black cursor-pointer transition-colors shadow-sm"
                        >
                          {topic.replace(/[^a-zA-Z]/g, "")}
                        </span>
                      ))
                    ) : (
                      <span className="text-gray-400 text-xs italic">
                        Processing topics...
                      </span>
                    )}
                  </div>
                </div>
              </>
            )}

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
        onUploadFiles={handleUploadFiles}
        onUploadUrls={handleUploadUrls}
      />
    </aside>
  );
}
