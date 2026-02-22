import React from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import { cn } from "@/lib/utils";

interface MarkdownRendererProps {
  content: string;
  className?: string;
}

export function MarkdownRenderer({
  content,
  className,
}: MarkdownRendererProps) {
  return (
    <div
      className={cn(
        "prose prose-slate prose-lg max-w-none text-gray-800 leading-relaxed font-normal",
        // Additional tech-chic overrides
        "prose-strong:bg-gray-900 prose-strong:text-white prose-strong:px-1 prose-strong:font-bold",
        "prose-a:underline prose-a:decoration-2 prose-a:underline-offset-4 prose-a:decoration-accent-main float-none",
        "prose-headings:font-bold prose-headings:text-black",
        "prose-code:bg-white prose-code:border prose-code:border-gray-300 prose-code:px-1 prose-code:text-xs prose-code:font-mono prose-code:text-accent-secondary",
        "prose-ul:list-none prose-ul:p-0 prose-ul:space-y-4",
        "prose-li:flex prose-li:gap-4",
        className,
      )}
    >
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeRaw]}
        components={{
          p({ children, ...props }) {
            return (
              <p className="mb-4 last:mb-0" {...props}>
                {children}
              </p>
            );
          },
          li({ children, ...props }) {
            // Custom list item style from the design
            return (
              <li className="flex gap-4 items-start" {...props}>
                <div className="w-6 h-6 bg-white border border-black text-black flex items-center justify-center font-bold text-xs shrink-0 mt-0.5 shadow-[2px_2px_0px_0px_rgba(0,0,0,1)]">
                  <span className="material-symbols-outlined icon-sm">
                    check
                  </span>
                </div>
                <div>{children}</div>
              </li>
            );
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}
