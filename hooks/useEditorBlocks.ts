import { create } from "zustand";
import { Block } from "@blocknote/core";

interface EditorBlocksStore {
  blocks: Block[];
  setBlocks: (blocks: Block[]) => void;
}

export const useEditorBlocks = create<EditorBlocksStore>((set) => ({
  blocks: [],
  setBlocks: (blocks) => set({ blocks }),
}));
