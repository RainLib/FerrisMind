import * as React from "react";
import { cn } from "@/lib/utils";

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
  className?: string;
}

export function Modal({
  isOpen,
  onClose,
  title,
  children,
  className,
}: ModalProps) {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/40 backdrop-blur-sm transition-opacity"
        onClick={onClose}
      />

      {/* Modal Content */}
      <div
        className={cn(
          "relative z-10 w-full max-w-md bg-white border-2 border-border-bold shadow-hard p-6 transform transition-all",
          className,
        )}
      >
        <div className="flex items-center justify-between mb-4 pb-2 border-b-2 border-border-bold">
          <h2 className="text-xl font-black uppercase tracking-tight">
            {title}
          </h2>
          <button
            onClick={onClose}
            className="p-1 text-gray-400 hover:text-black hover:bg-gray-100 transition-colors rounded-sm"
          >
            <span className="material-symbols-outlined icon-sm">close</span>
          </button>
        </div>
        <div className="mt-2">{children}</div>
      </div>
    </div>
  );
}
