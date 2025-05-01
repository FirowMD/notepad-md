import { writable } from 'svelte/store';
import { configStore } from './configStore';

export type Theme = 
  | 'NotepadMD'
  | 'catppuccin' | 'cerberus' | 'concord' | 'crimson' | 'fennec' 
  | 'hamlindigo' | 'legacy' | 'mint' | 'modern' | 'mona' | 'nosh' 
  | 'nouveau' | 'pine' | 'reign' | 'rocket' | 'rose' | 'sahara' 
  | 'seafoam' | 'terminus' | 'vintage' | 'vox' | 'wintry';

function createThemeStore() {
  const { subscribe, set } = writable<Theme>('NotepadMD');

  return {
    subscribe,
    setTheme: (theme: Theme) => {
      document.documentElement.setAttribute('data-theme', theme);
      configStore.save({ colorscheme: theme });
      set(theme);
    },
    loadTheme: (theme: Theme) => {
      document.documentElement.setAttribute('data-theme', theme);
      set(theme);
    }
  };
}

export const themeStore = createThemeStore();