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
  import { editorStore } from './stores/editor';
  import { notificationStore } from './stores/notification';
  import { getLanguageFromExtension } from './stores/language';
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';

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
        
        if (config.font_size) {
          editorStore.setFontSize(config.font_size);
        }

        if (config.word_wrap !== undefined) {
          editorStore.setWordWrap(config.word_wrap);
        }

        if (config.show_invisibles !== undefined) {
          editorStore.setShowInvisibles(config.show_invisibles);
        }
        
        if (config.opened_files) {
          const loadedFiles = [];
          for (const filePath of config.opened_files) {
            try {
              const fileData = await invoke('read_file', { 
                path: filePath,
                encoding: config.default_encoding || 'utf-8'
              }) as { content: string, hash: string };
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
              
              fileStore.addFile(fileInfo, true);
              loadedFiles.push(filePath);
              
              try {
                await invoke('watch_file', { path: filePath });
              } catch (error) {
                console.error('Error setting up file watch:', error);
              }
            } catch (error) {
              console.error('Error restoring file:', error);
              if (String(error).includes('File too large')) {
                notificationStore.show('File too large (>100MB). Large files are not supported.', 'error');
              }
            }
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
              fileStore.updateFileFromExternal(file.id, {
                content: fileData.content,
                hash: fileData.hash,
                modified: new Date()
              });
            }
          } catch (error) {
            console.error('Error reading updated file:', error);
            if (String(error).includes('File too large')) {
              notificationStore.show('File too large (>100MB). Large files are not supported.', 'error');
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
              if (String(error).includes('File too large')) {
                notificationStore.show('File too large (>100MB). Large files are not supported.', 'error');
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
      if (String(error).includes('File too large')) {
        notificationStore.show('File too large (>100MB). Large files are not supported.', 'error');
      } else {
        notificationStore.show("Error reading file", "error");
      }
    }
  }
</script>

<div 
  class="flex flex-col w-full h-full bg-surface-900 relative"
  role="presentation"
>
  <PanelTop />
  <PaneGroup direction="horizontal" class="flex w-full h-full">
    <Pane defaultSize={20} minSize={20}>
      <PanelSide />
    </Pane>
    <PaneResizer class="w-1 bg-surface-700 hover:bg-primary-500/20 transition-colors cursor-col-resize" />
    <Pane defaultSize={80} minSize={30}>
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