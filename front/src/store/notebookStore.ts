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

export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  stages?: { stage: string; message: string; progress: number }[];
  metadata?: { intent?: string; error?: string; [key: string]: unknown };
  isStreaming?: boolean;
  suggestedQuestions?: string[];
}

interface NotebookState {
  sources: Source[];
  selectedIds: Set<string>;
  initialSessionId: string | null;
  initialMessages: ChatMessage[];
  setSources: (sources: Source[] | ((prev: Source[]) => Source[])) => void;
  setSelectedIds: (
    ids: Set<string> | ((prev: Set<string>) => Set<string>),
  ) => void;
  addSelectedId: (id: string) => void;
  setInitialChat: (sessionId: string | null, messages: ChatMessage[]) => void;
}

export const useNotebookStore = create<NotebookState>((set) => ({
  sources: [],
  selectedIds: new Set(),
  initialSessionId: null,
  initialMessages: [],
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
  setInitialChat: (sessionId, messages) =>
    set(() => ({
      initialSessionId: sessionId,
      initialMessages: messages,
    })),
}));
