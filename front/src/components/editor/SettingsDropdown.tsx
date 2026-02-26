"use client";

import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { useTheme } from "@/lib/ThemeContext";

export function SettingsDropdown() {
  const { theme, setTheme } = useTheme();

  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        <button
          className="w-10 h-10 flex items-center justify-center border border-transparent hover:border-border-bold hover:bg-bg-sources transition-all rounded-none text-gray-600 hover:text-primary"
          title="Settings"
        >
          <span className="material-symbols-outlined icon-sm">settings</span>
        </button>
      </DropdownMenu.Trigger>

      <DropdownMenu.Portal>
        <DropdownMenu.Content
          className="min-w-[200px] bg-bg-main border border-border-bold shadow-hard z-100 p-1 animate-in fade-in zoom-in duration-75"
          sideOffset={5}
          align="end"
        >
          <div className="px-2 py-1.5 text-[10px] font-bold uppercase tracking-widest text-gray-500">
            Appearance
          </div>

          <DropdownMenu.Item
            className={`flex items-center gap-2 px-2 py-1.5 text-sm cursor-pointer outline-none transition-colors ${
              theme === "light"
                ? "bg-accent-light text-accent-secondary"
                : "hover:bg-bg-sources"
            }`}
            onClick={() => setTheme("light")}
          >
            <span className="material-symbols-outlined icon-sm">
              light_mode
            </span>
            <span className="font-medium">Light</span>
            {theme === "light" && (
              <span className="material-symbols-outlined icon-sm ml-auto">
                check
              </span>
            )}
          </DropdownMenu.Item>

          <DropdownMenu.Item
            className={`flex items-center gap-2 px-2 py-1.5 text-sm cursor-pointer outline-none transition-colors ${
              theme === "dark"
                ? "bg-accent-light text-accent-secondary"
                : "hover:bg-bg-sources"
            }`}
            onClick={() => setTheme("dark")}
          >
            <span className="material-symbols-outlined icon-sm">dark_mode</span>
            <span className="font-medium">Dark</span>
            {theme === "dark" && (
              <span className="material-symbols-outlined icon-sm ml-auto">
                check
              </span>
            )}
          </DropdownMenu.Item>

          <DropdownMenu.Item
            className={`flex items-center gap-2 px-2 py-1.5 text-sm cursor-pointer outline-none transition-colors ${
              theme === "system"
                ? "bg-accent-light text-accent-secondary"
                : "hover:bg-bg-sources"
            }`}
            onClick={() => setTheme("system")}
          >
            <span className="material-symbols-outlined icon-sm">
              desktop_windows
            </span>
            <span className="font-medium">System</span>
            {theme === "system" && (
              <span className="material-symbols-outlined icon-sm ml-auto">
                check
              </span>
            )}
          </DropdownMenu.Item>

          <DropdownMenu.Separator className="h-px bg-border-light my-1" />

          <div className="px-2 py-1.5 text-[10px] font-bold uppercase tracking-widest text-gray-500">
            Notebook Settings
          </div>

          <DropdownMenu.Item className="flex items-center gap-2 px-2 py-1.5 text-sm cursor-pointer outline-none hover:bg-bg-sources transition-colors group">
            <span className="material-symbols-outlined icon-sm text-gray-500 group-hover:text-primary">
              delete
            </span>
            <span className="font-medium">Delete Notebook</span>
          </DropdownMenu.Item>
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
}
