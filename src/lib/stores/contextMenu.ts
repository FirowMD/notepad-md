import { writable } from 'svelte/store';

interface ContextMenuState {
  isOpen: boolean;
  x: number;
  y: number;
  fileId: string | null;
}

function createContextMenuStore() {
  const { subscribe, set, update } = writable<ContextMenuState>({
    isOpen: false,
    x: 0,
    y: 0,
    fileId: null
  });

  return {
    subscribe,
    open: (x: number, y: number, fileId: string) => set({ isOpen: true, x, y, fileId }),
    close: () => set({ isOpen: false, x: 0, y: 0, fileId: null })
  };
}

export const contextMenuStore = createContextMenuStore();