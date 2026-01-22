import { create } from "zustand";

type CoverImageStore = {
  url?: string;
  documentId?: string;
  isOpen: boolean;
  onOpen: (documentId?: string) => void;
  onClose: () => void;
  onReplace: (url: string) => void;
};

export const useCoverImage = create<CoverImageStore>((set) => ({
  url: undefined,
  documentId: undefined,
  isOpen: false,
  onOpen: (documentId?: string) =>
    set({ isOpen: true, url: undefined, documentId }),
  onClose: () => set({ isOpen: false, url: undefined, documentId: undefined }),
  onReplace: (url: string) => set({ isOpen: true, url }),
}));
