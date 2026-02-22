"use client";

import React, { useRef, useState, useEffect } from "react";
import {
  Panel,
  Group,
  Separator,
  PanelImperativeHandle,
} from "react-resizable-panels";
import { LeftSidebar } from "./LeftSidebar";
import { ChatPanel } from "./ChatPanel";
import { RightSidebar } from "./RightSidebar";

function ResizeHandle() {
  return (
    <Separator className="w-1.5 flex flex-col items-center justify-center bg-transparent hover:bg-black/10 transition-colors cursor-col-resize group z-50">
      <div className="h-8 w-0.5 bg-gray-300 rounded-full group-hover:bg-gray-500 transition-colors" />
    </Separator>
  );
}

export function EditorLayout() {
  const [isLeftExpanded, setIsLeftExpanded] = useState(true);
  const [isRightExpanded, setIsRightExpanded] = useState(true);
  const [isMobile, setIsMobile] = useState(false);
  const [mounted, setMounted] = useState(false);

  const leftPanelRef = useRef<PanelImperativeHandle>(null);
  const rightPanelRef = useRef<PanelImperativeHandle>(null);

  // eslint-disable-next-line react-hooks/exhaustive-deps
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
    const panel = leftPanelRef.current;
    if (panel) {
      if (panel.isCollapsed()) {
        panel.expand();
        setIsLeftExpanded(true);
      } else {
        panel.collapse();
        setIsLeftExpanded(false);
      }
    } else {
      setIsLeftExpanded(!isLeftExpanded);
    }
  };

  const toggleRightSidebar = () => {
    const panel = rightPanelRef.current;
    if (panel) {
      if (panel.isCollapsed()) {
        panel.expand();
        setIsRightExpanded(true);
      } else {
        panel.collapse();
        setIsRightExpanded(false);
      }
    } else {
      setIsRightExpanded(!isRightExpanded);
    }
  };

  if (!mounted) return null;

  if (isMobile) {
    return (
      <div className="flex-1 flex overflow-hidden relative w-full h-full">
        {/* Mobile Left Sidebar Overlay */}
        <div
          className={`absolute inset-y-0 left-0 z-40 transform transition-transform duration-300 ease-in-out ${
            isLeftExpanded ? "translate-x-0" : "-translate-x-full"
          }`}
        >
          <div className="h-full w-80 shadow-2xl">
            <LeftSidebar isMobile onToggle={toggleLeftSidebar} />
          </div>
        </div>

        {/* Mobile Right Sidebar Overlay */}
        <div
          className={`absolute inset-y-0 right-0 z-40 transform transition-transform duration-300 ease-in-out ${
            isRightExpanded ? "translate-x-0" : "translate-x-full"
          }`}
        >
          <div className="h-full w-80 shadow-2xl flex">
            {/* We provide a wrapper to support the expanded/collapsed state on mobile if needed, but on mobile we just slide the whole thing */}
            <RightSidebar isExpanded={true} onToggle={toggleRightSidebar} />
          </div>
        </div>

        {/* Mobile Backdrop */}
        {(isLeftExpanded || isRightExpanded) && (
          <div
            className="absolute inset-0 bg-black/50 z-30"
            onClick={() => {
              if (isLeftExpanded) setIsLeftExpanded(false);
              if (isRightExpanded) setIsRightExpanded(false);
            }}
          />
        )}

        {/* Mobile Main Chat Panel */}
        <div className="flex-1 w-full h-full overflow-hidden">
          <ChatPanel
            onOpenLeft={toggleLeftSidebar}
            onOpenRight={toggleRightSidebar}
            isMobile
          />
        </div>
      </div>
    );
  }

  return (
    <Group direction="horizontal" className="flex-1 w-full h-full">
      <Panel
        panelRef={leftPanelRef}
        defaultSize={20}
        minSize={15}
        maxSize={30}
        collapsible
        collapsedSize={0}
        onResize={(size, id, prevSize) => {
          if (prevSize !== undefined) {
            // Handle resizing state updates if needed
          }
        }}
        className="flex flex-col h-full bg-bg-sources"
      >
        <LeftSidebar />
      </Panel>

      <ResizeHandle />

      <Panel minSize={30} className="flex flex-col h-full relative">
        <ChatPanel
          onOpenLeft={!isLeftExpanded ? toggleLeftSidebar : undefined}
          onOpenRight={!isRightExpanded ? toggleRightSidebar : undefined}
        />
      </Panel>

      <ResizeHandle />

      <Panel
        panelRef={rightPanelRef}
        defaultSize={25}
        minSize={20}
        maxSize={40}
        collapsible
        collapsedSize={5}
        onResize={(size, id, prevSize) => {
          // Handle resizing state
        }}
        className="flex flex-col h-full relative"
      >
        <RightSidebar
          isExpanded={isRightExpanded}
          onToggle={toggleRightSidebar}
          isPanel
        />
      </Panel>
    </Group>
  );
}
