<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import PanelSide from "../lib/PanelSide.svelte";
  import PanelEditor from "../lib/PanelEditor.svelte";
  import PanelTop from "../lib/PanelTop.svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { fileStore } from './stores/files';
  import { configStore } from './stores/configStore';
  import { themeStore } from './stores/theme';
  import { editorStore } from './stores/editor';
  import { getLanguageFromExtension } from './stores/language';
  import type { AppConfig } from './types/config';
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';

  let isDragging = false;
  let unlisten: (() => void) | undefined;
  let unlistenFileChange: (() => void) | undefined;

  function handleTabSwitch(event: KeyboardEvent) {
    if (event.ctrlKey && !event.altKey && event.key === 'Tab') {
      event.preventDefault();
      
      const files = $fileStore.files;
      if (files.length <= 1) return;
      
      const currentIndex = files.findIndex(f => f.id === $fileStore.activeFileId);
      let nextIndex;
      
      if (event.shiftKey) {
        // Ctrl+Shift+Tab - move backwards
        nextIndex = currentIndex <= 0 ? files.length - 1 : currentIndex - 1;
      } else {
        // Ctrl+Tab - move forwards
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
        
        if (config.opened_files) {
          for (const filePath of config.opened_files) {
            try {
              const content = await invoke('read_file', { path: filePath });
              const pathParts = filePath.split(/[/\\]/);
              const fileName = pathParts[pathParts.length - 1];
              const extension = fileName.split('.').pop()?.toLowerCase() || '';
              
              const nextId = ($fileStore.files.length + 1).toString();
              const fileInfo = {
                id: nextId,
                path: filePath,
                name: fileName,
                content: content as string,
                encoding: 'utf-8',
                language: getLanguageFromExtension(extension),
                created: new Date(),
                modified: new Date(),
                cursor: {
                  line: 1,
                  column: 1
                },
                stats: {
                  lines: (content as string).split('\n').length,
                  length: (content as string).length
                }
              };
              
              fileStore.addFile(fileInfo);
              
              try {
                await invoke('watch_file', { path: filePath });
              } catch (error) {
                console.error('Error setting up file watch:', error);
              }
            } catch (error) {
              console.error('Error restoring file:', error);
            }
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
            const content = await invoke('read_file', { path: filePath });
            fileStore.updateFile(file.id, {
              content: content as string,
              modified: new Date()
            });
          } catch (error) {
            console.error('Error reading updated file:', error);
          }
        }
      });
    };

    initialize();

    return () => {
      window.removeEventListener('keydown', handleTabSwitch);
      if (unlisten) unlisten();
      if (unlistenFileChange) unlistenFileChange();
    };
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleTabSwitch);
    if (unlisten) unlisten();
    if (unlistenFileChange) unlistenFileChange();
  });

  async function handleFileDrop(filePath: string) {
    try {
      const content = await invoke('read_file', { path: filePath });
      const pathParts = filePath.split(/[/\\]/);
      const fileName = pathParts[pathParts.length - 1];
      const extension = fileName.split('.').pop()?.toLowerCase() || '';
      
      const nextId = ($fileStore.files.length + 1).toString();
      const fileInfo = {
        id: nextId,
        path: filePath,
        name: fileName,
        content: content as string,
        encoding: 'utf-8',
        language: getLanguageFromExtension(extension),
        created: new Date(),
        modified: new Date(),
        cursor: {
          line: 1,
          column: 1
        },
        stats: {
          lines: (content as string).split('\n').length,
          length: (content as string).length
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
    }
  }
</script>

<div 
  class="flex flex-col w-full h-full bg-surface-900 relative"
  role="presentation"
>
  <PanelTop />
  <div class="flex flex-row w-full h-full">
    <PanelSide />
    <PanelEditor />
  </div>

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
</div>