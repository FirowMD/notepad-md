<script lang="ts">
  import { fileStore } from './stores/files';

  $: activeFile = $fileStore.activeFileId 
    ? $fileStore.files.find(f => f.id === $fileStore.activeFileId) 
    : null;

  function getFileName(file: any): string {
    if (file.path) {
      const pathParts = file.path.split(/[/\\]/);
      return pathParts[pathParts.length - 1];
    }
    return file.name;
  }
</script>

<div class="flex h-6 w-full px-2 preset-filled-tertiary-500 items-center text-xs">
  <div class="flex text-left gap-2 sticky left-0 w-full shrink-0 bg-tertiary-500 z-10">
    {#if activeFile}
      <span>{getFileName(activeFile)}</span>
      {#if activeFile.isModified}
        <span>•</span>
      {/if}
    {:else}
      <span>No file open</span>
    {/if}
  </div>
</div>