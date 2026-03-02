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

export type ActivityType = "note" | "audio" | "video";

export interface ActivityItem {
  id: string;
  type: ActivityType;
  title: string;
  content?: string;
  createdAt: Date;
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
  isAddSourceModalOpen: boolean;
  setIsAddSourceModalOpen: (isOpen: boolean) => void;
  activities: ActivityItem[];
  setActivities: (
    activities: ActivityItem[] | ((prev: ActivityItem[]) => ActivityItem[]),
  ) => void;
  addActivity: (activity: ActivityItem) => void;
  activeActivity: { id: string; type: ActivityType } | null;
  setActiveActivity: (
    activity: { id: string; type: ActivityType } | null,
  ) => void;
  activeDetailId: string | null;
  setActiveDetailId: (id: string | null) => void;
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
  isAddSourceModalOpen: false,
  setIsAddSourceModalOpen: (isOpen) => set({ isAddSourceModalOpen: isOpen }),
  activities: [],
  setActivities: (updater) =>
    set((state) => ({
      activities:
        typeof updater === "function" ? updater(state.activities) : updater,
    })),
  addActivity: (activity) =>
    set((state) => ({ activities: [activity, ...state.activities] })),
  activeActivity: null,
  setActiveActivity: (activity) => set({ activeActivity: activity }),
  activeDetailId: null,
  setActiveDetailId: (id) => set({ activeDetailId: id }),
}));
