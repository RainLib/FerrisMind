import * as React from "react";
import { cn } from "@/lib/utils";

interface PanelDetailViewProps {
  title: string;
  onBack: () => void;
  children: React.ReactNode;
  className?: string;
  icon?: string;
  headerActions?: React.ReactNode;
}

export function PanelDetailView({
  title,
  onBack,
  children,
  className,
  icon,
  headerActions,
}: PanelDetailViewProps) {
  return (
    <div
      className={cn(
        "absolute inset-0 bg-white z-50 flex flex-col border-l border-border-bold transform transition-transform animate-in slide-in-from-right-full duration-300 ease-out",
        className,
      )}
    >
      <div className="h-14 px-4 flex items-center justify-between border-b border-border-bold shrink-0 bg-gray-50/50">
        <div className="flex items-center gap-2 overflow-hidden flex-1">
          <button
            onClick={onBack}
            className="flex items-center justify-center p-1.5 -ml-1.5 text-gray-500 hover:text-black hover:bg-gray-200 rounded-sm transition-colors shrink-0"
            title="Back"
          >
            <span className="material-symbols-outlined icon-sm">
              arrow_back
            </span>
          </button>

          <div className="flex items-center gap-2 overflow-hidden flex-1 px-1">
            {icon && (
              <span className="material-symbols-outlined icon-sm text-gray-400 shrink-0">
                {icon}
              </span>
            )}
            <h2 className="text-xs font-bold text-gray-700 uppercase tracking-widest truncate">
              {title}
            </h2>
          </div>
        </div>

        {headerActions && (
          <div className="flex items-center gap-1 shrink-0 ml-2">
            {headerActions}
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto w-full h-full relative">
        {children}
      </div>
    </div>
  );
}
