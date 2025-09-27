import { writable, get } from 'svelte/store';
import type { FileInfo } from '../types/file';
import { configStore } from './configStore';
import { message, ask } from '@tauri-apps/plugin-dialog';
import { invoke } from "@tauri-apps/api/core";
import { getLanguageFromExtension } from './language';

interface FileStore {
  files: FileInfo[];
  activeFileId: string | null;
  nextId: number;
  untitledCounter: number;
}

function createFileStore() {
  const { subscribe, update } = writable<FileStore>({
    files: [],
    activeFileId: null,
    nextId: 1,
    untitledCounter: 0
  });

  return {
    subscribe,
    addUntitledFile: () => update(store => {
      const untitledName = `Untitled_${store.untitledCounter}.txt`;
      const fileInfo = {
        id: store.nextId.toString(),
        path: '',
        name: untitledName,
        content: '',
        encoding: 'utf-8',
        language: 'plaintext',
        created: new Date(),
        modified: new Date(),
        isModified: false,
        hash: '',
        cursor: { line: 1, column: 1 },
        stats: { lines: 1, length: 0 }
      };

      return {
        ...store,
        files: [...store.files, fileInfo],
        activeFileId: store.nextId.toString(),
        nextId: store.nextId + 1,
        untitledCounter: store.untitledCounter + 1
      };
    }),
    addFile: (file: FileInfo, skipConfigSave: boolean = false) => update(store => {
      const existingFile = store.files.find(f => f.hash === file.hash && file.hash !== '');
      if (existingFile) {
        return {
          ...store,
          activeFileId: existingFile.id
        };
      }
      const fileWithId = {
        ...file,
        id: store.nextId.toString(),
        isModified: file.isModified ?? false
      };

      const newStore = {
        ...store,
        files: [...store.files, fileWithId],
        activeFileId: store.nextId.toString(),
        nextId: store.nextId + 1
      };
      
      if (!skipConfigSave) {
        const openedFiles = newStore.files
          .filter(f => f.path)
          .map(f => f.path);
        configStore.save({ opened_files: openedFiles });
      }
      
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
        const fileData = await invoke('read_file', { 
          path: filePath,
          encoding: config.default_encoding || 'utf-8'
        }) as { content: string, hash: string };
        
        let fileSystemModified: Date | undefined;
        try {
          const modifiedTimestamp = await invoke('get_file_metadata', { path: filePath }) as number;
          fileSystemModified = new Date(modifiedTimestamp * 1000);
        } catch (error) {
          console.error('Error getting file metadata:', error);
        }
        
        const pathParts = filePath.split(/[/\\]/);
        const fileName = pathParts[pathParts.length - 1];
        const extension = fileName.split('.').pop()?.toLowerCase() || '';
        
        const store = get(fileStore);
        const fileInfo = {
          id: store.nextId.toString(),
          path: filePath,
          name: fileName,
          content: fileData.content,
          encoding: 'utf-8',
          language: getLanguageFromExtension(extension),
          created: new Date(),
          modified: new Date(),
          fileSystemModified,
          isModified: false,
          hash: fileData.hash,
          cursor: { line: 1, column: 1 },
          stats: {
            lines: fileData.content.split('\n').length,
            length: fileData.content.length
          }
        };
        
        fileStore.addFile(fileInfo);
        fileStore.setActiveFile(store.nextId.toString());
        
        await configStore.save({
          recent_files: recentFiles.slice(1)
        });
        
        await invoke('watch_file', { path: filePath });
      } catch (error) {
        const errorStr = String(error);
        if (errorStr.includes('File too large')) {
          await message('File too large (>100MB). Large files are not supported.', { title: 'Error' });
        } else if (errorStr.includes('PERMISSION_DENIED')) {
          const isAdmin = await invoke('check_admin_privileges') as boolean;
          if (!isAdmin) {
            const shouldRelaunch = await ask(
              `Failed to open file due to insufficient permissions.\n\nWould you like to restart the application with administrator privileges?`,
              { title: 'Permission Denied', kind: 'warning' }
            );
            if (shouldRelaunch) {
              await invoke('relaunch_as_admin', { args: [filePath] });
            }
          } else {
            await message(`Failed to restore file: ${filePath}\n\nPermission denied even with admin privileges.`, { title: 'Error' });
          }
        } else {
          await message(`Failed to restore file: ${filePath}`, { title: 'Error' });
        }
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
    updateFileFromExternal: (id: string, updates: Partial<FileInfo>) => update(store => ({
      ...store,
      files: store.files.map(f => {
        if (f.id === id) {
          return { ...f, ...updates, isModified: f.isModified };
        }
        return f;
      })
    })),
    markAsModified: (id: string) => update(store => ({
      ...store,
      files: store.files.map(f => f.id === id ? { ...f, isModified: true, modified: new Date() } : f)
    })),
    markAsSaved: (id: string) => update(store => ({
      ...store,
      files: store.files.map(f => f.id === id ? { ...f, isModified: false } : f)
    })),
    reorderFiles: (newFiles: FileInfo[]) => update(store => {
      const updatedFiles = newFiles.map(file => ({
        ...file
      }));

      const newStore = {
        ...store,
        files: updatedFiles,
        activeFileId: store.activeFileId
      };
      
      const openedFiles = newStore.files
        .filter(f => f.path)
        .map(f => f.path);
      configStore.save({ opened_files: openedFiles });
      
      return newStore;
    })
  };
}

export const fileStore = createFileStore();