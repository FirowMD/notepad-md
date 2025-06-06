<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { FilePlus, FolderOpen, Save, WrapText, Eye, Palette, Code, RotateCcw, Info } from "lucide-svelte";
  import { editorStore } from './stores/editor';
  import { themeStore } from './stores/theme';
  import { fileStore } from './stores/files';
  import { availableLanguages, getLanguageFromExtension } from './stores/language';
  import { open, save, message } from '@tauri-apps/plugin-dialog';
  import { onMount } from 'svelte';

  $: wordWrap = $editorStore.wordWrap;
  $: showInvisibles = $editorStore.showInvisibles;
  $: language = $editorStore.language;
  $: fontSize = $editorStore.fontSize;

  let isFontSizeMenuOpen = false;
  const fontSizes = [8, 9, 10, 11, 12, 13, 14, 15, 16, 18, 20, 22, 24, 26, 28, 30, 32];

  function handleFontSizeChange(size: number) {
    editorStore.setFontSize(size);
    isFontSizeMenuOpen = false;
  }

  let isLanguageMenuOpen = false;

  function handleLanguageChange(lang: string) {
    editorStore.setLanguage(lang);
    if ($fileStore.activeFileId) {
      fileStore.updateFile($fileStore.activeFileId, {
        language: lang
      });
    }
    isLanguageMenuOpen = false;
  }

  const themes = [
    'NotepadMD', 'catppuccin', 'cerberus', 'concord', 'crimson',
    'fennec', 'hamlindigo', 'legacy', 'mint', 'modern', 'mona',
    'nosh', 'nouveau', 'pine', 'reign', 'rocket', 'rose', 'sahara',
    'seafoam', 'terminus', 'vintage', 'vox', 'wintry'
  ] as const;

  let isThemeMenuOpen = false;

  function handleNewFile() {
    const nextId = ($fileStore.files.length + 1).toString();
    const fileInfo = {
      id: nextId,
      path: '',
      name: 'untitled.txt',
      content: '',
      encoding: 'utf-8',
      language: 'plaintext',
      created: new Date(),
      modified: new Date(),
      cursor: {
        line: 1,
        column: 1
      },
      stats: {
        lines: 1,
        length: 0
      }
    };

    fileStore.addFile(fileInfo);
  }

  async function handleOpenFile() {
    try {
      const selected = await open({
        multiple: true,
      });
      
      if (selected) {
        const files = Array.isArray(selected) ? selected : [selected];
        
        for (const filePath of files) {
          const content = await invoke('read_file', { 
            path: filePath,
            encoding: $editorStore.encoding || 'utf-8'
          });
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
          
          if (filePath) {
            try {
              await invoke('watch_file', { path: filePath });
            } catch (error) {
              console.error('Error setting up file watch:', error);
            }
          }
        }
      }
    } catch (err) {
      console.error("Error opening file:", err);
    }
  }

  async function handleSaveFile() {
    const activeFile = $fileStore.files.find(f => f.id === $fileStore.activeFileId);
    if (!activeFile) return;

    try {
      let savePath: string | null = activeFile.path;
      
      if (!savePath) {
        savePath = await save({});

        if (!savePath) return;
      }
      
      await invoke('save_file', { 
        path: savePath,
        content: activeFile.content
      });
      
      if (savePath !== activeFile.path) {
        const pathParts = savePath.split(/[/\\]/);
        const fileName = pathParts[pathParts.length - 1];
        
        fileStore.updateFile(activeFile.id, {
          path: savePath,
          name: fileName,
          modified: new Date()
        });
      }
    } catch (err) {
      console.error("Error saving file:", err);
    }
  }

  async function handleRestoreFile() {
    await fileStore.restoreRecentFile();
  }

  async function handleAbout() {
    await message('NotepadMD v.0.1.2', 'About');
  }

  function handleKeydown(event: KeyboardEvent) {
    // Prevent default behavior for these shortcuts
    if ((event.ctrlKey || event.metaKey) && !event.shiftKey && event.key === 'n') {
      event.preventDefault();
      handleNewFile();
    } else if ((event.ctrlKey || event.metaKey) && !event.shiftKey && event.key === 'o') {
      event.preventDefault();
      handleOpenFile();
    } else if ((event.ctrlKey || event.metaKey) && !event.shiftKey && event.key === 's') {
      event.preventDefault();
      handleSaveFile();
    } else if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'T') {
      event.preventDefault();
      handleRestoreFile();
    } else if (event.altKey && !event.ctrlKey && !event.shiftKey && event.key === 'z') {
      event.preventDefault();
      editorStore.setWordWrap(!wordWrap);
    }
  }

  const encodings = [
    'utf-8',
    'utf-16le',
    'utf-16be',
    'windows-1252'
  ];

  let isEncodingMenuOpen = false;

  async function handleEncodingChange(encoding: string) {
    if ($fileStore.activeFileId) {
      const activeFile = $fileStore.files.find(f => f.id === $fileStore.activeFileId);
      if (activeFile && activeFile.path) {
        try {
          const content = await invoke('read_file', { 
            path: activeFile.path,
            encoding: encoding
          });
          
          fileStore.updateFile(activeFile.id, {
            content: content as string,
            encoding: encoding
          });
          
          editorStore.setEncoding(encoding);
          
        } catch (error) {
          console.error('Error changing file encoding:', error);
        }
      }
    } else {
      // If no file is open, just update the default encoding
      editorStore.setEncoding(encoding);
    }
    isEncodingMenuOpen = false;
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

<div class="flex flex-row w-full min-h-[40px] max-h-[40px] bg-surface-800 items-center px-2 gap-2">
  <button 
    type="button" 
    class="btn btn-sm h-8 flex items-center preset-filled-surface-500 rounded-none"
    on:click={handleNewFile}
    title="New (Ctrl+N)"
  >
    <FilePlus size={16} />
  </button>
  <button 
    type="button" 
    class="btn btn-sm h-8 flex items-center preset-filled-surface-500 rounded-none"
    on:click={handleOpenFile}
    title="Open (Ctrl+O)"
  >
    <FolderOpen size={16} />
  </button>
  <button 
    type="button" 
    class="btn btn-sm h-8 flex items-center preset-filled-surface-500 rounded-none"
    on:click={handleSaveFile}
    title="Save (Ctrl+S)"
  >
    <Save size={16} />
  </button>
  <div class="w-px h-6 bg-surface-700 mx-1"></div>
  <button 
    type="button" 
    class="btn btn-sm h-8 flex items-center preset-filled-surface-500 rounded-none"
    on:click={handleRestoreFile}
    title="Restore (Ctrl+Shift+T)"
  >
    <RotateCcw size={16} />
  </button>
  <button 
    type="button" 
    class="btn btn-sm h-8 flex items-center {wordWrap ? 'preset-tonal-surface' : 'preset-filled-surface-500'} rounded-none"
    on:click={() => editorStore.setWordWrap(!wordWrap)}
    title="Word Wrap (Alt+Z)"
  >
    <WrapText size={16} />
  </button>
  <button 
    type="button" 
    class="btn btn-sm h-8 flex items-center {showInvisibles ? 'preset-tonal-surface' : 'preset-filled-surface-500'} rounded-none"
    on:click={() => editorStore.setShowInvisibles(!showInvisibles)}
    title="Show Space Characters"
  >
    <Eye size={16} />
  </button>
  <div class="w-px h-6 bg-surface-700 mx-1"></div>
  <div class="relative">
    <button 
      type="button" 
      class="btn btn-sm h-8 flex items-center {isThemeMenuOpen ? 'preset-tonal-surface' : 'preset-filled-surface-500'} rounded-none"
      on:click={() => isThemeMenuOpen = !isThemeMenuOpen}
      title="Theme"
    >
      <Palette size={16} />
    </button>
    {#if isThemeMenuOpen}
      <div 
        role="menu"
        tabindex="0"
        class="absolute left-0 top-full mt-1 w-48 bg-surface-700 shadow-xl z-50 max-h-64 overflow-y-auto"
        on:mouseleave={() => isThemeMenuOpen = false}
      >
        {#each themes as theme}
          <button
            role="menuitem"
            class="text-xs w-full px-3 py-1.5 text-left hover:bg-surface-600 transition-colors capitalize"
            class:bg-surface-500={$themeStore === theme}
            on:click={() => {
              themeStore.setTheme(theme);
              isThemeMenuOpen = false;
            }}
          >
            {theme}
          </button>
        {/each}
      </div>
    {/if}
  </div>
  <div class="relative">
    <button 
      type="button" 
      class="btn btn-sm h-8 flex items-center gap-2 {isLanguageMenuOpen ? 'preset-tonal-surface' : 'preset-filled-surface-500'} rounded-none"
      on:click={() => isLanguageMenuOpen = !isLanguageMenuOpen}
      title="Language"
    >
      <Code size={16} />
      <span class="text-xs capitalize">{language}</span>
    </button>
    {#if isLanguageMenuOpen}
      <div 
        role="menu"
        tabindex="0"
        class="absolute left-0 top-full mt-1 w-48 bg-surface-700 shadow-xl z-50 max-h-64 overflow-y-auto"
        on:mouseleave={() => isLanguageMenuOpen = false}
      >
        {#each availableLanguages as lang}
          <button
            role="menuitem"
            class="text-xs w-full px-3 py-1.5 text-left hover:bg-surface-600 transition-colors capitalize"
            class:bg-surface-500={language === lang}
            on:click={() => handleLanguageChange(lang)}
          >
            {lang}
          </button>
        {/each}
      </div>
    {/if}
  </div>
  <div class="relative">
    <button 
      type="button" 
      class="btn btn-sm h-8 flex items-center gap-2 {isEncodingMenuOpen ? 'preset-tonal-surface' : 'preset-filled-surface-500'} rounded-none"
      on:click={() => isEncodingMenuOpen = !isEncodingMenuOpen}
      title="Encoding"
    >
      <span class="text-xs uppercase">{$editorStore.encoding || 'UTF-8'}</span>
    </button>
    {#if isEncodingMenuOpen}
      <div 
        role="menu"
        tabindex="0"
        class="absolute left-0 top-full mt-1 w-48 bg-surface-700 shadow-xl z-50 max-h-64 overflow-y-auto"
        on:mouseleave={() => isEncodingMenuOpen = false}
      >
        {#each encodings as encoding}
          <button
            role="menuitem"
            class="text-xs w-full px-3 py-1.5 text-left hover:bg-surface-600 transition-colors uppercase"
            class:bg-surface-500={$editorStore.encoding === encoding}
            on:click={() => handleEncodingChange(encoding)}
          >
            {encoding}
          </button>
        {/each}
      </div>
    {/if}
  </div>
  <div class="relative">
    <button 
      type="button" 
      class="btn btn-sm h-8 flex items-center gap-2 {isFontSizeMenuOpen ? 'preset-tonal-surface' : 'preset-filled-surface-500'} rounded-none"
      on:click={() => isFontSizeMenuOpen = !isFontSizeMenuOpen}
      title="Font Size"
    >
      <span class="text-xs">{fontSize}px</span>
    </button>
    {#if isFontSizeMenuOpen}
      <div 
        role="menu"
        tabindex="0"
        class="absolute left-0 top-full mt-1 w-24 bg-surface-700 shadow-xl z-50 max-h-64 overflow-y-auto"
        on:mouseleave={() => isFontSizeMenuOpen = false}
      >
        {#each fontSizes as size}
          <button
            role="menuitem"
            class="text-xs w-full px-3 py-1.5 text-left hover:bg-surface-600 transition-colors"
            class:bg-surface-500={fontSize === size}
            on:click={() => handleFontSizeChange(size)}
          >
            {size}px
          </button>
        {/each}
      </div>
    {/if}
  </div>
  <div class="w-px h-6 bg-surface-700 mx-1"></div>
  <button
    type="button"
    class="btn btn-sm h-8 flex items-center preset-filled-surface-500 rounded-none"
    on:click={handleAbout}
    title="About"
  >
    <Info size={16} />
  </button>
</div>
