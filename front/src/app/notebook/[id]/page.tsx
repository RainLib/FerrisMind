"use client";

import { EditorLayout } from "@/components/editor/EditorLayout";
import Link from "next/link";
import { Logo } from "@/components/ui/logo";
import { useState, useRef, useEffect } from "react";
import { useParams } from "next/navigation";
import { fetchGraphQL, GET_NOTEBOOK, Notebook } from "@/lib/graphql";

export default function Editor() {
  const params = useParams();
  const id = params?.id as string;
  const [title, setTitle] = useState("Loading...");
  const [isEditing, setIsEditing] = useState(false);
  const [tempTitle, setTempTitle] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const loadData = async () => {
      try {
        const { data, errors } = await fetchGraphQL<{ notebook: Notebook }>(
          GET_NOTEBOOK,
          { id },
        );
        if (data?.notebook) {
          setTitle(data.notebook.name);
          setTempTitle(data.notebook.name);
        } else if (errors) {
          console.error("Failed to load notebook:", errors);
          setTitle("Error Loading Notebook");
        }
      } catch (e) {
        console.error(e);
        setTitle("Error Loading Notebook");
      }
    };

    loadData();
  }, [id]);

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
      <EditorLayout notebookId={id} />
    </>
  );
}
