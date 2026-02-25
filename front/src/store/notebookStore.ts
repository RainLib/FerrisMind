import { create } from "zustand";

export interface Source {
  id: string;
  icon: string;
  title: string;
  sub: string;
  rawStatus?: string;
  file?: File;
  url?: string;
}

interface NotebookState {
  sources: Source[];
  selectedIds: Set<string>;
  setSources: (sources: Source[] | ((prev: Source[]) => Source[])) => void;
  setSelectedIds: (
    ids: Set<string> | ((prev: Set<string>) => Set<string>),
  ) => void;
  addSelectedId: (id: string) => void;
}

export const useNotebookStore = create<NotebookState>((set) => ({
  sources: [],
  selectedIds: new Set(),
  setSources: (updater) =>
    set((state) => ({
      sources: typeof updater === "function" ? updater(state.sources) : updater,
    })),
  setSelectedIds: (updater) =>
    set((state) => ({
      selectedIds:
        typeof updater === "function" ? updater(state.selectedIds) : updater,
    })),
  addSelectedId: (id) =>
    set((state) => {
      const nextIds = new Set(state.selectedIds);
      nextIds.add(id);
      return { selectedIds: nextIds };
    }),
}));
