import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { configStore } from './configStore';

export interface MonacoThemeData {
  base?: string;
  inherit?: boolean;
  rules?: Array<{
    token: string;
    foreground?: string;
    background?: string;
    fontStyle?: string;
  }>;
  colors?: {
    [key: string]: string;
  };
}

function createMonacoThemeStore() {
  const { subscribe, set, update } = writable<string>('vs-dark');
  let monaco: any = null;

  return {
    subscribe,
    set,
    update,
    setMonaco: (monacoInstance: any) => {
      monaco = monacoInstance;
    },
    setTheme: async (themeName: string) => {
      set(themeName);
      
      // Update config
      const config = get(configStore);
      configStore.updateConfig({
        ...config,
        monaco_editor_theme: themeName
      });

      // Apply theme to Monaco Editor
      if (monaco) {
        if (themeName === 'vs' || themeName === 'vs-dark' || themeName === 'hc-black') {
          // Built-in themes
          monaco.editor.setTheme(themeName);
        } else {
          // Custom theme from file
          try {
            const themeContent = await invoke<string>('read_monaco_theme', { themeName });
            const themeData: MonacoThemeData = JSON.parse(themeContent);
            
            // Define the custom theme
            monaco.editor.defineTheme(themeName, {
              base: themeData.base || 'vs-dark',
              inherit: themeData.inherit !== false,
              rules: themeData.rules || [],
              colors: themeData.colors || {}
            });
            
            // Apply the theme
            monaco.editor.setTheme(themeName);
          } catch (error) {
            console.error('Error loading Monaco theme:', error);
            // Fallback to vs-dark if theme loading fails
            monaco.editor.setTheme('vs-dark');
            set('vs-dark');
          }
        }
      }
    },
    getAvailableThemes: async (): Promise<string[]> => {
      try {
        const themes = await invoke<string[]>('get_monaco_themes');
        return themes;
      } catch (error) {
        console.error('Error getting Monaco themes:', error);
        return ['vs', 'vs-dark', 'hc-black'];
      }
    }
  };
}

export const monacoThemeStore = createMonacoThemeStore();