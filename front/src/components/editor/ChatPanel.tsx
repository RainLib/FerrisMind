import { Button } from "@/components/ui/button";
import { MarkdownRenderer } from "@/components/ui/markdown-renderer";
import { useState, useRef, useEffect } from "react";
import { useNotebookStore } from "@/store/notebookStore";

interface ChatPanelProps {
  isMobile?: boolean;
  onOpenLeft?: () => void;
}

export function ChatPanel({}: ChatPanelProps) {
  const { sources } = useNotebookStore();
  const hasSources = sources.length > 0;

  const [inputValue, setInputValue] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto-resize textarea based on content
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 200)}px`;
    }
  }, [inputValue]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      // Handle submit here
      if (inputValue.trim()) {
        console.log("Submit:", inputValue);
        setInputValue("");
      }
    }
  };

  const markdownContent = `
These documents mainly explore the integration and application of **Large Language Models (LLM)** and **Agentic** architectures in modern recommendation systems.

The content details advanced technologies ranging from **Transformer** infrastructure to Retrieval-Augmented Generation (**RAG**), specifically leveraging Knowledge Graphs and Multi-Agent frameworks (like [LangGraph](#)) to enhance system reasoning and planning capabilities.

Through a specific movie recommendation project case, the documents demonstrate how to build complex backend systems containing microservices, real-time data pipelines, and automated evaluation feedback loops.
  `.trim();

  const responseMarkdown = `
Introducing **Skills** and designing an **Agentic Framework** are core to building modern autonomous AI systems.

### 1. Skill Encapsulation Design
"Skills" are not just API calls; they should be designed as independent modules.

- **Skill-as-a-Folder**
  Standardize by encapsulating each Skill as a folder containing \`SKILL.md\`.
- **Stateless Atomic Design**
  Input → Execution → Output. Skills should not retain conversation history.
  `.trim();

  return (
    <section className="flex-1 flex flex-col bg-bg-main relative w-full h-full">
      <div className="h-14 px-4 border-b border-border-bold flex items-center justify-between z-10 sticky top-0 bg-white/95 backdrop-blur-sm shrink-0">
        <div className="flex items-center gap-2 md:gap-3">
          <h2 className="text-xs font-bold text-gray-500 uppercase tracking-widest hidden sm:block">
            Chat
          </h2>
          <div className="px-2 py-0.5 bg-accent-light text-accent-secondary text-[10px] font-bold border border-accent-main rounded-sm whitespace-nowrap">
            9 sources active
          </div>
        </div>
        <div className="flex gap-2">
          <Button variant="icon" title="Tune" className="hidden sm:inline-flex">
            <span className="material-symbols-outlined icon-sm">tune</span>
          </Button>
          <Button variant="icon" title="More" className="hidden sm:inline-flex">
            <span className="material-symbols-outlined icon-sm">
              more_horiz
            </span>
          </Button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto px-4 sm:px-8 lg:px-16 pt-8 pb-40">
        {hasSources ? (
          <div className="max-w-3xl mx-auto space-y-12">
            {/* Summary Section */}
            <div className="space-y-6">
              <div className="flex flex-col sm:flex-row sm:items-start justify-between border-b border-gray-200 pb-6 gap-4">
                <h1 className="text-3xl sm:text-4xl font-black text-black tracking-tight uppercase leading-none">
                  Agentic AI Overview
                </h1>
                <div className="flex gap-2 self-start sm:self-auto">
                  <button
                    className="p-1.5 hover:bg-black hover:text-white transition-colors border border-transparent hover:border-black text-gray-400"
                    title="Copy"
                  >
                    <span className="material-symbols-outlined icon-sm">
                      content_copy
                    </span>
                  </button>
                  <button
                    className="p-1.5 hover:bg-black hover:text-white transition-colors border border-transparent hover:border-black text-gray-400"
                    title="Save"
                  >
                    <span className="material-symbols-outlined icon-sm">
                      bookmark_border
                    </span>
                  </button>
                </div>
              </div>

              <MarkdownRenderer content={markdownContent} />

              <div className="flex flex-wrap gap-3 pt-4">
                <button className="px-4 py-2 bg-white border border-black shadow-hard-sm hover:shadow-hard hover:-translate-y-0.5 transition-all text-xs font-bold uppercase tracking-wider text-gray-700">
                  Suggest related topics
                </button>
                <button className="px-4 py-2 bg-white border border-black shadow-hard-sm hover:shadow-hard hover:-translate-y-0.5 transition-all text-xs font-bold uppercase tracking-wider text-gray-700">
                  Draft summary
                </button>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <button className="text-left p-5 bg-white border border-black shadow-hard-sm hover:shadow-hard hover:-translate-y-1 transition-all group hover:bg-accent-light/30">
                <span className="block font-bold text-accent-secondary mb-2 text-[10px] uppercase tracking-widest">
                  Beginner&apos;s Guide
                </span>
                <span className="block text-sm font-bold text-black">
                  How to use AWS to build Agentic AI?
                </span>
              </button>
              <button className="text-left p-5 bg-white border border-black shadow-hard-sm hover:shadow-hard hover:-translate-y-1 transition-all group hover:bg-accent-light/30">
                <span className="block font-bold text-accent-secondary mb-2 text-[10px] uppercase tracking-widest">
                  Comparison
                </span>
                <span className="block text-sm font-bold text-black">
                  LangGraph vs LlamaIndex pros &amp; cons.
                </span>
              </button>
              <button className="text-left p-5 bg-white border border-black shadow-hard-sm hover:shadow-hard hover:-translate-y-1 transition-all md:col-span-2 group hover:bg-accent-light/30">
                <span className="block font-bold text-accent-secondary mb-2 text-[10px] uppercase tracking-widest">
                  Best Practices
                </span>
                <span className="block text-sm font-bold text-black">
                  Ensuring AI agent safety &amp; performance in production?
                </span>
              </button>
            </div>

            <div className="flex justify-center py-4">
              <span className="text-[10px] font-bold text-gray-400 uppercase tracking-widest bg-white px-3 py-1">
                Today, 2:34 PM
              </span>
            </div>

            <div className="flex justify-end w-full">
              <div className="bg-gray-100 text-black px-4 sm:px-6 py-4 border border-black shadow-hard-sm max-w-[90%] sm:max-w-[80%]">
                <p className="text-sm font-medium">
                  I expect how to introduce skills and framework design.
                </p>
              </div>
            </div>

            <div className="flex justify-start w-full pb-8">
              <div className="w-full bg-white border border-black p-4 sm:p-8 shadow-hard relative mt-4">
                <div className="absolute -top-3 -left-3 bg-accent-main border border-black px-3 py-1 text-xs font-bold uppercase text-white shadow-sm">
                  AI Response
                </div>
                <MarkdownRenderer content={responseMarkdown} />
              </div>
            </div>
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-center max-w-md mx-auto pt-[15vh]">
            <div className="w-16 h-16 bg-gray-50 border border-dashed border-gray-300 rounded-full flex items-center justify-center mb-6 shadow-sm relative">
              <span className="material-symbols-outlined text-gray-400 text-3xl">
                chat
              </span>
              <div className="absolute -top-1 -right-1 w-3 h-3 bg-red-400 rounded-full animate-pulse"></div>
            </div>
            <h3 className="text-xl font-black text-black tracking-tight uppercase mb-3 text-shadow-sm">
              Your AI Assistant is Ready
            </h3>
            <p className="text-sm font-medium text-gray-500 leading-relaxed mb-8">
              There is currently no data in this notebook. Please add sources
              from the left sidebar to start generating insights and asking
              questions.
            </p>
          </div>
        )}
      </div>

      <div className="absolute bottom-0 left-0 right-0 p-4 sm:p-6 bg-linear-to-t from-white via-white/80 to-transparent pt-12">
        <div className="max-w-3xl mx-auto">
          <div className="bg-white border border-black shadow-hard hover:shadow-hard-hover transition-all flex flex-col relative z-20">
            <textarea
              ref={textareaRef}
              value={inputValue}
              onChange={(e) => {
                setInputValue(e.target.value);
                const el = e.target;
                el.style.height = "auto";
                el.style.height = `${Math.min(el.scrollHeight, 300)}px`;
              }}
              onKeyDown={handleKeyDown}
              className={`w-full bg-transparent border-none text-black placeholder-gray-400 focus:ring-0 text-sm font-medium px-4 ${hasSources ? "py-4" : "py-3"} resize-none outline-none overflow-y-auto`}
              style={{
                minHeight: hasSources ? "80px" : "44px",
                maxHeight: "200px",
              }}
              placeholder="Ask a follow up question or share context..."
              rows={3}
            ></textarea>
            <div className="flex items-center justify-between px-3 pb-3">
              <div className="flex gap-2">
                <Button variant="icon" title="Add context">
                  <span className="material-symbols-outlined icon-sm">
                    add_circle
                  </span>
                </Button>
                <Button variant="icon" title="Voice dictation">
                  <span className="material-symbols-outlined icon-sm">mic</span>
                </Button>
              </div>
              <button
                className="p-2 bg-accent-main border border-black text-white hover:bg-accent-secondary hover:shadow-hard-sm transition-all active:translate-y-0.5 rounded-sm disabled:opacity-50 disabled:hover:translate-y-0 disabled:hover:shadow-none"
                disabled={!inputValue.trim()}
                onClick={() => {
                  if (inputValue.trim()) {
                    console.log("Submit:", inputValue);
                    setInputValue("");
                    if (textareaRef.current) {
                      textareaRef.current.style.height = "auto";
                    }
                  }
                }}
              >
                <span className="material-symbols-outlined icon-sm">
                  arrow_upward
                </span>
              </button>
            </div>
          </div>
          <div className="text-center mt-3 hidden sm:block">
            <p className="text-[10px] font-bold text-gray-400 uppercase tracking-widest">
              AI may produce inaccurate information
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
