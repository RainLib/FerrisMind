"use client";

import { EditorLayout } from "@/components/editor/EditorLayout";
import Link from "next/link";
import { Logo } from "@/components/ui/logo";
import { useState, useRef, useEffect } from "react";
import { useParams } from "next/navigation";
import {
  fetchGraphQL,
  GET_NOTEBOOK_INITIAL_DATA,
  NotebookInitialData,
} from "@/lib/graphql";
import { useNotebookStore, ChatMessage } from "@/store/notebookStore";

export default function Editor() {
  const params = useParams();
  const [title, setTitle] = useState("Untitled Notebook");
  const [isEditing, setIsEditing] = useState(false);
  const [tempTitle, setTempTitle] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const [isLoading, setIsLoading] = useState(true);

  const { setSources, setInitialChat, setSelectedIds } = useNotebookStore();

  // Decode the ID in case it comes URL-encoded from useParams (e.g. notebook%3A123 -> notebook:123)
  const idRaw = params?.id as string;
  const decodedId = decodeURIComponent(idRaw || "");

  useEffect(() => {
    const loadData = async () => {
      try {
        const { data, errors } = await fetchGraphQL<NotebookInitialData>(
          GET_NOTEBOOK_INITIAL_DATA,
          { notebookId: decodedId },
        );
        if (data?.notebook) {
          setTitle(data.notebook.name || "Untitled Notebook");
          setTempTitle(data.notebook.name || "Untitled Notebook");

          // Hydrate the store with documents
          const docs = data.documents || [];
          const newSources = docs.map((doc) => ({
            id: doc.id,
            icon: doc.filename.endsWith(".pdf")
              ? "picture_as_pdf"
              : "description",
            title: doc.filename,
            sub:
              doc.uploadStatus === "completed"
                ? `${Math.round(doc.chunkCount * 1.5)} words`
                : doc.uploadStatus,
            rawStatus: doc.uploadStatus,
          }));
          setSources(newSources);

          // Auto-select completed sources
          const completedIds = new Set(
            docs.filter((d) => d.uploadStatus === "completed").map((d) => d.id),
          );
          setSelectedIds(completedIds);

          // Hydrate the store with the chat history
          if (data.notebookChatHistory) {
            const { sessionId, messages } = data.notebookChatHistory;
            const chatMessages: ChatMessage[] = messages.map((m) => {
              let suggestedQuestions: string[] | undefined;
              if (m.metadata) {
                try {
                  const meta = JSON.parse(m.metadata);
                  if (Array.isArray(meta?.suggested_questions)) {
                    suggestedQuestions = meta.suggested_questions;
                  }
                } catch {
                  /* ignore */
                }
              }
              return {
                id: m.id,
                role: m.role as "user" | "assistant",
                content: m.content,
                isStreaming: false,
                suggestedQuestions,
              };
            });
            setInitialChat(sessionId || null, chatMessages);
          } else {
            setInitialChat(null, []);
          }
        } else if (errors) {
          console.error("Failed to load notebook data:", errors);
          setTitle("Untitled Notebook");
          setTempTitle("Untitled Notebook");
          setSources([]);
          setInitialChat(null, []);
        } else {
          setTitle("Untitled Notebook");
          setTempTitle("Untitled Notebook");
          setSources([]);
          setInitialChat(null, []);
        }
      } catch (e) {
        console.error(e);
        setTitle("Error Loading Notebook");
      } finally {
        setIsLoading(false);
      }
    };

    loadData();
  }, [decodedId, setInitialChat, setSelectedIds, setSources]);

  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  const handleSave = () => {
    if (tempTitle.trim()) {
      setTitle(tempTitle.trim());
    } else {
      setTempTitle(title);
    }
    setIsEditing(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleSave();
    if (e.key === "Escape") {
      setTempTitle(title);
      setIsEditing(false);
    }
  };

  if (isLoading) {
    return (
      <div className="flex flex-col items-center justify-center w-screen h-screen bg-bg-main relative overflow-hidden">
        <div
          className="absolute inset-0 pointer-events-none opacity-[0.03]"
          style={{
            backgroundImage:
              "repeating-linear-gradient(45deg, #000, #000 1px, transparent 1px, transparent 10px)",
          }}
        ></div>
        <div className="relative z-10 flex flex-col items-center">
          <div className="flex items-center justify-center w-20 h-20 rounded-2xl bg-white shadow-hard border-2 border-black animate-pulse mb-8 relative">
            <Logo className="w-12 h-12 text-black" />
            <div className="absolute -bottom-2 -right-2 w-6 h-6 bg-accent-main border-2 border-black rounded-full animate-bounce"></div>
          </div>
          <div className="flex items-center space-x-2 bg-white px-4 py-2 rounded-full border border-black shadow-hard-sm">
            <div className="w-2.5 h-2.5 bg-blue-500 rounded-full animate-bounce leading-none"></div>
            <div
              className="w-2.5 h-2.5 bg-red-500 rounded-full animate-bounce leading-none"
              style={{ animationDelay: "0.15s" }}
            ></div>
            <div
              className="w-2.5 h-2.5 bg-yellow-500 rounded-full animate-bounce leading-none"
              style={{ animationDelay: "0.3s" }}
            ></div>
            <div
              className="w-2.5 h-2.5 bg-green-500 rounded-full animate-bounce leading-none"
              style={{ animationDelay: "0.45s" }}
            ></div>
          </div>
          <h2 className="mt-6 text-sm font-black tracking-widest text-black uppercase">
            Loading Studio
          </h2>
        </div>
      </div>
    );
  }

  return (
    <>
      <header className="h-16 shrink-0 border-b border-border-bold flex items-center justify-between px-4 sm:px-6 bg-white z-20 relative overflow-hidden">
        <div
          className="absolute top-0 right-0 bottom-0 w-64 pointer-events-none opacity-10"
          style={{
            backgroundImage:
              "repeating-linear-gradient(45deg, #171717, #171717 1px, transparent 1px, transparent 6px)",
          }}
        ></div>
        <div className="flex items-center gap-4 relative z-10">
          <Link
            href="/"
            className="w-10 h-10 flex items-center justify-center transform transition-transform hover:-translate-y-0.5 group"
          >
            <Logo className="w-8 h-8 text-black" />
          </Link>
          {isEditing ? (
            <input
              ref={inputRef}
              type="text"
              value={tempTitle}
              onChange={(e) => setTempTitle(e.target.value)}
              onBlur={handleSave}
              onKeyDown={handleKeyDown}
              className="font-bold text-lg sm:text-xl tracking-tight uppercase bg-gray-50 border-b-2 border-black focus:outline-none px-1 py-0.5 min-w-[200px]"
            />
          ) : (
            <h1
              onClick={() => setIsEditing(true)}
              className="font-bold text-lg sm:text-xl tracking-tight uppercase truncate cursor-pointer hover:bg-gray-50 px-1 py-0.5 border-b-2 border-transparent hover:border-gray-200 transition-all"
            >
              {title}
            </h1>
          )}
        </div>
        <div className="flex items-center gap-2 sm:gap-3 relative z-10">
          <button className="hidden sm:flex items-center gap-2 px-4 py-2 bg-accent-main text-white border border-black shadow-hard-sm hover:shadow-hard hover:-translate-y-0.5 active:translate-y-0 active:shadow-none transition-all rounded-none font-bold text-sm">
            <span className="material-symbols-outlined icon-sm">add</span>
            Notebook
          </button>
          <div className="hidden sm:block h-6 w-px bg-gray-300 mx-2"></div>
          <button
            className="w-10 h-10 hidden sm:flex items-center justify-center border border-transparent hover:border-black hover:bg-gray-50 transition-all rounded-none text-gray-600 hover:text-black"
            title="Analytics"
          >
            <span className="material-symbols-outlined icon-sm">analytics</span>
          </button>
          <button
            className="w-10 h-10 hidden sm:flex items-center justify-center border border-transparent hover:border-black hover:bg-gray-50 transition-all rounded-none text-gray-600 hover:text-black"
            title="Share"
          >
            <span className="material-symbols-outlined icon-sm">share</span>
          </button>
          <button
            className="w-10 h-10 flex items-center justify-center border border-transparent hover:border-black hover:bg-gray-50 transition-all rounded-none text-gray-600 hover:text-black"
            title="Settings"
          >
            <span className="material-symbols-outlined icon-sm">settings</span>
          </button>
          <div className="w-8 h-8 sm:w-10 sm:h-10 border border-black overflow-hidden shadow-hard-sm ml-1 sm:ml-2">
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img
              alt="User Avatar"
              className="w-full h-full object-cover grayscale hover:grayscale-0 transition-all"
              src="https://lh3.googleusercontent.com/aida-public/AB6AXuDU-DdYBfrFz-om3NR3ti3vMwzqmnGUGZLKIiUxgjgXeggfaNKkY4I7KzszndsvY7r90cccF3eWELBKnYVytB6PDtTlC9zAwd6ULKKLUmvHlt76S9XdpTsG_v3MgdW5thM63xoMm-gknjo3UFZkCpDmnYnerCiDaIGG4_5FjTWyrXPqf5Z_UMWcgXrWelxirf9_Ne6wWI52X_af3MNcsIOQe-tBE9EeO01HQX6mLI9Ovlagabo_xz1alYPg0osyOjcZMQFRlhTLo83t"
            />
          </div>
        </div>
      </header>
      <EditorLayout notebookId={decodedId} />
    </>
  );
}
