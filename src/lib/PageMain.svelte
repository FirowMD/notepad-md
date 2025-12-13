<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import PanelSide from "../lib/PanelSide.svelte";
  import PanelEditor from "../lib/PanelEditor.svelte";
  import PanelTop from "../lib/PanelTop.svelte";
  import NotificationContainer from "../lib/NotificationContainer.svelte";
  import { PaneGroup, Pane, PaneResizer } from "paneforge";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { fileStore } from './stores/files';
  import { configStore } from './stores/configStore';
  import { themeStore } from './stores/theme';
  import { monacoThemeStore } from './stores/monacoTheme';
  import { editorStore } from './stores/editor';
  import { notificationStore } from './stores/notification';
  import { sidePanelStore } from './stores/sidePanelStore';
  import { getLanguageFromExtension } from './stores/language';
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { isTauri } from '@tauri-apps/api/core';

  let isDragging = false;
  let unlisten: (() => void) | undefined;
  let unlistenFileChange: (() => void) | undefined;
  let unlistenFilesUpdated: (() => void) | undefined;

  function handleTabSwitch(event: KeyboardEvent) {
    if (event.ctrlKey && !event.altKey && event.code === 'Tab') {
      event.preventDefault();
      
      const files = $fileStore.files;
      if (files.length <= 1) return;
      
      const currentIndex = files.findIndex(f => f.id === $fileStore.activeFileId);
      let nextIndex;
      
      if (event.shiftKey) {
        nextIndex = currentIndex <= 0 ? files.length - 1 : currentIndex - 1;
      } else {
        nextIndex = currentIndex >= files.length - 1 ? 0 : currentIndex + 1;
      }
      
      fileStore.setActiveFile(files[nextIndex].id);
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleTabSwitch);
    
    const initialize = async () => {
      const config = await configStore.load();
      
      if (config) {
        if (config.colorscheme) {
          themeStore.loadTheme(config.colorscheme);
        }

        if (config.monaco_editor_theme) {
          // Just set the store value, the actual theme will be applied when Monaco loads
          monacoThemeStore.set(config.monaco_editor_theme);
        }
        
        if (config.font_size) {
          editorStore.setFontSize(config.font_size);
        }

        if (config.word_wrap !== undefined) {
          editorStore.setWordWrap(config.word_wrap);
        }

        if (config.show_invisibles !== undefined) {
          editorStore.setShowInvisibles(config.show_invisibles);
        }
        
        if (isTauri()) {
          const win = getCurrentWindow();
          try { await win.setDecorations(!(config.transparent_mode || false)); } catch {}
          try { await win.setAlwaysOnTop(config.transparent_mode || false); } catch {}
        }
        
        if (config.opened_files) {
          const loadedFiles = [];
          for (const filePath of config.opened_files) {
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
              
              const nextId = ($fileStore.files.length + 1).toString();
              const fileInfo = {
                id: nextId,
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
                cursor: {
                  line: 1,
                  column: 1
                },
                stats: {
                  lines: fileData.content.split('\n').length,
                  length: fileData.content.length
                }
              };
              
              fileStore.addFile(fileInfo, true, false);
              loadedFiles.push(filePath);
              
              try {
                await invoke('watch_file', { path: filePath });
              } catch (error) {
                console.error('Error setting up file watch:', error);
              }
            } catch (error) {
              console.error('Error restoring file:', error);
              const errorStr = String(error);
              if (errorStr.includes('File too large')) {
                notificationStore.show('File too large (>100MB). Large files are not supported.', 'error');
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
                  notificationStore.show('Permission denied even with admin privileges.', 'error');
                }
              }
            }
          }
          if ($fileStore.files.length > 0) {
            const lastFileId = $fileStore.files[$fileStore.files.length - 1].id;
            fileStore.setActiveFile(lastFileId);
          }
          
          if (loadedFiles.length > 0) {
            configStore.save({ opened_files: loadedFiles });
          }
        }
      }
      
      unlisten = await getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === 'over') {
          isDragging = true;
        } else if (event.payload.type === 'drop') {
          for (const filePath of event.payload.paths) {
            handleFileDrop(filePath);
          }
          isDragging = false;
        } else {
          isDragging = false;
        }
      });

      unlistenFileChange = await listen('file-changed', async (event) => {
        const filePath = event.payload as string;
        const file = $fileStore.files.find(f => f.path === filePath);
        
        if (file) {
          try {
            const fileData = await invoke('read_file', { path: filePath }) as { content: string, hash: string };
            if (fileData.hash !== file.hash) {
              // Get updated file system metadata when file changes externally
              let fileSystemModified: Date | undefined;
              try {
                const modifiedTimestamp = await invoke('get_file_metadata', { path: filePath }) as number;
                fileSystemModified = new Date(modifiedTimestamp * 1000);
              } catch (error) {
                console.error('Error getting file metadata:', error);
              }
              
              fileStore.updateFileFromExternal(file.id, {
                content: fileData.content,
                hash: fileData.hash,
                modified: new Date(),
                fileSystemModified
              });
            }
          } catch (error) {
            console.error('Error reading updated file:', error);
            const errorStr = String(error);
            if (errorStr.includes('File too large')) {
              notificationStore.show('File too large (>100MB). Large files are not supported.', 'error');
            } else if (errorStr.includes('PERMISSION_DENIED')) {
              const isAdmin = await invoke('check_admin_privileges') as boolean;
              if (!isAdmin) {
                const shouldRelaunch = await ask(
                  `Failed to read file due to insufficient permissions.\n\nWould you like to restart the application with administrator privileges?`,
                  { title: 'Permission Denied', kind: 'warning' }
                );
                if (shouldRelaunch) {
                  const files = $fileStore.files.map(f => f.path).filter(p => p);
                  await invoke('relaunch_as_admin', { args: files });
                }
              } else {
                notificationStore.show('Permission denied even with admin privileges.', 'error');
              }
            }
          }
        }
      });

      unlistenFilesUpdated = await listen('files-updated', async () => {
        const config = await configStore.load();
        
        if (config && config.opened_files) {
          for (const filePath of config.opened_files) {
            const existingFile = $fileStore.files.find(f => f.path === filePath);
            if (existingFile) {
              fileStore.setActiveFile(existingFile.id);
              continue;
            }
            
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
              
              const nextId = ($fileStore.files.length + 1).toString();
              const fileInfo = {
                id: nextId,
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
                cursor: {
                  line: 1,
                  column: 1
                },
                stats: {
                  lines: fileData.content.split('\n').length,
                  length: fileData.content.length
                }
              };
              
              fileStore.addFile(fileInfo);
              
              try {
                await invoke('watch_file', { path: filePath });
              } catch (error) {
                console.error('Error setting up file watch:', error);
              }
            } catch (error) {
              console.error('Error loading new file:', error);
              const errorStr = String(error);
              if (errorStr.includes('File too large')) {
                notificationStore.show('File too large (>100MB). Large files are not supported.', 'error');
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
                  notificationStore.show('Permission denied even with admin privileges.', 'error');
                }
              }
            }
          }
        }
      });
    };

    initialize();

    return () => {
      window.removeEventListener('keydown', handleTabSwitch);
      if (unlisten) unlisten();
      if (unlistenFileChange) unlistenFileChange();
      if (unlistenFilesUpdated) unlistenFilesUpdated();
    };
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleTabSwitch);
    if (unlisten) unlisten();
    if (unlistenFileChange) unlistenFileChange();
    if (unlistenFilesUpdated) unlistenFilesUpdated();
  });

  async function handleFileDrop(filePath: string) {
    try {
      const fileData = await invoke('read_file', { 
        path: filePath,
        encoding: $editorStore.encoding 
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
      
      const nextId = ($fileStore.files.length + 1).toString();
      const fileInfo = {
        id: nextId,
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
        cursor: {
          line: 1,
          column: 1
        },
        stats: {
          lines: fileData.content.split('\n').length,
          length: fileData.content.length
        }
      }; 

      fileStore.addFile(fileInfo);
      
      try {
        await invoke('watch_file', { path: filePath });
      } catch (error) {
        console.error('Error setting up file watch:', error);
      }
    } catch (error) {
      console.error('Error reading file:', error);
      const errorStr = String(error);
      if (errorStr.includes('File too large')) {
        notificationStore.show('File too large (>100MB). Large files are not supported.', 'error');
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
          notificationStore.show('Permission denied even with admin privileges.', 'error');
        }
      } else {
        notificationStore.show("Error reading file", "error");
      }
    }
  }
  
  $: if (isTauri()) {
    const win = getCurrentWindow();
    try { win.setDecorations(true); } catch {}
    try { win.setAlwaysOnTop($configStore.transparent_mode || false); } catch {}
    try {
      const next = ($configStore.transparent_mode || false)
        ? String(($configStore.window_opacity ?? 0.85))
        : '1';
      document.documentElement.style.setProperty('--overlayOpacity', next);
    } catch {}
    try {
      if ($configStore.transparent_mode) {
        document.documentElement.classList.add('transparent-mode');
      } else {
        document.documentElement.classList.remove('transparent-mode');
      }
    } catch {}
  }
</script>

<div 
  class="preset-gradient flex flex-col w-full h-full relative"
  style="opacity: var(--overlayOpacity, 1)"
  role="presentation"
>
  <PanelTop />
  <PaneGroup direction="horizontal" class="flex w-full h-full">
    {#if $sidePanelStore}
      <Pane defaultSize={20} minSize={20}>
        <PanelSide />
      </Pane>
      <PaneResizer class="preset-glass w-1 hover:bg-primary-500/40 transition-all duration-200 cursor-col-resize" />
    {/if}
    <Pane defaultSize={$sidePanelStore ? 80 : 100} minSize={30}>
      <PanelEditor />
    </Pane>
  </PaneGroup>

  {#if isDragging}
    <div 
      class="absolute inset-0 bg-surface-900/80 flex items-center justify-center"
      role="presentation"
    >
      <div class="w-96 text-center">
        <p class="text-lg">Drop file to open</p>
      </div>
    </div>
  {/if}

  <NotificationContainer />
</div>
