"use client";

import React, { useState, useEffect, useCallback } from "react";
import { LeftSidebar } from "./LeftSidebar";
import { ChatPanel } from "./ChatPanel";
import { RightSidebar } from "./RightSidebar";
import { CollapsedLeftSidebar } from "./CollapsedLeftSidebar";
import { CollapsedRightSidebar } from "./CollapsedRightSidebar";

function ResizeHandle({
  onMouseDown,
  isDragging,
}: {
  onMouseDown: (e: React.MouseEvent) => void;
  isDragging?: boolean;
}) {
  return (
    <div
      onMouseDown={onMouseDown}
      className={`h-full w-px bg-border-bold flex flex-col items-center justify-center transition-colors cursor-col-resize z-50 relative shrink-0 hover:bg-black group ${
        isDragging ? "bg-black" : ""
      }`}
    >
      {/* Invisible wider hit area */}
      <div className="absolute inset-y-0 -left-2 w-4 bg-transparent z-10" />
      {/* Visual pill */}
      <div
        className={`absolute h-8 w-1 rounded-full transition-colors z-20 ${
          isDragging ? "bg-black" : "bg-transparent group-hover:bg-black"
        }`}
      />
    </div>
  );
}

export function EditorLayout({ notebookId }: { notebookId: string }) {
  const [isLeftExpanded, setIsLeftExpanded] = useState(true);
  const [isRightExpanded, setIsRightExpanded] = useState(true);

  // Pixel widths for sidebars
  const [leftWidth, setLeftWidth] = useState(320);
  const [rightWidth, setRightWidth] = useState(320);

  const [isDraggingLeft, setIsDraggingLeft] = useState(false);
  const [isDraggingRight, setIsDraggingRight] = useState(false);

  const [isMobile, setIsMobile] = useState(false);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setMounted(true);
  }, []);

  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768);
    };
    checkMobile();
    window.addEventListener("resize", checkMobile);
    return () => window.removeEventListener("resize", checkMobile);
  }, []);

  // Left drag logic
  const handleLeftMouseMove = useCallback((e: MouseEvent) => {
    setLeftWidth(Math.max(240, Math.min(window.innerWidth * 0.6, e.clientX)));
  }, []);

  const handleLeftMouseUp = useCallback(() => {
    setIsDraggingLeft(false);
  }, []);

  useEffect(() => {
    if (isDraggingLeft) {
      window.addEventListener("mousemove", handleLeftMouseMove);
      window.addEventListener("mouseup", handleLeftMouseUp);
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";
    } else {
      window.removeEventListener("mousemove", handleLeftMouseMove);
      window.removeEventListener("mouseup", handleLeftMouseUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }
    return () => {
      window.removeEventListener("mousemove", handleLeftMouseMove);
      window.removeEventListener("mouseup", handleLeftMouseUp);
    };
  }, [isDraggingLeft, handleLeftMouseMove, handleLeftMouseUp]);

  // Right drag logic
  const handleRightMouseMove = useCallback((e: MouseEvent) => {
    setRightWidth(
      Math.max(
        240,
        Math.min(window.innerWidth * 0.6, window.innerWidth - e.clientX),
      ),
    );
  }, []);

  const handleRightMouseUp = useCallback(() => {
    setIsDraggingRight(false);
  }, []);

  useEffect(() => {
    if (isDraggingRight) {
      window.addEventListener("mousemove", handleRightMouseMove);
      window.addEventListener("mouseup", handleRightMouseUp);
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";
    } else {
      window.removeEventListener("mousemove", handleRightMouseMove);
      window.removeEventListener("mouseup", handleRightMouseUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }
    return () => {
      window.removeEventListener("mousemove", handleRightMouseMove);
      window.removeEventListener("mouseup", handleRightMouseUp);
    };
  }, [isDraggingRight, handleRightMouseMove, handleRightMouseUp]);

  const toggleLeftSidebar = () => setIsLeftExpanded(!isLeftExpanded);
  const toggleRightSidebar = () => setIsRightExpanded(!isRightExpanded);

  // Skip rendering until mounted to avoid hydration issues
  if (!mounted) return <div className="flex-1 w-full h-full bg-stone-50" />;

  // Mobile drawer rendering
  if (isMobile) {
    return (
      <div className="flex-1 flex overflow-hidden relative w-full h-full bg-stone-50">
        <div
          className={`absolute inset-y-0 left-0 z-40 transform transition-transform duration-300 ease-in-out ${
            isLeftExpanded ? "translate-x-0" : "-translate-x-full"
          }`}
        >
          <div className="h-full w-80 shadow-2xl">
            <LeftSidebar
              isMobile
              onToggle={() => setIsLeftExpanded(false)}
              notebookId={notebookId}
            />
          </div>
        </div>

        <div
          className={`absolute inset-y-0 right-0 z-40 transform transition-transform duration-300 ease-in-out ${
            isRightExpanded ? "translate-x-0" : "translate-x-full"
          }`}
        >
          <div className="h-full w-80 shadow-2xl flex">
            <RightSidebar
              isExpanded={true}
              onToggle={() => setIsRightExpanded(false)}
            />
          </div>
        </div>

        {(isLeftExpanded || isRightExpanded) && (
          <div
            className="absolute inset-0 bg-black/50 z-30"
            onClick={() => {
              setIsLeftExpanded(false);
              setIsRightExpanded(false);
            }}
          />
        )}

        <div className="flex-1 w-full h-full overflow-hidden">
          <ChatPanel
            notebookId={notebookId}
            onOpenLeft={
              !isLeftExpanded ? () => setIsLeftExpanded(true) : undefined
            }
          />
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 flex overflow-hidden w-full h-full bg-bg-main relative">
      {/* Fixed Collapsed Left Strip */}
      {!isLeftExpanded && (
        <CollapsedLeftSidebar onExpand={() => setIsLeftExpanded(true)} />
      )}

      {/* Expanded Left Sidebar + Handle */}
      <div
        style={{
          width: `${leftWidth}px`,
          display: isLeftExpanded ? "flex" : "none",
        }}
        className="h-full bg-bg-sources overflow-hidden shrink-0 flex-col relative"
      >
        <LeftSidebar onToggle={toggleLeftSidebar} notebookId={notebookId} />
        {isDraggingLeft && (
          <div className="absolute inset-0 z-50 pointer-events-auto cursor-col-resize" />
        )}
      </div>
      <div className={`h-full ${isLeftExpanded ? "block" : "hidden"}`}>
        <ResizeHandle
          isDragging={isDraggingLeft}
          onMouseDown={(e) => {
            e.preventDefault();
            setIsDraggingLeft(true);
          }}
        />
      </div>

      {/* Main Center Panel */}
      <div className="flex-1 min-w-0 h-full relative overflow-hidden bg-white">
        <ChatPanel
          notebookId={notebookId}
          onOpenLeft={!isLeftExpanded ? toggleLeftSidebar : undefined}
        />
        {(isDraggingLeft || isDraggingRight) && (
          <div className="absolute inset-0 z-50 pointer-events-auto cursor-col-resize" />
        )}
      </div>

      {/* Right Handle + Expanded Right Sidebar */}
      <div className={`h-full ${isRightExpanded ? "block" : "hidden"}`}>
        <ResizeHandle
          isDragging={isDraggingRight}
          onMouseDown={(e) => {
            e.preventDefault();
            setIsDraggingRight(true);
          }}
        />
      </div>
      <div
        style={{
          width: `${rightWidth}px`,
          display: isRightExpanded ? "flex" : "none",
        }}
        className="h-full relative overflow-hidden shrink-0 flex-col"
      >
        <RightSidebar isExpanded={true} onToggle={toggleRightSidebar} isPanel />
        {isDraggingRight && (
          <div className="absolute inset-0 z-50 pointer-events-auto cursor-col-resize" />
        )}
      </div>

      {/* Fixed Collapsed Right Strip */}
      {!isRightExpanded && (
        <CollapsedRightSidebar onExpand={() => setIsRightExpanded(true)} />
      )}
    </div>
  );
}
