import { create } from "zustand";

type SaveStatus = "saved" | "saving" | "pending" | "error";

interface SaveStatusStore {
  status: SaveStatus;
  lastSavedAt: Date | null;
  setStatus: (status: SaveStatus) => void;
  setSaved: () => void;
  setPending: () => void;
  setSaving: () => void;
  setError: () => void;
}

export const useSaveStatus = create<SaveStatusStore>((set) => ({
  status: "saved",
  lastSavedAt: null,
  setStatus: (status) => set({ status }),
  setSaved: () => set({ status: "saved", lastSavedAt: new Date() }),
  setPending: () => set({ status: "pending" }),
  setSaving: () => set({ status: "saving" }),
  setError: () => set({ status: "error" }),
}));
