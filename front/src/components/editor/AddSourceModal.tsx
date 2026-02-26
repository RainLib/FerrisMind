import * as React from "react";
import { useState, useRef, useEffect } from "react";

interface AddSourceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onUploadFiles?: (files: File[]) => void;
  onUploadUrls?: (urls: string[]) => void;
}

export function AddSourceModal({
  isOpen,
  onClose,
  onUploadFiles,
  onUploadUrls,
}: AddSourceModalProps) {
  const [inputValue, setInputValue] = useState("");
  const [urlInputValue, setUrlInputValue] = useState("");
  const [activeTab, setActiveTab] = useState<"main" | "website" | "text">(
    "main",
  );
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Auto-resize textarea based on content
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 160)}px`;
    }
  }, [inputValue]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-bg-main/60 backdrop-blur-[2px]">
      <div className="bg-bg-main w-[600px] border border-border-bold shadow-modal relative">
        <button
          onClick={onClose}
          className="absolute top-4 right-4 p-1 hover:bg-bg-sources rounded-sm transition-colors z-10 border border-transparent hover:border-border-light"
        >
          <span className="material-symbols-outlined text-gray-400 hover:text-primary">
            close
          </span>
        </button>
        <div className="absolute top-0 left-0 px-2 py-1 bg-primary text-bg-main text-[10px] font-mono font-bold border-br border-border-bold">
          M-01: UPLOAD
        </div>
        <div className="p-8 pt-10">
          {activeTab === "main" ? (
            <>
              <h2 className="text-xl font-bold mb-1 text-center text-primary">
                Create Overview from your documents
              </h2>
              <p className="text-center text-sm text-gray-500 mb-8 font-medium">
                Add sources to generate insights
              </p>
              <div className="relative group mb-8 flex flex-col bg-bg-main border border-border-light focus-within:border-border-bold focus-within:shadow-hard-sm transition-all">
                <div className="relative flex-1">
                  <span className="material-symbols-outlined absolute left-4 top-3.5 text-gray-400 z-10 icon-sm">
                    search
                  </span>
                  <textarea
                    ref={textareaRef}
                    value={inputValue}
                    onChange={(e) => setInputValue(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        if (inputValue.trim()) {
                          console.log(
                            "Adding source via URL/Search:",
                            inputValue.trim(),
                          );
                          setInputValue("");
                          if (textareaRef.current) {
                            textareaRef.current.style.height = "auto";
                          }
                        }
                      }
                    }}
                    className="w-full bg-transparent border-none py-3 pl-12 pr-4 text-sm font-medium placeholder-gray-400 text-primary focus:ring-0 resize-none outline-none"
                    style={{
                      minHeight: "64px",
                      maxHeight: "160px",
                      scrollbarWidth: "none",
                      msOverflowStyle: "none",
                    }}
                    placeholder="Paste URLs, text or search the web for new sources"
                    rows={1}
                  />
                </div>
                <div className="flex justify-end p-2 bg-bg-sources/50 border-t border-border-light">
                  <button className="px-3 py-1.5 bg-bg-sources hover:bg-primary hover:text-bg-main border border-border-light text-[10px] font-bold uppercase tracking-wider text-gray-600 rounded-sm transition-colors flex items-center gap-1">
                    <span className="material-symbols-outlined text-[14px]">
                      language
                    </span>{" "}
                    Web
                  </button>
                </div>
              </div>
              <div className="relative h-64 border-2 border-dashed border-border-light bg-bg-sources/30 hover:bg-bg-main hover:border-border-bold transition-all group flex flex-col items-center justify-center gap-4 mb-4 overflow-hidden">
                <div className="absolute inset-0 bg-background-image-diagonal-hatch opacity-10 pointer-events-none"></div>
                <div className="relative z-10 flex flex-col items-center">
                  <p className="text-base font-medium text-primary">
                    or drop your files
                  </p>
                  <p className="text-xs text-gray-500 mt-1">
                    pdf, images, docs, audio,{" "}
                    <span className="underline cursor-pointer hover:text-primary transition-colors">
                      and more
                    </span>
                  </p>
                </div>
                <div className="relative z-10 flex gap-3 mt-4">
                  <input
                    type="file"
                    multiple
                    accept=".pdf,.doc,.docx,.wps,.txt,application/pdf,application/msword,application/vnd.openxmlformats-officedocument.wordprocessingml.document,text/plain"
                    className="hidden"
                    ref={fileInputRef}
                    onChange={(e) => {
                      if (e.target.files && e.target.files.length > 0) {
                        try {
                          if (onUploadFiles) {
                            onUploadFiles(Array.from(e.target.files));
                          }
                          onClose();
                        } catch (error) {
                          console.error("Failed to select files:", error);
                        } finally {
                          if (fileInputRef.current) {
                            fileInputRef.current.value = "";
                          }
                        }
                      }
                    }}
                  />
                  <button
                    onClick={() => fileInputRef.current?.click()}
                    className="flex items-center gap-2 px-4 py-2 bg-primary text-bg-main border border-border-bold shadow-hard-sm hover:shadow-hard hover:-translate-y-0.5 transition-all text-xs font-bold"
                  >
                    <span className="material-symbols-outlined icon-sm">
                      upload_file
                    </span>
                    Upload files
                  </button>
                  <button
                    onClick={() => setActiveTab("website")}
                    className="flex items-center gap-2 px-4 py-2 bg-bg-main text-primary border border-border-bold hover:bg-bg-sources shadow-sm hover:shadow-hard-sm hover:-translate-y-0.5 transition-all text-xs font-bold"
                  >
                    <span className="material-symbols-outlined icon-sm">
                      link
                    </span>
                    Websites
                  </button>
                  <button className="flex items-center gap-2 px-4 py-2 bg-bg-main text-primary border border-border-bold hover:bg-bg-sources shadow-sm hover:shadow-hard-sm hover:-translate-y-0.5 transition-all text-xs font-bold">
                    <span className="material-symbols-outlined icon-sm">
                      add_to_drive
                    </span>
                    Drive
                  </button>
                  <button
                    onClick={() => setActiveTab("text")}
                    className="flex items-center gap-2 px-4 py-2 bg-bg-main text-primary border border-border-bold hover:bg-bg-sources shadow-sm hover:shadow-hard-sm hover:-translate-y-0.5 transition-all text-xs font-bold"
                  >
                    <span className="material-symbols-outlined icon-sm">
                      content_paste
                    </span>
                    Copied text
                  </button>
                </div>
              </div>
            </>
          ) : activeTab === "website" ? (
            <div className="flex flex-col h-full animate-in fade-in slide-in-from-right-4 duration-300">
              <div className="flex items-center gap-3 mb-6">
                <button
                  onClick={() => setActiveTab("main")}
                  className="p-1 hover:bg-bg-sources rounded-sm transition-colors border border-transparent hover:border-border-light"
                >
                  <span className="material-symbols-outlined icon-sm text-gray-500 hover:text-primary">
                    arrow_back
                  </span>
                </button>
                <h2 className="text-xl font-bold text-primary">
                  Website and YouTube URLs
                </h2>
              </div>
              <p className="text-sm text-gray-600 mb-4">
                Paste in Website and YouTube URLs below to upload as a source in
                NotebookLM.
              </p>
              <div className="relative border border-accent-main rounded-xl overflow-hidden mb-6 flex-1 min-h-[250px] focus-within:ring-2 focus-within:ring-accent-light transition-all bg-bg-main shadow-hard-sm">
                <textarea
                  className="w-full h-full min-h-[250px] p-4 text-sm resize-none outline-none placeholder-gray-500 text-primary bg-transparent"
                  placeholder="Paste any links"
                  value={urlInputValue}
                  onChange={(e) => setUrlInputValue(e.target.value)}
                  autoFocus
                />
              </div>
              <ul className="text-xs text-gray-500 space-y-2 list-disc pl-5 mb-8">
                <li>
                  To add multiple URLs, separate with a space or new line.
                </li>
                <li>
                  Only the visible text on the website will be imported at this
                  time.
                </li>
                <li>Paid articles are not supported.</li>
                <li>
                  Only the text transcript in YouTube will be imported at this
                  time.
                </li>
                <li>Only public YouTube videos are supported.</li>
                <li>
                  Recently uploaded videos may not be available to import.
                </li>
                <li>
                  If upload fails,{" "}
                  <a href="#" className="text-blue-600 hover:underline">
                    learn more
                  </a>{" "}
                  for common reasons.
                </li>
              </ul>
              <div className="flex justify-end gap-3 mt-auto">
                <button
                  className="px-6 py-2 bg-primary text-bg-main hover:opacity-90 font-bold rounded-full text-sm shrink-0 disabled:opacity-50 disabled:bg-bg-sources disabled:text-gray-400 border border-border-bold transition-all"
                  disabled={!urlInputValue.trim()}
                  onClick={() => {
                    const links = urlInputValue
                      .split(/[\s\n]+/)
                      .filter((u) => u.startsWith("http"));
                    if (links.length > 0 && onUploadUrls) {
                      onUploadUrls(links);
                      setUrlInputValue("");
                      onClose();
                    }
                  }}
                >
                  Insert
                </button>
              </div>
            </div>
          ) : (
            <div className="flex flex-col h-full animate-in fade-in slide-in-from-right-4 duration-300">
              <div className="flex items-center gap-3 mb-6">
                <button
                  onClick={() => setActiveTab("main")}
                  className="p-1 hover:bg-bg-sources rounded-sm transition-colors border border-transparent hover:border-border-light"
                >
                  <span className="material-symbols-outlined icon-sm text-gray-500 hover:text-primary">
                    arrow_back
                  </span>
                </button>
                <h2 className="text-xl font-bold text-primary">Copied text</h2>
              </div>
              <p className="text-sm text-gray-600 mb-4">
                Paste any copied text below to upload as a source.
              </p>
              <div className="relative border border-border-light focus-within:border-border-bold rounded-xl overflow-hidden mb-6 flex-1 min-h-[250px] transition-all bg-bg-main shadow-hard-sm">
                <textarea
                  className="w-full h-full min-h-[250px] p-4 text-sm resize-none outline-none placeholder-gray-500 text-primary bg-transparent"
                  placeholder="Paste text here..."
                  autoFocus
                />
              </div>
              <div className="flex justify-end gap-3 mt-auto">
                <button
                  className="px-6 py-2 bg-gray-200 text-gray-400 font-bold rounded-full text-sm shrink-0"
                  disabled
                >
                  Insert
                </button>
              </div>
            </div>
          )}

          {activeTab === "main" && (
            <div className="flex items-center gap-3 mt-6">
              <div className="flex-1 h-1.5 bg-bg-sources rounded-full overflow-hidden border border-border-light">
                <div className="h-full bg-accent-main w-[65%] rounded-full"></div>
              </div>
              <span className="text-[10px] font-mono font-bold text-gray-500">
                11 / 300
              </span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
