import React, { createContext, useContext } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import { cn } from "@/lib/utils";

const ListContext = createContext({ depth: 0, ordered: false });

const CustomUl = ({
  children,
  className,
  ...props
}: React.HTMLAttributes<HTMLUListElement> & { node?: unknown }) => {
  const { depth } = useContext(ListContext);
  return (
    <ListContext.Provider value={{ depth: depth + 1, ordered: false }}>
      <ul
        className={cn(
          depth === 0
            ? "list-none p-0 space-y-4 mb-4"
            : "list-disc pl-6 mt-3 space-y-2 mb-3",
          className,
        )}
        {...props}
      >
        {children}
      </ul>
    </ListContext.Provider>
  );
};

const CustomOl = ({
  children,
  className,
  ...props
}: React.HTMLAttributes<HTMLOListElement> & { node?: unknown }) => {
  const { depth } = useContext(ListContext);
  return (
    <ListContext.Provider value={{ depth: depth + 1, ordered: true }}>
      <ol
        className={cn(
          depth === 0
            ? "list-decimal pl-6 space-y-4 mb-4"
            : "list-decimal pl-6 mt-3 space-y-2 mb-3",
          className,
        )}
        {...props}
      >
        {children}
      </ol>
    </ListContext.Provider>
  );
};

const CustomLi = ({
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLLIElement> & {
  node?: unknown;
  ordered?: boolean;
}) => {
  const { depth, ordered } = useContext(ListContext);
  const isTaskListItem = className?.includes("task-list-item");

  if (ordered || depth > 1 || isTaskListItem) {
    return (
      <li
        className={cn("text-gray-800 marker:text-gray-500", className)}
        {...props}
      >
        {children}
      </li>
    );
  }

  return (
    <li className={cn("flex gap-3 items-start", className)} {...props}>
      <div className="w-5 h-5 bg-white border border-black text-black flex items-center justify-center font-bold text-xs shrink-0 mt-[3px] shadow-[2px_2px_0px_0px_rgba(0,0,0,1)]">
        <span className="material-symbols-outlined text-[14px]">check</span>
      </div>
      <div className="flex-1 min-w-0">{children}</div>
    </li>
  );
};

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
        "prose-ul:m-0 prose-ul:p-0",
        className,
      )}
    >
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeRaw]}
        components={{
          p({ children, className, ...props }) {
            return (
              <p
                className={cn("mb-4 last:mb-0 first:mt-0", className)}
                {...props}
              >
                {children}
              </p>
            );
          },
          ul: CustomUl,
          ol: CustomOl,
          li: CustomLi,
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}
