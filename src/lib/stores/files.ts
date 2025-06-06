import { writable, get } from 'svelte/store';
import type { FileInfo } from '../types/file';
import { configStore } from './configStore';
import { message } from '@tauri-apps/plugin-dialog';
import { invoke } from "@tauri-apps/api/core";
import { getLanguageFromExtension } from './language';

interface FileStore {
  files: FileInfo[];
  activeFileId: string | null;
  nextId: number;
}

function createFileStore() {
  const { subscribe, set, update } = writable<FileStore>({
    files: [],
    activeFileId: null,
    nextId: 1
  });

  return {
    subscribe,
    addFile: (file: FileInfo) => update(store => {
      const existingFile = store.files.find(f => f.path === file.path && file.path !== '');
      if (existingFile) {
        return {
          ...store,
          activeFileId: existingFile.id
        };
      }
      // Use nextId for new files
      const fileWithId = {
        ...file,
        id: store.nextId.toString()
      };

      const newStore = {
        ...store,
        files: [...store.files, fileWithId],
        activeFileId: store.nextId.toString(),
        nextId: store.nextId + 1
      };
      
      const openedFiles = newStore.files
        .filter(f => f.path)
        .map(f => f.path);
      configStore.save({ opened_files: openedFiles });
      
      return newStore;
    }),
    removeFile: (id: string) => update(store => {
      const fileToRemove = store.files.find(f => f.id === id);
      if (fileToRemove?.path) {
        const config = get(configStore);
        const recentFiles = config.recent_files || [];
        const updatedRecent = [fileToRemove.path, 
          ...recentFiles.filter(f => f !== fileToRemove.path)
        ].slice(0, 100);
        configStore.save({
          recent_files: updatedRecent
        });
      }
      
      const remainingFiles = store.files.filter(f => f.id !== id);
      let newActiveId = store.activeFileId;
      
      if (store.activeFileId === id && remainingFiles.length > 0) {
        const currentIndex = store.files.findIndex(f => f.id === id);
        const nextFile = remainingFiles[currentIndex] || remainingFiles[currentIndex - 1];
        newActiveId = nextFile ? nextFile.id : null;
      } else if (remainingFiles.length === 0) {
        newActiveId = null;
      }
      
      const newStore = {
        ...store,
        files: remainingFiles,
        activeFileId: newActiveId
      };
      
      const openedFiles = newStore.files
        .filter(f => f.path)
        .map(f => f.path);
      configStore.save({ opened_files: openedFiles });
      
      return newStore;
    }),
    restoreRecentFile: async () => {
      const config = await configStore.load();
      if (!config) {
        await message('Failed to load configuration', { title: 'Error' });
        return;
      }
      
      const recentFiles = config.recent_files || [];
      
      if (recentFiles.length === 0) {
        await message('No recent files to restore', { title: 'Restore File' });
        return;
      }
      
      const filePath = recentFiles[0];
      
      try {
        const content = await invoke('read_file', { 
          path: filePath,
          encoding: config.default_encoding || 'utf-8'
        });
        const pathParts = filePath.split(/[/\\]/);
        const fileName = pathParts[pathParts.length - 1];
        const extension = fileName.split('.').pop()?.toLowerCase() || '';
        
        const store = get(fileStore);
        const fileInfo = {
          id: store.nextId.toString(),
          path: filePath,
          name: fileName,
          content: content as string,
          encoding: 'utf-8',
          language: getLanguageFromExtension(extension),
          created: new Date(),
          modified: new Date(),
          cursor: { line: 1, column: 1 },
          stats: {
            lines: (content as string).split('\n').length,
            length: (content as string).length
          }
        };
        
        fileStore.addFile(fileInfo);
        fileStore.setActiveFile(store.nextId.toString());
        
        await configStore.save({
          recent_files: recentFiles.slice(1)
        });
        
        await invoke('watch_file', { path: filePath });
      } catch (error) {
        await message(`Failed to restore file: ${filePath}`, { title: 'Error' });
        await configStore.save({
          recent_files: recentFiles.slice(1)
        });
      }
    },
    setActiveFile: (id: string) => update(store => ({
      ...store,
      activeFileId: id
    })),
    updateFile: (id: string, updates: Partial<FileInfo>) => update(store => ({
      ...store,
      files: store.files.map(f => f.id === id ? { ...f, ...updates } : f)
    })),
    reorderFiles: (newFiles: FileInfo[]) => update(store => {
      const updatedFiles = newFiles.map(file => ({
        ...file
      }));

      return {
        ...store,
        files: updatedFiles,
        activeFileId: store.activeFileId
      };
    })
  };
}

export const fileStore = createFileStore();