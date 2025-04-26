import { writable } from 'svelte/store';
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from '../types/config';

function createConfigStore() {
  const { subscribe, set, update } = writable<AppConfig>({
    colorscheme: 'cerberus',
    recent_files: [],
    opened_files: []
  });

  return {
    subscribe,
    load: async () => {
      try {
        await invoke('load_config');
        const config = await invoke<AppConfig>('get_config');
        if (config) {
          set(config);
        }
        return config;
      } catch (error) {
        console.error('Error loading config:', error);
        return null;
      }
    },
    save: async (updates: Partial<AppConfig>) => {
      try {
        update(store => {
          const newConfig = { ...store, ...updates };
          invoke('save_config', { config: newConfig }).catch(error => {
            console.error('Error saving config:', error);
          });
          return newConfig;
        });
      } catch (error) {
        console.error('Error saving config:', error);
      }
    }
  };
}

export const configStore = createConfigStore();
