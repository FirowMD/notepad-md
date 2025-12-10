<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { FilePlus, FolderOpen, Save, WrapText, Eye, Palette, Code, RotateCcw, Info, PanelLeftClose, PanelLeft, FileCode, Clock } from "lucide-svelte";
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
  import { Combobox, Portal, type ComboboxRootProps, useListCollection } from '@skeletonlabs/skeleton-svelte';

  const wordWrap = $derived($editorStore.wordWrap);
  const showInvisibles = $derived($editorStore.showInvisibles);
  const language = $derived($editorStore.language);
  const fontSize = $derived($editorStore.fontSize);
  const isSidePanelVisible = $derived($sidePanelStore);
  const monacoTheme = $derived($monacoThemeStore);
  const recentFiles = $derived(($configStore.recent_files || []).slice(0, 10));

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

  const languageData = availableLanguages.map((lang) => ({ label: lang, value: lang }));
  let languageItems = $state(languageData);
  const languageCollection = $derived(
    useListCollection({
      items: languageItems,
      itemToString: (item) => item.label,
      itemToValue: (item) => item.value,
    })
  );
  const onLanguageOpenChange = () => {
    languageItems = languageData;
  };
  const onLanguageInputValueChange: ComboboxRootProps['onInputValueChange'] = (event) => {
    const q = event.inputValue.toLowerCase();
    const filtered = languageData.filter((l) => l.label.toLowerCase().includes(q));
    languageItems = filtered.length > 0 ? filtered : languageData;
  };
  const onLanguageValueChange: ComboboxRootProps['onValueChange'] = (event) => {
    const valAny = Array.isArray(event.value) ? event.value[0] : event.value;
    const val = valAny as string;
    handleLanguageChange(val);
  };

  const themes = [
    'NotepadMD', 'catppuccin', 'cerberus', 'concord', 'crimson',
    'fennec', 'hamlindigo', 'legacy', 'mint', 'modern', 'mona',
    'nosh', 'nouveau', 'pine', 'reign', 'rocket', 'rose', 'sahara',
    'seafoam', 'terminus', 'vintage', 'vox', 'wintry'
  ] as const;

  let isThemeMenuOpen = false;
  let isMonacoThemeMenuOpen = false;
  let availableMonacoThemes: string[] = ['vs', 'vs-dark', 'hc-black'];

  const themesData = themes.map((theme) => ({ label: theme, value: theme }));
  let themeItems = $state(themesData);
  const themeCollection = $derived(
    useListCollection({
      items: themeItems,
      itemToString: (item) => item.label,
      itemToValue: (item) => item.value,
    })
  );
  const onThemeOpenChange = () => {
    themeItems = themesData;
  };
  const onThemeInputValueChange: ComboboxRootProps['onInputValueChange'] = (event) => {
    const q = event.inputValue.toLowerCase();
    const filtered = themesData.filter((t) => t.label.toLowerCase().includes(q));
    themeItems = filtered.length > 0 ? filtered : themesData;
  };
  const onThemeValueChange: ComboboxRootProps['onValueChange'] = (event) => {
    const valAny = Array.isArray(event.value) ? event.value[0] : event.value;
    const val = valAny as typeof themes[number];
    themeStore.setTheme(val);
  };

  const monacoThemesData = $derived(
    availableMonacoThemes.map((theme) => ({ label: theme, value: theme }))
  );
  let monacoThemeItems = $state(monacoThemesData);
  $effect(() => {
    monacoThemeItems = monacoThemesData;
  });
  const monacoCollection = $derived(
    useListCollection({
      items: monacoThemeItems,
      itemToString: (item) => item.label,
      itemToValue: (item) => item.value,
    })
  );
  const onMonacoOpenChange = () => {
    monacoThemeItems = monacoThemesData;
  };
  const onMonacoInputValueChange: ComboboxRootProps['onInputValueChange'] = (event) => {
    const q = event.inputValue.toLowerCase();
    const filtered = monacoThemesData.filter((t) => t.label.toLowerCase().includes(q));
    monacoThemeItems = filtered.length > 0 ? filtered : monacoThemesData;
  };
  const onMonacoValueChange: ComboboxRootProps['onValueChange'] = (event) => {
    const valAny = Array.isArray(event.value) ? event.value[0] : event.value;
    const val = valAny as string;
    monacoThemeStore.setTheme(val);
  };

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


  const activeFile = $derived($fileStore.files.find(f => f.id === $fileStore.activeFileId));
  const windowTitle = $derived(activeFile ? `${activeFile.name || 'Untitled'}${activeFile.isModified ? ' •' : ''}` : 'NotepadMD');


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

  const encodingData = encodings.map((enc) => ({ label: enc, value: enc }));
  let encodingItems = $state(encodingData);
  const encodingCollection = $derived(
    useListCollection({
      items: encodingItems,
      itemToString: (item) => item.label,
      itemToValue: (item) => item.value,
    })
  );
  const onEncodingOpenChange = () => {
    encodingItems = encodingData;
  };
  const onEncodingInputValueChange: ComboboxRootProps['onInputValueChange'] = (event) => {
    const q = event.inputValue.toLowerCase();
    const filtered = encodingData.filter((e) => e.label.toLowerCase().includes(q));
    encodingItems = filtered.length > 0 ? filtered : encodingData;
  };
  const onEncodingValueChange: ComboboxRootProps['onValueChange'] = (event) => {
    const valAny = Array.isArray(event.value) ? event.value[0] : event.value;
    const val = valAny as string;
    handleEncodingChange(val);
  };

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
  <div class="flex flex-row w-full min-h-[30px] max-h-[30px] bg-surface-900 items-center">
    <div class="flex-1 px-3 text-sm font-medium text-surface-200 select-none">
      {windowTitle}
    </div>
  </div>
  
  <div class="flex flex-row w-full min-h-[40px] max-h-[40px] bg-surface-800 items-center px-2 gap-2">
  <button 
    type="button" 
    class="btn btn-sm h-8 flex items-center preset-filled-surface-500 rounded-none"
    onclick={handleNewFile}
    title="New (Ctrl+N)"
  >
    <FilePlus size={14} />
  </button>
  <button 
    type="button" 
    class="btn btn-sm h-8 flex items-center preset-filled-surface-500 rounded-none"
    onclick={handleOpenFile}
    title="Open (Ctrl+O)"
  >
    <FolderOpen size={14} />
  </button>
  <button 
    type="button" 
    class="btn btn-sm h-8 flex items-center preset-filled-surface-500 rounded-none"
    onclick={handleSaveFile}
    title="Save (Ctrl+S)"
  >
    <Save size={14} />
  </button>
  <div class="relative">
    <button 
      type="button" 
      class="btn btn-sm h-8 flex items-center {isRecentFilesMenuOpen ? 'preset-tonal-surface' : 'preset-filled-surface-500'} rounded-none"
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
    class="btn btn-sm h-8 flex items-center preset-filled-surface-500 rounded-none"
    onclick={handleRestoreFile}
    title="Restore (Ctrl+Shift+T)"
  >
    <RotateCcw size={14} />
  </button>
  <button 
    type="button" 
    class="btn btn-sm h-8 flex items-center {wordWrap ? 'preset-tonal-surface' : 'preset-filled-surface-500'} rounded-none"
    onclick={() => editorStore.setWordWrap(!wordWrap)}
    title="Word Wrap (Alt+Z)"
  >
    <WrapText size={14} />
  </button>
  <button 
    type="button" 
    class="btn btn-sm h-8 flex items-center {showInvisibles ? 'preset-tonal-surface' : 'preset-filled-surface-500'} rounded-none"
    onclick={() => editorStore.setShowInvisibles(!showInvisibles)}
    title="Show Space Characters"
  >
    <Eye size={14} />
  </button>
  <div class="w-px h-6 bg-surface-700 mx-1"></div>
  <div class="min-w-32">
    <Combobox class="w-full" placeholder="Theme..." collection={themeCollection} onOpenChange={onThemeOpenChange} onInputValueChange={onThemeInputValueChange} onValueChange={onThemeValueChange}>
      <Combobox.Label class="sr-only">Theme</Combobox.Label>
      <Combobox.Control>
        <Combobox.Input class="h-8 text-xs" />
        <Combobox.Trigger />
      </Combobox.Control>
      <Portal>
        <Combobox.Positioner>
          <Combobox.Content class="max-h-64 overflow-y-auto">
            {#each themeItems as item (item.value)}
              <Combobox.Item {item}>
                <Combobox.ItemText>{item.label}</Combobox.ItemText>
                <Combobox.ItemIndicator />
              </Combobox.Item>
            {/each}
          </Combobox.Content>
        </Combobox.Positioner>
      </Portal>
    </Combobox>
  </div>
  <div class="w-48">
    <Combobox class="w-full" placeholder="Monaco theme..." collection={monacoCollection} onOpenChange={onMonacoOpenChange} onInputValueChange={onMonacoInputValueChange} onValueChange={onMonacoValueChange}>
      <Combobox.Label class="sr-only">Monaco Editor Theme</Combobox.Label>
      <Combobox.Control>
        <Combobox.Input class="h-8 text-xs" />
        <Combobox.Trigger />
      </Combobox.Control>
      <Portal>
        <Combobox.Positioner>
          <Combobox.Content class="max-h-64 overflow-y-auto">
            {#each monacoThemeItems as item (item.value)}
              <Combobox.Item {item}>
                <Combobox.ItemText>{item.label}</Combobox.ItemText>
                <Combobox.ItemIndicator />
              </Combobox.Item>
            {/each}
          </Combobox.Content>
        </Combobox.Positioner>
      </Portal>
    </Combobox>
  </div>
  <div class="w-48">
    <Combobox class="w-full" placeholder="Language..." collection={languageCollection} onOpenChange={onLanguageOpenChange} onInputValueChange={onLanguageInputValueChange} onValueChange={onLanguageValueChange}>
      <Combobox.Label class="sr-only">Language</Combobox.Label>
      <Combobox.Control>
        <Combobox.Input class="h-8 text-xs" />
        <Combobox.Trigger />
      </Combobox.Control>
      <Portal>
        <Combobox.Positioner>
          <Combobox.Content class="max-h-64 overflow-y-auto">
            {#each languageItems as item (item.value)}
              <Combobox.Item {item}>
                <Combobox.ItemText class="capitalize">{item.label}</Combobox.ItemText>
                <Combobox.ItemIndicator />
              </Combobox.Item>
            {/each}
          </Combobox.Content>
        </Combobox.Positioner>
      </Portal>
    </Combobox>
  </div>
  <div class="w-40">
    <Combobox class="w-full" placeholder="Encoding..." collection={encodingCollection} onOpenChange={onEncodingOpenChange} onInputValueChange={onEncodingInputValueChange} onValueChange={onEncodingValueChange}>
      <Combobox.Label class="sr-only">Encoding</Combobox.Label>
      <Combobox.Control>
        <Combobox.Input class="h-8 text-xs uppercase" />
        <Combobox.Trigger />
      </Combobox.Control>
      <Portal>
        <Combobox.Positioner>
          <Combobox.Content class="max-h-64 overflow-y-auto">
            {#each encodingItems as item (item.value)}
              <Combobox.Item {item}>
                <Combobox.ItemText class="uppercase">{item.label}</Combobox.ItemText>
                <Combobox.ItemIndicator />
              </Combobox.Item>
            {/each}
          </Combobox.Content>
        </Combobox.Positioner>
      </Portal>
    </Combobox>
  </div>
  <div class="relative">
    <button 
      type="button" 
      class="btn btn-sm h-8 flex items-center gap-2 {isFontSizeMenuOpen ? 'preset-tonal-surface' : 'preset-filled-surface-500'} rounded-none"
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
    class="btn btn-sm h-8 flex items-center {isSidePanelVisible ? 'preset-tonal-surface' : 'preset-filled-surface-500'} rounded-none"
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
    class="btn btn-sm h-8 flex items-center preset-filled-surface-500 rounded-none"
    onclick={handleAbout}
    title="About"
  >
    <Info size={14} />
  </button>
  </div>
</div>
