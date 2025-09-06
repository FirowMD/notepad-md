import { writable } from 'svelte/store';

function createSidePanelStore() {
  const { subscribe, set, update } = writable(true);

  return {
    subscribe,
    toggle: () => update(visible => !visible),
    show: () => set(true),
    hide: () => set(false)
  };
}

export const sidePanelStore = createSidePanelStore();