"use client";

import React, { useRef, useState, useEffect } from "react";
import {
  Panel,
  Group as PanelGroup,
  Separator as PanelResizeHandle,
  PanelImperativeHandle,
} from "react-resizable-panels";
import { LeftSidebar } from "./LeftSidebar";
import { ChatPanel } from "./ChatPanel";
import { RightSidebar } from "./RightSidebar";
import { CollapsedLeftSidebar } from "./CollapsedLeftSidebar";
import { CollapsedRightSidebar } from "./CollapsedRightSidebar";

function ResizeHandle() {
  return (
    <PanelResizeHandle className="w-4 flex flex-col items-center justify-center bg-transparent hover:bg-black/5 transition-colors cursor-col-resize group z-50 relative -mx-2">
      <div className="h-12 w-1.5 bg-gray-200 rounded-full group-hover:bg-accent-main transition-colors" />
    </PanelResizeHandle>
  );
}

export function EditorLayout() {
  const [isLeftExpanded, setIsLeftExpanded] = useState(true);
  const [isRightExpanded, setIsRightExpanded] = useState(true);
  const [isMobile, setIsMobile] = useState(false);
  const [mounted, setMounted] = useState(false);

  const leftPanelRef = useRef<PanelImperativeHandle>(null);
  const rightPanelRef = useRef<PanelImperativeHandle>(null);

  useEffect(() => {
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

  const toggleLeftSidebar = () => {
    setIsLeftExpanded(!isLeftExpanded);
  };

  const toggleRightSidebar = () => {
    setIsRightExpanded(!isRightExpanded);
  };

  // Skip rendering until mounted to avoid hydration issues
  if (!mounted) return <div className="flex-1 w-full h-full bg-stone-50" />;

  if (isMobile) {
    return (
      <div className="flex-1 flex overflow-hidden relative w-full h-full bg-stone-50">
        <div
          className={`absolute inset-y-0 left-0 z-40 transform transition-transform duration-300 ease-in-out ${
            isLeftExpanded ? "translate-x-0" : "-translate-x-full"
          }`}
        >
          <div className="h-full w-80 shadow-2xl">
            <LeftSidebar isMobile onToggle={() => setIsLeftExpanded(false)} />
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
            onOpenLeft={
              !isLeftExpanded ? () => setIsLeftExpanded(true) : undefined
            }
            onOpenRight={
              !isRightExpanded ? () => setIsRightExpanded(true) : undefined
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

      {/* Resizable Area */}
      <PanelGroup
        autoSaveId="editor-layout"
        orientation="horizontal"
        className="flex-1 h-full"
      >
        {isLeftExpanded && (
          <>
            <Panel
              id="left-panel"
              panelRef={leftPanelRef}
              defaultSize={20}
              minSize={15}
              maxSize={35}
              className="h-full bg-bg-sources overflow-hidden"
            >
              <LeftSidebar onToggle={toggleLeftSidebar} />
            </Panel>
            <ResizeHandle />
          </>
        )}

        <Panel
          id="chat-panel"
          minSize={30}
          className="h-full relative overflow-hidden bg-white"
        >
          <ChatPanel
            onOpenLeft={!isLeftExpanded ? toggleLeftSidebar : undefined}
            onOpenRight={!isRightExpanded ? toggleRightSidebar : undefined}
          />
        </Panel>

        {isRightExpanded && (
          <>
            <ResizeHandle />
            <Panel
              id="right-panel"
              panelRef={rightPanelRef}
              defaultSize={25}
              minSize={20}
              maxSize={40}
              className="h-full relative overflow-hidden"
            >
              <RightSidebar
                isExpanded={true}
                onToggle={toggleRightSidebar}
                isPanel
              />
            </Panel>
          </>
        )}
      </PanelGroup>

      {/* Fixed Collapsed Right Strip */}
      {!isRightExpanded && (
        <CollapsedRightSidebar onExpand={() => setIsRightExpanded(true)} />
      )}
    </div>
  );
}
