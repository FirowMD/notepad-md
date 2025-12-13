<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import TabFile from "./elems/TabFile.svelte";
  import { fileStore } from "./stores/files";
  import type { FileInfo } from './types/file';
  import { Search } from 'lucide-svelte';
  import { configStore } from './stores/configStore';

  $: files = $fileStore.files;
  $: activeFileId = $fileStore.activeFileId;
  
  let searchQuery = '';
  
  $: filteredFiles = searchQuery 
    ? files.filter(file => 
        file.name.toLowerCase().includes(searchQuery.toLowerCase()))
    : files;
</script>

<div class="flex flex-col w-full h-full preset-glass">
  <div class="flex items-center px-2 py-2 z-10 preset-filled-surface-100-900 shadow-xl">
    <div class="relative w-full">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Search files..."
        class="preset-outlined-primary-500-500 w-full h-8 text-sm px-8 rounded-xl focus:outline-none"
      />
      <Search size={14} class="absolute left-2 top-1/2 -translate-y-1/2 opacity-70 text-primary-300" />
    </div>
  </div>
  
  <div class="flex-1 overflow-y-auto min-h-0">
    <div class="flex flex-col gap-1 w-full pb-12">
      {#each filteredFiles as file, index (file.id)}
        <TabFile 
          {file} 
          {index}
          isActive={file.id === activeFileId}
          totalFiles={files.length}
        />
      {/each}
    </div>
  </div>
</div>
