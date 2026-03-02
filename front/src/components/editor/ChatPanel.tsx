import { Button } from "@/components/ui/button";
import { MarkdownRenderer } from "@/components/ui/markdown-renderer";
import { useState, useRef, useEffect } from "react";
import { useNotebookStore } from "@/store/notebookStore";

interface ChatPanelProps {
  notebookId: string;
  isMobile?: boolean;
  onOpenLeft?: () => void;
}

export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  stages?: { stage: string; message: string; progress: number }[];
  metadata?: { intent?: string; [key: string]: unknown };
  isStreaming?: boolean;
  suggestedQuestions?: string[];
}

export function ChatPanel({ notebookId }: ChatPanelProps) {
  const { sources, selectedIds, initialSessionId, initialMessages } =
    useNotebookStore();
  const hasSources = sources.length > 0;

  const [inputValue, setInputValue] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const [sessionId, setSessionId] = useState<string | null>(initialSessionId);
  const [messages, setMessages] = useState<ChatMessage[]>(initialMessages);
  const [isSending, setIsSending] = useState(false);

  useEffect(() => {
    setSessionId(initialSessionId);
    setMessages(initialMessages);
  }, [initialSessionId, initialMessages]);

  // Auto-resize textarea based on content
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 200)}px`;
    }
  }, [inputValue]);

  const handleCopy = (content: string) => {
    navigator.clipboard.writeText(content);
    // Optional: show a small toast here if a toast context is available
  };

  const handleAction = (actionName: string, msgId: string) => {
    console.log(`Action [${actionName}] clicked for message ${msgId}`);
  };

  const handleSuggestionClick = (question: string) => {
    handleSend(question);
  };

  const handleSend = async (overrideText?: string) => {
    const textToSend = overrideText !== undefined ? overrideText : inputValue;
    if (!textToSend.trim() || isSending) return;

    const userMsg: ChatMessage = {
      id: "usr_" + Date.now(),
      role: "user",
      content: textToSend.trim(),
    };

    const aiMsgId = "ai_" + Date.now();
    const aiMsg: ChatMessage = {
      id: aiMsgId,
      role: "assistant",
      content: "",
      stages: [],
      isStreaming: true,
    };

    setMessages((prev) => [...prev, userMsg, aiMsg]);
    setInputValue("");
    setIsSending(true);

    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }

    try {
      const response = await fetch(
        `${process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:8080"}/api/chat/stream`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            notebook_id: notebookId,
            content: userMsg.content,
            session_id: sessionId,
            source_ids: Array.from(selectedIds),
          }),
        },
      );

      if (!response.ok) {
        throw new Error("Failed to send message");
      }

      if (!response.body) {
        throw new Error("No response body");
      }

      const reader = response.body.getReader();
      const decoder = new TextDecoder();
      let done = false;
      let buffer = "";
      let currentEvent = "message";
      let dataLines: string[] = [];

      const processEvent = (eventType: string, dataStr: string) => {
        if (dataStr === "[DONE]") {
          setMessages((prev) =>
            prev.map((msg) =>
              msg.id === aiMsgId ? { ...msg, isStreaming: false } : msg,
            ),
          );
          return;
        }

        if (eventType === "session") {
          try {
            const data = JSON.parse(dataStr);
            if (data.session_id) setSessionId(data.session_id);
          } catch {
            /* ignore */
          }
        } else if (eventType === "stage") {
          try {
            const data = JSON.parse(dataStr);
            setMessages((prev) =>
              prev.map((msg) => {
                if (msg.id === aiMsgId) {
                  const stages = [...(msg.stages || []), data];
                  return { ...msg, stages };
                }
                return msg;
              }),
            );
          } catch {
            /* ignore */
          }
        } else if (eventType === "metadata") {
          try {
            const data = JSON.parse(dataStr);
            setMessages((prev) =>
              prev.map((msg) =>
                msg.id === aiMsgId ? { ...msg, metadata: data } : msg,
              ),
            );
          } catch {
            /* ignore */
          }
        } else if (eventType === "answer") {
          setMessages((prev) =>
            prev.map((msg) =>
              msg.id === aiMsgId
                ? { ...msg, content: msg.content + dataStr }
                : msg,
            ),
          );
        } else if (eventType === "suggestions") {
          try {
            const questions: string[] = JSON.parse(dataStr);
            setMessages((prev) =>
              prev.map((msg) =>
                msg.id === aiMsgId
                  ? { ...msg, suggestedQuestions: questions }
                  : msg,
              ),
            );
          } catch {
            /* ignore */
          }
        } else if (eventType === "error") {
          setMessages((prev) =>
            prev.map((msg) =>
              msg.id === aiMsgId
                ? {
                    ...msg,
                    content:
                      "Sorry, an error occurred while processing your request. Please try again.",
                    isStreaming: false,
                    metadata: { ...msg.metadata, error: dataStr },
                  }
                : msg,
            ),
          );
          done = true;
        } else if (eventType === "done") {
          setMessages((prev) =>
            prev.map((msg) =>
              msg.id === aiMsgId ? { ...msg, isStreaming: false } : msg,
            ),
          );
        }
      };

      while (!done) {
        const { value, done: readerDone } = await reader.read();
        done = readerDone;
        if (value) {
          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split(/\r?\n/);
          buffer = lines.pop() || "";

          for (const line of lines) {
            if (done) break;

            if (line.startsWith("event: ")) {
              currentEvent = line.substring(7).trim();
            } else if (line.startsWith("event:")) {
              currentEvent = line.substring(6).trim();
            } else if (line.startsWith("data: ")) {
              dataLines.push(line.substring(6));
            } else if (line.startsWith("data:")) {
              dataLines.push(line.substring(5));
            } else if (line === "") {
              if (dataLines.length > 0) {
                const dataStr = dataLines.join("\n");
                dataLines = [];
                processEvent(currentEvent, dataStr);
              }
              currentEvent = "message";
            }
          }
        }
      }
    } catch (e) {
      console.error("Chat error:", e);
      setMessages((prev) =>
        prev.map((msg) =>
          msg.id === aiMsgId
            ? {
                ...msg,
                content: msg.content + "\n\n*Error: Failed to get response.*",
                isStreaming: false,
              }
            : msg,
        ),
      );
    } finally {
      setIsSending(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const markdownContent = `
These documents mainly explore the integration and application of **Large Language Models (LLM)** and **Agentic** architectures in modern recommendation systems.

The content details advanced technologies ranging from **Transformer** infrastructure to Retrieval-Augmented Generation (**RAG**), specifically leveraging Knowledge Graphs and Multi-Agent frameworks (like [LangGraph](#)) to enhance system reasoning and planning capabilities.

Through a specific movie recommendation project case, the documents demonstrate how to build complex backend systems containing microservices, real-time data pipelines, and automated evaluation feedback loops.
  `.trim();

  return (
    <section className="flex-1 flex flex-col bg-bg-main relative w-full h-full overflow-hidden">
      <div className="h-14 px-4 border-b border-border-bold flex items-center justify-between z-10 sticky top-0 bg-bg-main/95 backdrop-blur-sm shrink-0">
        <div className="flex items-center gap-2 md:gap-3">
          <h2 className="text-xs font-bold text-gray-500 uppercase tracking-widest hidden sm:block">
            Chat
          </h2>
          {selectedIds.size > 0 && (
            <div className="px-2 py-0.5 bg-accent-light/20 text-accent-secondary text-[10px] font-bold border border-accent-main rounded-sm whitespace-nowrap">
              {selectedIds.size} source{selectedIds.size === 1 ? "" : "s"}{" "}
              active
            </div>
          )}
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

      <div className="flex-1 overflow-y-auto px-4 sm:px-8 lg:px-16 pt-8 pb-40 scrollbar-thin">
        {hasSources ? (
          <div className="max-w-5xl mx-auto space-y-12">
            {messages.length === 0 && (
              <>
                {/* Summary Section */}
                <div className="space-y-6">
                  <div className="flex flex-col sm:flex-row sm:items-start justify-between border-b border-border-light pb-6 gap-4">
                    <h1 className="text-3xl sm:text-4xl font-black text-primary tracking-tight uppercase leading-none">
                      Agentic AI Overview
                    </h1>
                    <div className="flex gap-2 self-start sm:self-auto">
                      <button
                        className="p-1.5 hover:bg-primary hover:text-bg-main transition-colors border border-transparent hover:border-border-bold text-gray-400"
                        title="Copy"
                      >
                        <span className="material-symbols-outlined icon-sm">
                          content_copy
                        </span>
                      </button>
                      <button
                        className="p-1.5 hover:bg-primary hover:text-bg-main transition-colors border border-transparent hover:border-border-bold text-gray-400"
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
                    <button className="px-4 py-2 bg-bg-main border border-border-bold shadow-hard-sm hover:shadow-hard hover:-translate-y-0.5 transition-all text-xs font-bold uppercase tracking-wider text-primary">
                      Suggest related topics
                    </button>
                    <button className="px-4 py-2 bg-bg-main border border-border-bold shadow-hard-sm hover:shadow-hard hover:-translate-y-0.5 transition-all text-xs font-bold uppercase tracking-wider text-primary">
                      Draft summary
                    </button>
                  </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <button
                    onClick={() =>
                      handleSend("How to use AWS to build Agentic AI?")
                    }
                    className="text-left p-5 bg-bg-main border border-border-bold shadow-hard-sm hover:shadow-hard hover:-translate-y-1 transition-all group hover:bg-accent-light/10"
                  >
                    <span className="block font-bold text-accent-secondary mb-2 text-[10px] uppercase tracking-widest">
                      Beginner&apos;s Guide
                    </span>
                    <span className="block text-sm font-bold text-primary">
                      How to use AWS to build Agentic AI?
                    </span>
                  </button>
                  <button
                    onClick={() =>
                      handleSend("LangGraph vs LlamaIndex pros & cons.")
                    }
                    className="text-left p-5 bg-bg-main border border-border-bold shadow-hard-sm hover:shadow-hard hover:-translate-y-1 transition-all group hover:bg-accent-light/10"
                  >
                    <span className="block font-bold text-accent-secondary mb-2 text-[10px] uppercase tracking-widest">
                      Comparison
                    </span>
                    <span className="block text-sm font-bold text-primary">
                      LangGraph vs LlamaIndex pros &amp; cons.
                    </span>
                  </button>
                  <button
                    onClick={() =>
                      handleSend(
                        "Ensuring AI agent safety & performance in production?",
                      )
                    }
                    className="text-left p-5 bg-bg-main border border-border-bold shadow-hard-sm hover:shadow-hard hover:-translate-y-1 transition-all md:col-span-2 group hover:bg-accent-light/10"
                  >
                    <span className="block font-bold text-accent-secondary mb-2 text-[10px] uppercase tracking-widest">
                      Best Practices
                    </span>
                    <span className="block text-sm font-bold text-primary">
                      Ensuring AI agent safety &amp; performance in production?
                    </span>
                  </button>
                </div>

                <div className="flex justify-center py-4">
                  <span className="text-[10px] font-bold text-gray-500 uppercase tracking-widest bg-bg-main px-3 py-1">
                    Today, 2:34 PM
                  </span>
                </div>
              </>
            )}

            <div className="flex flex-col gap-6">
              {messages.map((msg, index) => (
                <div key={msg.id} className="w-full">
                  {msg.role === "user" ? (
                    <div className="flex justify-end w-full">
                      <div className="bg-bg-sources text-primary px-4 sm:px-6 py-4 border border-border-bold shadow-hard-sm max-w-full">
                        <p className="text-sm font-medium whitespace-pre-wrap">
                          {msg.content}
                        </p>
                      </div>
                    </div>
                  ) : (
                    <div className="flex justify-start w-full">
                      <div className="w-full flex flex-col gap-2 relative mt-4">
                        <div
                          className={`w-full border p-4 sm:p-8 shadow-hard relative transition-all ${
                            msg.metadata?.error
                              ? "bg-red-500/10 border-red-500 shadow-[4px_4px_0px_0px_rgba(239,68,68,0.3)]"
                              : "bg-bg-paper border-border-bold"
                          }`}
                        >
                          <div
                            className={`absolute -top-3 -left-3 border px-3 py-1 text-xs font-bold uppercase text-bg-main shadow-sm flex items-center gap-2 ${
                              msg.metadata?.error
                                ? "bg-red-500 border-red-600"
                                : "bg-accent-main border-border-bold"
                            }`}
                          >
                            {msg.metadata?.error ? "Error" : "AI Response"}
                            {msg.metadata?.intent && !msg.metadata?.error && (
                              <span className="text-[9px] bg-bg-main/30 px-1.5 py-0.5 rounded-sm border border-bg-main/20">
                                {msg.metadata.intent}
                              </span>
                            )}
                          </div>
                          {msg.stages &&
                            msg.stages.length > 0 &&
                            msg.isStreaming && (
                              <div className="mb-4 text-xs font-mono text-gray-500 bg-bg-sources border border-dashed border-border-light p-3 rounded-sm overflow-hidden truncate">
                                <span className="material-symbols-outlined text-[14px] align-middle mr-1 animate-spin">
                                  progress_activity
                                </span>
                                {msg.stages[msg.stages.length - 1].stage}:{" "}
                                {msg.stages[msg.stages.length - 1].message}
                              </div>
                            )}
                          <MarkdownRenderer content={msg.content} />
                          {msg.isStreaming && (
                            <span className="inline-block w-2 h-4 bg-primary/50 animate-pulse ml-1 align-middle" />
                          )}
                        </div>

                        {/* Action Bar (Outside the Box) */}
                        {!msg.isStreaming && !msg.metadata?.error && (
                          <div className="flex items-center gap-1 text-gray-500 px-2 py-1 select-none">
                            <button
                              onClick={() => handleAction("save", msg.id)}
                              className="px-3 py-1.5 hover:bg-primary hover:text-bg-main transition-colors flex items-center gap-1.5 rounded-full text-xs font-bold uppercase tracking-widest border border-transparent hover:border-border-bold"
                              title="Save to Note"
                            >
                              <span className="material-symbols-outlined text-[16px]">
                                push_pin
                              </span>
                              Save to Note
                            </button>
                            <button
                              onClick={() => handleCopy(msg.content)}
                              className="p-1.5 hover:bg-primary hover:text-bg-main transition-colors rounded-full border border-transparent hover:border-border-bold"
                              title="Copy"
                            >
                              <span className="material-symbols-outlined text-[16px]">
                                content_copy
                              </span>
                            </button>
                            <button
                              onClick={() => handleAction("like", msg.id)}
                              className="p-1.5 hover:bg-primary hover:text-bg-main transition-colors rounded-full border border-transparent hover:border-border-bold ml-1"
                              title="Good Response"
                            >
                              <span className="material-symbols-outlined text-[16px]">
                                thumb_up
                              </span>
                            </button>
                            <button
                              onClick={() => handleAction("dislike", msg.id)}
                              className="p-1.5 hover:bg-primary hover:text-bg-main transition-colors rounded-full border border-transparent hover:border-border-bold"
                              title="Poor Response"
                            >
                              <span className="material-symbols-outlined text-[16px]">
                                thumb_down
                              </span>
                            </button>
                          </div>
                        )}

                        {/* Suggested follow-up questions */}
                        {!msg.isStreaming &&
                          msg.suggestedQuestions &&
                          msg.suggestedQuestions.length > 0 &&
                          index === messages.length - 1 && (
                            <div className="flex flex-wrap gap-2 mt-3 px-1">
                              {msg.suggestedQuestions.map((q, i) => (
                                <button
                                  key={i}
                                  onClick={() => handleSuggestionClick(q)}
                                  className="text-left px-3 py-2 bg-bg-main border border-border-light hover:border-border-bold hover:shadow-hard-sm hover:-translate-y-0.5 transition-all text-xs font-medium text-gray-500 hover:text-primary rounded-sm max-w-full"
                                >
                                  <span className="material-symbols-outlined text-[14px] align-middle mr-1.5 text-accent-secondary">
                                    arrow_forward
                                  </span>
                                  {q}
                                </button>
                              ))}
                            </div>
                          )}
                      </div>
                    </div>
                  )}
                </div>
              ))}
              <div ref={messagesEndRef} />
            </div>
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-center max-w-md mx-auto pt-[15vh]">
            <div className="w-16 h-16 bg-bg-sources border border-dashed border-border-light rounded-full flex items-center justify-center mb-6 shadow-sm relative">
              <span className="material-symbols-outlined text-gray-400 text-3xl">
                chat
              </span>
              <div className="absolute -top-1 -right-1 w-3 h-3 bg-red-400 rounded-full animate-pulse"></div>
            </div>
            <h3 className="text-xl font-black text-primary tracking-tight uppercase mb-3 text-shadow-sm">
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

      <div className="absolute bottom-0 left-0 right-0 p-4 sm:p-6 bg-linear-to-t from-bg-main via-bg-main/80 to-transparent pt-12">
        <div className="max-w-5xl mx-auto">
          <div className="bg-bg-main border border-border-bold shadow-hard hover:shadow-hard-hover transition-all flex flex-col relative z-20 group">
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
              className={`w-full bg-transparent border-none text-primary placeholder-gray-400 focus:ring-0 text-sm font-medium px-4 ${hasSources ? "py-3" : "py-2"} resize-none outline-none overflow-y-auto`}
              style={{
                minHeight: hasSources ? "56px" : "40px",
                maxHeight: "200px",
              }}
              placeholder="Ask a follow up question or share context..."
              rows={1}
            ></textarea>
            <div className="flex items-center justify-between px-3 pb-2">
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
                className="p-2 bg-accent-main border border-border-bold text-bg-main hover:bg-accent-secondary hover:shadow-hard-sm transition-all active:translate-y-0.5 rounded-sm disabled:opacity-50 disabled:hover:translate-y-0 disabled:hover:shadow-none"
                disabled={!inputValue.trim() || isSending}
                onClick={() => handleSend()}
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
