<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { FilePlus, FolderOpen, Save, WrapText, Eye, Palette, Code, RotateCcw, Info, PanelLeftClose, PanelLeft, FileCode, Clock, Droplet } from "lucide-svelte";
  import { editorStore } from './stores/editor';
  import { themeStore } from './stores/theme';
  import { fileStore } from './stores/files';
  import { notificationStore } from './stores/notification';
  import { sidePanelStore } from './stores/sidePanelStore';
  import { monacoThemeStore } from './stores/monacoTheme';
  import { availableLanguages, getLanguageFromExtension } from './stores/language';
  import { configStore } from './stores/configStore';
  import { open, save, message, ask } from '@tauri-apps/plugin-dialog';
  import { onMount } from 'svelte';

  $: wordWrap = $editorStore.wordWrap;
  $: showInvisibles = $editorStore.showInvisibles;
  $: language = $editorStore.language;
  $: fontSize = $editorStore.fontSize;
  $: isSidePanelVisible = $sidePanelStore;
  $: monacoTheme = $monacoThemeStore;
  $: recentFiles = ($configStore.recent_files || []).slice(0, 10);

  let isFontSizeMenuOpen = false;
  const fontSizes = [8, 9, 10, 11, 12, 13, 14, 15, 16, 18, 20, 22, 24, 26, 28, 30, 32];
  let isRecentFilesMenuOpen = false;
  let selectedRecentIndex = 0;
  let recentFilesMenu: HTMLDivElement;

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
  let isMonacoThemeMenuOpen = false;
  let availableMonacoThemes: string[] = ['vs', 'vs-dark', 'hc-black'];

  function toggleTransparentMode() {
    const next = !$configStore.transparent_mode;
    configStore.save({ transparent_mode: next });
  }

  function handleNewFile() {
    fileStore.addUntitledFile();
  }

  async function handleCloseActiveFile() {
    const activeFileId = $fileStore.activeFileId;
    if (activeFileId) {
      const activeFile = $fileStore.files.find(f => f.id === activeFileId);
      if (activeFile?.path) {
        try {
          await invoke('unwatch_file', { path: activeFile.path });
        } catch (error) {
          console.error('Error unwatching file:', error);
        }
      }
      fileStore.removeFile(activeFileId);
    }
  }

  async function handleOpenFile() {
    try {
      const selected = await open({
        multiple: true,
      });
      
      if (selected) {
        const files = Array.isArray(selected) ? selected : [selected];
        
        for (const filePath of files) {
          const fileData = await invoke('read_file', { 
            path: filePath,
            encoding: $editorStore.encoding || 'utf-8'
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
      const errorStr = String(err);
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
            const files = $fileStore.files.map(f => f.path).filter(p => p);
            await invoke('relaunch_as_admin', { args: files });
          }
        } else {
          notificationStore.show('Permission denied even with admin privileges.', 'error');
        }
      } else {
        notificationStore.show("Error opening file", "error");
      }
    }
  }

  function openRecentFilesMenu() {
    isRecentFilesMenuOpen = true;
    selectedRecentIndex = 0;
    setTimeout(() => {
      if (recentFilesMenu) {
        recentFilesMenu.focus();
      }
    }, 0);
  }

  function handleRecentMenuKeydown(event: KeyboardEvent) {
    if (!isRecentFilesMenuOpen || recentFiles.length === 0) return;

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        selectedRecentIndex = (selectedRecentIndex + 1) % recentFiles.length;
        scrollToSelectedItem();
        break;
      case 'ArrowUp':
        event.preventDefault();
        selectedRecentIndex = (selectedRecentIndex - 1 + recentFiles.length) % recentFiles.length;
        scrollToSelectedItem();
        break;
      case 'Enter':
        event.preventDefault();
        if (recentFiles[selectedRecentIndex]) {
          handleOpenRecentFile(recentFiles[selectedRecentIndex]);
        }
        break;
      case 'Escape':
        event.preventDefault();
        isRecentFilesMenuOpen = false;
        break;
    }
  }

  function scrollToSelectedItem() {
    setTimeout(() => {
      if (recentFilesMenu) {
        const selectedElement = recentFilesMenu.querySelector(`[data-index="${selectedRecentIndex}"]`);
        if (selectedElement) {
          selectedElement.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
        }
      }
    }, 0);
  }

  async function handleOpenRecentFile(filePath: string) {
    try {
      const fileData = await invoke('read_file', { 
        path: filePath,
        encoding: $editorStore.encoding || 'utf-8'
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
      
      if (filePath) {
        try {
          await invoke('watch_file', { path: filePath });
        } catch (error) {
          console.error('Error setting up file watch:', error);
        }
      }
      
      isRecentFilesMenuOpen = false;
    } catch (err) {
      console.error("Error opening recent file:", err);
      const errorStr = String(err);
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
      } else if (errorStr.includes('No such file')) {
        notificationStore.show('File not found. It may have been moved or deleted.', 'error');
        // Remove from recent files if it doesn't exist
        const updatedRecent = recentFiles.filter(f => f !== filePath);
        configStore.save({ recent_files: updatedRecent });
      } else {
        notificationStore.show("Error opening file", "error");
      }
    }
  }

  async function handleSaveFile() {
    const activeFile = $fileStore.files.find(f => f.id === $fileStore.activeFileId);
    if (!activeFile) return;

    try {
      let savePath: string | null = activeFile.path;
      
      if (!savePath) {
        const suggestedName = activeFile.name && activeFile.name !== 'Untitled' ? activeFile.name : 'untitled.txt';
        savePath = await save({
          defaultPath: suggestedName
        });

        if (!savePath) return;
      }
      
      await invoke('save_file', { 
        path: savePath,
        content: activeFile.content
      });
      
      const savedHash = await invoke('calculate_file_hash_command', { content: activeFile.content }) as string;
      
      let fileSystemModified: Date | undefined;
      try {
        const modifiedTimestamp = await invoke('get_file_metadata', { path: savePath }) as number;
        fileSystemModified = new Date(modifiedTimestamp * 1000);
      } catch (error) {
        console.error('Error getting file metadata after save:', error);
      }
      
      if (savePath !== activeFile.path) {
        const pathParts = savePath.split(/[/\\]/);
        const fileName = pathParts[pathParts.length - 1];
        
        fileStore.updateFile(activeFile.id, {
          path: savePath,
          name: fileName,
          hash: savedHash,
          modified: new Date(),
          fileSystemModified
        });
      } else {
        fileStore.updateFile(activeFile.id, {
          hash: savedHash,
          modified: new Date(),
          fileSystemModified
        });
      }
      
      fileStore.markAsSaved(activeFile.id);
      notificationStore.show("File saved successfully", "success", 2500);
      
    } catch (err) {
      console.error("Error saving file:", err);
      const errorStr = String(err);
      if (errorStr.includes('PERMISSION_DENIED')) {
        const isAdmin = await invoke('check_admin_privileges') as boolean;
        if (!isAdmin) {
          const shouldRelaunch = await ask(
            `Failed to save file due to insufficient permissions.\n\nWould you like to restart the application with administrator privileges?`,
            { title: 'Permission Denied', kind: 'warning' }
          );
          if (shouldRelaunch) {
            const files = $fileStore.files.map(f => f.path).filter(p => p);
            await invoke('relaunch_as_admin', { args: files });
          }
        } else {
          notificationStore.show('Permission denied even with admin privileges.', 'error');
        }
      } else {
        notificationStore.show("Error saving file", "error");
      }
    }
  }

  async function handleRestoreFile() {
    await fileStore.restoreRecentFile();
  }

  async function handleAbout() {
    await message('NotepadMD v.0.2.1', 'About');
  }


  $: windowTitle = (() => {
    const activeFile = $fileStore.files.find(f => f.id === $fileStore.activeFileId);
    if (activeFile) {
      const fileName = activeFile.name || 'Untitled';
      const modifiedIndicator = activeFile.isModified ? ' •' : '';
      return `${fileName}${modifiedIndicator}`;
    }
    return 'NotepadMD';
  })();


  function handleKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && !event.shiftKey && event.code === 'KeyN') {
      event.preventDefault();
      handleNewFile();
    } else if ((event.ctrlKey || event.metaKey) && !event.shiftKey && event.code === 'KeyO') {
      event.preventDefault();
      handleOpenFile();
    } else if ((event.ctrlKey || event.metaKey) && !event.shiftKey && event.code === 'KeyS') {
      event.preventDefault();
      handleSaveFile();
    } else if ((event.ctrlKey || event.metaKey) && !event.shiftKey && event.code === 'KeyR') {
      event.preventDefault();
      openRecentFilesMenu();
    } else if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.code === 'KeyT') {
      event.preventDefault();
      handleRestoreFile();
    } else if (event.altKey && !event.ctrlKey && !event.shiftKey && event.code === 'KeyZ') {
      event.preventDefault();
      editorStore.setWordWrap(!wordWrap);
    } else if ((event.ctrlKey || event.metaKey) && !event.shiftKey && event.code === 'KeyW') {
      event.preventDefault();
      handleCloseActiveFile();
    } else if ((event.ctrlKey || event.metaKey) && !event.shiftKey && event.code === 'KeyB') {
      event.preventDefault();
      sidePanelStore.toggle();
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
          const fileData = await invoke('read_file', { 
            path: activeFile.path,
            encoding: encoding
          }) as { content: string, hash: string };
          
          fileStore.updateFile(activeFile.id, {
            content: fileData.content,
            encoding: encoding,
            hash: fileData.hash
          });
          
          editorStore.setEncoding(encoding);
          
        } catch (error) {
          console.error('Error changing file encoding:', error);
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
          } else {
            notificationStore.show("Error changing file encoding", "error");
          }
        }
      }
    } else {
      editorStore.setEncoding(encoding);
    }
    isEncodingMenuOpen = false;
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    
    // Load available Monaco themes
    monacoThemeStore.getAvailableThemes().then(themes => {
      availableMonacoThemes = themes;
    }).catch(error => {
      console.error('Error loading Monaco themes:', error);
    });
    
    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

<div class="flex flex-col w-full">
  <div class="flex flex-row w-full min-h-[22px] max-h-[22px] items-center"
       data-tauri-drag-region
       class:bg-primary-900={!$configStore.transparent_mode}
       class:bg-transparent={$configStore.transparent_mode}>
    <div class="flex-1 px-3 text-sm font-medium text-surface-200 select-none">
      {windowTitle}
    </div>
  </div>
  
  <div class="flex flex-row w-full min-h-[36px] max-h-[36px] items-center px-2 gap-2"
       class:bg-surface-800={!$configStore.transparent_mode}
       class:bg-transparent={$configStore.transparent_mode}>
  <button 
    type="button" 
    class="btn btn-sm h-7 flex items-center preset-filled-surface-500"
    onclick={handleNewFile}
    title="New (Ctrl+N)"
  >
    <FilePlus size={14} />
  </button>
  <button 
    type="button" 
    class="btn btn-sm h-7 flex items-center preset-filled-surface-500"
    onclick={handleOpenFile}
    title="Open (Ctrl+O)"
  >
    <FolderOpen size={14} />
  </button>
  <button 
    type="button" 
    class="btn btn-sm h-7 flex items-center preset-filled-surface-500"
    onclick={handleSaveFile}
    title="Save (Ctrl+S)"
  >
    <Save size={14} />
  </button>
  <div class="relative">
    <button 
      type="button" 
      class="btn btn-sm h-7 flex items-center {isRecentFilesMenuOpen ? 'preset-tonal-surface' : 'preset-filled-surface-500'}"
      onclick={openRecentFilesMenu}
      title="Recent (Ctrl+R)"
    >
      <Clock size={14} />
    </button>
    {#if isRecentFilesMenuOpen}
      <div 
        bind:this={recentFilesMenu}
        role="menu"
        tabindex="-1"
        class="absolute left-0 top-full mt-1 w-96 bg-surface-700 shadow-xl z-50 max-h-64 overflow-y-auto focus:outline-none"
        onmouseleave={() => isRecentFilesMenuOpen = false}
        onkeydown={handleRecentMenuKeydown}
      >
        {#if recentFiles.length > 0}
          {#each recentFiles as filePath, index}
            {@const fileName = filePath.split(/[/\\]/).pop() || filePath}
            <button
              role="menuitem"
              data-index={index}
              class="text-xs w-full px-3 py-1.5 text-left transition-colors flex flex-col"
              class:bg-surface-500={index === selectedRecentIndex}
              class:hover:bg-surface-600={index !== selectedRecentIndex}
              onclick={() => handleOpenRecentFile(filePath)}
              onmouseenter={() => selectedRecentIndex = index}
            >
              <span class="font-medium">{fileName}</span>
              <span class="text-surface-400 text-[10px] truncate">{filePath}</span>
            </button>
          {/each}
        {:else}
          <div class="text-xs px-3 py-2 text-surface-400">
            No recent files
          </div>
        {/if}
      </div>
    {/if}
  </div>
  <div class="w-px h-6 bg-surface-700 mx-1"></div>
  <button 
    type="button" 
    class="btn btn-sm h-7 flex items-center preset-filled-surface-500"
    onclick={handleRestoreFile}
    title="Restore (Ctrl+Shift+T)"
  >
    <RotateCcw size={14} />
  </button>
  <button 
    type="button" 
    class="btn btn-sm h-7 flex items-center {wordWrap ? 'preset-tonal-surface' : 'preset-filled-surface-500'}"
    onclick={() => editorStore.setWordWrap(!wordWrap)}
    title="Word Wrap (Alt+Z)"
  >
    <WrapText size={14} />
  </button>
  <button 
    type="button" 
    class="btn btn-sm h-7 flex items-center {showInvisibles ? 'preset-tonal-surface' : 'preset-filled-surface-500'}"
    onclick={() => editorStore.setShowInvisibles(!showInvisibles)}
    title="Show Space Characters"
  >
    <Eye size={14} />
  </button>
  <div class="w-px h-6 bg-surface-700 mx-1"></div>
  <div class="relative">
    <button 
      type="button" 
      class="btn btn-sm h-7 flex items-center {isThemeMenuOpen ? 'preset-tonal-surface' : 'preset-filled-surface-500'}"
      onclick={() => isThemeMenuOpen = !isThemeMenuOpen}
      title="Theme"
    >
      <Palette size={14} />
    </button>
    {#if isThemeMenuOpen}
      <div 
        role="menu"
        tabindex="0"
        class="absolute left-0 top-full mt-1 w-48 bg-surface-700 shadow-xl z-50 max-h-64 overflow-y-auto"
        onmouseleave={() => isThemeMenuOpen = false}
      >
        {#each themes as theme}
          <button
            role="menuitem"
            class="text-xs w-full px-3 py-1.5 text-left hover:bg-surface-600 transition-colors capitalize"
            class:bg-surface-500={$themeStore === theme}
            onclick={() => {
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
  <button 
    type="button" 
    class="btn btn-sm h-7 flex items-center { $configStore.transparent_mode ? 'preset-tonal-surface' : 'preset-filled-surface-500' }"
    onclick={toggleTransparentMode}
    title="Transparent Mode"
  >
    <Droplet size={14} />
  </button>
  <div class="relative">
    <button 
      type="button" 
      class="btn btn-sm h-7 flex items-center {isMonacoThemeMenuOpen ? 'preset-tonal-surface' : 'preset-filled-surface-500'}"
      onclick={() => isMonacoThemeMenuOpen = !isMonacoThemeMenuOpen}
      title="Monaco Editor Theme"
    >
      <FileCode size={14} />
    </button>
    {#if isMonacoThemeMenuOpen}
      <div 
        role="menu"
        tabindex="0"
        class="absolute left-0 top-full mt-1 w-48 bg-surface-700 shadow-xl z-50 max-h-64 overflow-y-auto"
        onmouseleave={() => isMonacoThemeMenuOpen = false}
      >
        {#each availableMonacoThemes as theme}
          <button
            role="menuitem"
            class="text-xs w-full px-3 py-1.5 text-left hover:bg-surface-600 transition-colors"
            class:bg-surface-500={monacoTheme === theme}
            onclick={() => {
              monacoThemeStore.setTheme(theme);
              isMonacoThemeMenuOpen = false;
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
      class="btn btn-sm h-7 flex items-center gap-2 {isLanguageMenuOpen ? 'preset-tonal-surface' : 'preset-filled-surface-500'}"
      onclick={() => isLanguageMenuOpen = !isLanguageMenuOpen}
      title="Language"
    >
      <Code size={14} />
      <span class="text-xs capitalize">{language}</span>
    </button>
    {#if isLanguageMenuOpen}
      <div 
        role="menu"
        tabindex="0"
        class="absolute left-0 top-full mt-1 w-48 bg-surface-700 shadow-xl z-50 max-h-64 overflow-y-auto"
        onmouseleave={() => isLanguageMenuOpen = false}
      >
        {#each availableLanguages as lang}
          <button
            role="menuitem"
            class="text-xs w-full px-3 py-1.5 text-left hover:bg-surface-600 transition-colors capitalize"
            class:bg-surface-500={language === lang}
            onclick={() => handleLanguageChange(lang)}
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
      class="btn btn-sm h-7 flex items-center gap-2 {isEncodingMenuOpen ? 'preset-tonal-surface' : 'preset-filled-surface-500'}"
      onclick={() => isEncodingMenuOpen = !isEncodingMenuOpen}
      title="Encoding"
    >
      <span class="text-xs uppercase">{$editorStore.encoding || 'UTF-8'}</span>
    </button>
    {#if isEncodingMenuOpen}
      <div 
        role="menu"
        tabindex="0"
        class="absolute left-0 top-full mt-1 w-48 bg-surface-700 shadow-xl z-50 max-h-64 overflow-y-auto"
        onmouseleave={() => isEncodingMenuOpen = false}
      >
        {#each encodings as encoding}
          <button
            role="menuitem"
            class="text-xs w-full px-3 py-1.5 text-left hover:bg-surface-600 transition-colors uppercase"
            class:bg-surface-500={$editorStore.encoding === encoding}
            onclick={() => handleEncodingChange(encoding)}
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
      class="btn btn-sm h-7 flex items-center gap-2 {isFontSizeMenuOpen ? 'preset-tonal-surface' : 'preset-filled-surface-500'}"
      onclick={() => isFontSizeMenuOpen = !isFontSizeMenuOpen}
      title="Font Size"
    >
      <span class="text-xs">{fontSize}px</span>
    </button>
    {#if isFontSizeMenuOpen}
      <div 
        role="menu"
        tabindex="0"
        class="absolute left-0 top-full mt-1 w-24 bg-surface-700 shadow-xl z-50 max-h-64 overflow-y-auto"
        onmouseleave={() => isFontSizeMenuOpen = false}
      >
        {#each fontSizes as size}
          <button
            role="menuitem"
            class="text-xs w-full px-3 py-1.5 text-left hover:bg-surface-600 transition-colors"
            class:bg-surface-500={fontSize === size}
            onclick={() => handleFontSizeChange(size)}
          >
            {size}px
          </button>
        {/each}
      </div>
    {/if}
  </div>
  <button 
    type="button" 
    class="btn btn-sm h-7 flex items-center {isSidePanelVisible ? 'preset-tonal-surface' : 'preset-filled-surface-500'}"
    onclick={() => sidePanelStore.toggle()}
    title="Show/Hide Side Panel (Ctrl+B)"
  >
    {#if isSidePanelVisible}
      <PanelLeftClose size={14} />
    {:else}
      <PanelLeft size={14} />
    {/if}
  </button>
  <div class="w-px h-6 bg-surface-700 mx-1"></div>
  <button
    type="button"
    class="btn btn-sm h-7 flex items-center preset-filled-surface-500"
    onclick={handleAbout}
    title="About"
  >
    <Info size={14} />
  </button>
  </div>
</div>
