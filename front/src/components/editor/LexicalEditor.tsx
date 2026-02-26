"use client";

import React, { useEffect, useRef } from "react";
import { LexicalComposer } from "@lexical/react/LexicalComposer";
import { RichTextPlugin } from "@lexical/react/LexicalRichTextPlugin";
import { ContentEditable } from "@lexical/react/LexicalContentEditable";
import { HistoryPlugin } from "@lexical/react/LexicalHistoryPlugin";
import { OnChangePlugin } from "@lexical/react/LexicalOnChangePlugin";
import { LexicalErrorBoundary } from "@lexical/react/LexicalErrorBoundary";
import { MarkdownShortcutPlugin } from "@lexical/react/LexicalMarkdownShortcutPlugin";

// Nodes
import { HeadingNode, QuoteNode } from "@lexical/rich-text";
import { ListNode, ListItemNode } from "@lexical/list";
import { LinkNode } from "@lexical/link";
import { CodeNode, CodeHighlightNode } from "@lexical/code";

// Markdown
import {
  TRANSFORMERS,
  $convertToMarkdownString,
  $convertFromMarkdownString,
} from "@lexical/markdown";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";

// Plugins
import { ToolbarPlugin } from "./plugins/ToolbarPlugin";
import { ComponentPickerPlugin } from "./plugins/ComponentPickerPlugin";

// Theme
const theme = {
  paragraph: "mb-4 text-base text-primary",
  text: {
    bold: "font-bold",
    italic: "italic",
    underline: "underline",
    strikethrough: "line-through",
    code: "bg-bg-sources rounded px-1.5 py-0.5 font-mono text-sm text-accent-secondary",
  },
  heading: {
    h1: "text-3xl font-black mb-4 mt-6 text-primary",
    h2: "text-2xl font-bold mb-3 mt-5 text-primary",
    h3: "text-xl font-bold mb-2 mt-4 text-primary",
    h4: "text-lg font-bold mb-2 mt-4 text-primary",
    h5: "text-base font-bold mb-1 mt-3 text-primary",
    h6: "text-sm font-bold mb-1 mt-3 text-gray-500 uppercase",
  },
  list: {
    ul: "list-disc ml-6 mb-4 text-primary",
    ol: "list-decimal ml-6 mb-4 text-primary",
    listitem: "mb-1",
    listitemChecked: "line-through text-gray-500",
    listitemUnchecked: "",
  },
  quote:
    "border-l-4 border-border-bold pl-4 py-2 italic mb-4 bg-bg-sources text-primary/80 rounded-r",
  code: "bg-primary text-bg-main p-4 rounded-md font-mono text-sm block overflow-x-auto mb-4",
  link: "text-accent-secondary hover:text-accent-main hover:underline cursor-pointer",
};

interface LexicalEditorProps {
  initialMarkdown: string;
  onChange: (markdown: string) => void;
}

// Plugin to load initial markdown
function InitialStatePlugin({ markdown }: { markdown: string }) {
  const [editor] = useLexicalComposerContext();
  const isFirstRender = useRef(true);

  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      editor.update(() => {
        $convertFromMarkdownString(markdown || "", TRANSFORMERS);
      });
    }
  }, [editor, markdown]);

  return null;
}

export function LexicalEditor({
  initialMarkdown,
  onChange,
}: LexicalEditorProps) {
  const customConfig = {
    // The editor theme
    namespace: "NoteEditor",
    theme,
    // Handling of errors during update
    onError(error: Error) {
      console.error(error);
    },
    // Any custom nodes go here
    nodes: [
      HeadingNode,
      ListNode,
      ListItemNode,
      QuoteNode,
      CodeNode,
      CodeHighlightNode,
      LinkNode,
    ],
  };

  return (
    <LexicalComposer initialConfig={customConfig}>
      <div className="relative h-full w-full flex flex-col font-sans group border border-border-bold bg-bg-main">
        <ToolbarPlugin />

        <div className="flex-1 relative overflow-auto px-4 py-2">
          <RichTextPlugin
            contentEditable={
              <ContentEditable className="h-full min-h-[500px] outline-none border-none resize-none py-2 text-primary leading-relaxed max-w-none" />
            }
            placeholder={
              <div className="absolute top-4 left-4 text-gray-400 pointer-events-none select-none italic text-sm">
                Start writing your note here... (Markdown &amp; &apos;/&apos;
                commands supported)
              </div>
            }
            ErrorBoundary={LexicalErrorBoundary}
          />
          <HistoryPlugin />
          <MarkdownShortcutPlugin transformers={TRANSFORMERS} />
          <ComponentPickerPlugin />

          <InitialStatePlugin markdown={initialMarkdown} />
          <OnChangePlugin
            onChange={(editorState) => {
              editorState.read(() => {
                const markdown = $convertToMarkdownString(TRANSFORMERS);
                onChange(markdown);
              });
            }}
          />
        </div>
      </div>
    </LexicalComposer>
  );
}
