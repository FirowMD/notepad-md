<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { FileInfo } from '../types/file';
  import { fileStore } from '../stores/files';
  import { contextMenuStore } from '../stores/contextMenu';
  import { ArrowUp, ArrowDown } from 'lucide-svelte';
  import { save } from '@tauri-apps/plugin-dialog';
  import { onMount } from 'svelte';

  export let file: FileInfo;
  export let isActive: boolean = false;
  export let index: number;
  export let totalFiles: number;

  function handleClick() {
    fileStore.setActiveFile(file.id);
  }

  async function handleClose(e: MouseEvent) {
    e.stopPropagation();
    
    if (file.path) {
      try {
        await invoke('unwatch_file', { path: file.path });
      } catch (error) {
        console.error('Error unwatching file:', error);
      }
    }
    
    fileStore.removeFile(file.id);
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    contextMenuStore.open(e.clientX, e.clientY, file.id);
  }

  function moveUp() {
    if (index > 0) {
      const files = [...$fileStore.files];
      const temp = files[index];
      files[index] = files[index - 1];
      files[index - 1] = temp;
      fileStore.reorderFiles(files);
    }
  }

  function moveDown() {
    if (index < totalFiles - 1) {
      const files = [...$fileStore.files];
      const temp = files[index];
      files[index] = files[index + 1];
      files[index + 1] = temp;
      fileStore.reorderFiles(files);
    }
  }

  $: dateCreated = file.created.toLocaleDateString();
  $: timeCreated = file.created.toLocaleTimeString();

  async function handleOpenFilePath() {
    try {
      await invoke('run_explorer', { path: file.path });
    } catch (error) {
      console.error('Failed to open file path:', error);
    }
    contextMenuStore.close();
  }

  let contextMenuElement: HTMLDivElement;

  function handleWindowClick(event: MouseEvent) {
    if ($contextMenuStore.isOpen && 
        contextMenuElement && 
        !contextMenuElement.contains(event.target as Node)) {
      contextMenuStore.close();
    }
  }

  $: if ($contextMenuStore.isOpen && $contextMenuStore.fileId === file.id) {
    window.addEventListener('click', handleWindowClick);
  } else {
    window.removeEventListener('click', handleWindowClick);
  }

  let isRenaming = false;
  let newFileName = file.name;
  let inputElement: HTMLInputElement;

  $: if (isRenaming && inputElement) {
    inputElement.focus();
  }

  function handleRename() {
    isRenaming = true;
    newFileName = file.name;
    contextMenuStore.close();
  }

  async function handleRenameSubmit() {
    if (newFileName && newFileName !== file.name) {
      const oldExt = file.name.split('.').pop();
      const newExt = newFileName.split('.').pop();
      const finalName = !newExt && oldExt ? `${newFileName}.${oldExt}` : newFileName;
      
      if (file.path) {
        const dirPath = file.path.substring(0, file.path.lastIndexOf('\\') + 1);
        const newPath = dirPath + finalName;
        
        try {
          await invoke('rename_file', { 
            oldPath: file.path,
            newPath: newPath
          });
          
          await invoke('unwatch_file', { path: file.path });
          
          fileStore.updateFile(file.id, {
            name: finalName,
            path: newPath,
            modified: new Date()
          });
          
          await invoke('watch_file', { path: newPath });
        } catch (error) {
          console.error('Error renaming file:', error);
          newFileName = file.name;
          isRenaming = false;
          return;
        }
      } else {
        fileStore.updateFile(file.id, {
          name: finalName,
          modified: new Date()
        });
      }
    }
    isRenaming = false;
  }

  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleRenameSubmit();
    } else if (e.key === 'Escape') {
      isRenaming = false;
      newFileName = file.name;
    }
  }

  async function handleSaveAs() {
    try {
      const savePath = await save({});
      
      if (savePath) {
        await invoke('save_file', { 
          path: savePath,
          content: file.content
        });
        
        const pathParts = savePath.split(/[/\\]/);
        const fileName = pathParts[pathParts.length - 1];
        
        fileStore.updateFile(file.id, {
          path: savePath,
          name: fileName,
          modified: new Date()
        });
      }
    } catch (err) {
      console.error("Error saving file:", err);
    }
    contextMenuStore.close();
  }

  
  function handleKeydown(event: KeyboardEvent) {
    if (!isActive) return;

    if (event.ctrlKey && !event.shiftKey && event.key === 'e') {
      event.preventDefault();
      handleOpenFilePath();
    } else if (!event.ctrlKey && !event.shiftKey && event.key === 'F2') {
      event.preventDefault();
      handleRename();
    } else if (event.ctrlKey && event.shiftKey && event.key === 'S') {
      event.preventDefault();
      handleSaveAs();
    } else if (event.ctrlKey && !event.shiftKey && event.key === 'w') {
      event.preventDefault();
      handleClose(new MouseEvent('click'));
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

<div 
  class="relative group flex items-center w-full" 
  on:contextmenu={handleContextMenu}
  role="button"
  tabindex="0"
>
  <div class="flex-none px-2 opacity-0 group-hover:opacity-100 transition-opacity flex flex-col gap-0.5">
    <button
      type="button"
      class="p-0.5 hover:bg-surface-600 disabled:opacity-25"
      on:click={moveUp}
      disabled={index === 0}
    >
      <ArrowUp size={14} />
    </button>
    <button
      type="button"
      class="p-0.5 hover:bg-surface-600 disabled:opacity-25"
      on:click={moveDown}
      disabled={index === totalFiles - 1}
    >
      <ArrowDown size={14} />
    </button>
  </div>
  <button
    type="button"
    class="flex-1 btn rounded-none h-14 flex flex-col items-start overflow-hidden {isActive ? 'preset-filled-primary-500' : 'preset-filled-surface-500'}"
    on:click={handleClick}
    title={file.name}
  >
    {#if isRenaming}
      <input
        type="text"
        bind:value={newFileName}
        bind:this={inputElement}
        on:blur={handleRenameSubmit}
        on:keydown={handleRenameKeydown}
        class="w-full preset-filled-secondary-500 text-sm px-1 focus:outline-none"
      />
      <span class="text-xs text-left opacity-50 truncate w-full">{dateCreated} {timeCreated}</span>
    {:else}
      <div class="w-full min-w-0">
        <span class="text-sm text-left truncate block w-full">{file.name}</span>
        <span class="text-xs text-left opacity-50 truncate block w-full">{dateCreated} {timeCreated}</span>
      </div>
    {/if}
  </button>
  <button
    type="button"
    class="absolute right-1 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity"
    on:click={handleClose}
  >
    ×
  </button>

  {#if $contextMenuStore.isOpen && $contextMenuStore.fileId === file.id}
    <div
      bind:this={contextMenuElement}
      class="fixed z-50 w-40 bg-surface-700 shadow-xl rounded-none py-0.5 text-sm"
      style="left: {$contextMenuStore.x}px; top: {$contextMenuStore.y}px"
    >
      <button
        class="text-xs w-full px-3 py-1.5 text-left hover:bg-surface-600 transition-colors"
        on:click={handleOpenFilePath}
      >
        Open file path
      </button>
      <button
        class="text-xs w-full px-3 py-1.5 text-left hover:bg-surface-600 transition-colors"
        on:click={handleRename}
      >
        Rename
      </button>
      <button
        class="text-xs w-full px-3 py-1.5 text-left hover:bg-surface-600 transition-colors"
        on:click={handleSaveAs}
      >
        Save as
      </button>
      <button
        class="text-xs w-full px-3 py-1.5 text-left hover:bg-surface-600 transition-colors"
        on:click={(e) => { handleClose(e); contextMenuStore.close(); }}
      >
        Close
      </button>
    </div>
  {/if}
</div>