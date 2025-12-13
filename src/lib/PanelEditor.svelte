<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import EasyMonacoEditor from '@cloudparker/easy-monaco-editor-svelte';
  import { writable } from 'svelte/store';
  import { editorStore } from './stores/editor';
  import { fileStore } from './stores/files';
  import { monacoThemeStore } from './stores/monacoTheme';
  import { configStore } from './stores/configStore';

  const rawText = writable('');

  let editorRef: HTMLDivElement;
  let containerRef: HTMLDivElement;
  let editor: any;

  $: monacoTheme = $monacoThemeStore;


  let previousActiveFileId: string | null = null;
  let isSystemChange = false;
  let editorInitialized = false;
  let lastContentHash = '';

  function hashContent(content: string): string {
    let hash = 0;
    for (let i = 0; i < content.length; i++) {
      const char = content.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash;
    }
    return hash.toString();
  }

  $: {
    const activeFile = $fileStore.files.find(f => f.id === $fileStore.activeFileId);
    if (activeFile && editor && editorInitialized) {
      const currentValue = editor.getValue();
      
      if (currentValue !== activeFile.content) {
        isSystemChange = true;
        editor.setValue(activeFile.content);
        lastContentHash = hashContent(activeFile.content);
        
        setTimeout(() => {
          isSystemChange = false;
        }, 10);
      }
      
      editorStore.setLanguage(activeFile.language);
      editor.getModel().setLanguage(activeFile.language);
      
      if (previousActiveFileId !== $fileStore.activeFileId) {
        editor.setPosition({
          lineNumber: activeFile.cursor.line,
          column: activeFile.cursor.column
        });
        editor.revealPosition({
          lineNumber: activeFile.cursor.line,
          column: activeFile.cursor.column
        });
        previousActiveFileId = $fileStore.activeFileId;
      }
    } else if (!activeFile && editor && editorInitialized) {
      isSystemChange = true;
      editor.setValue('');
      editor.getModel().setLanguage('plaintext');
      lastContentHash = '';
      setTimeout(() => {
        isSystemChange = false;
      }, 10);
    }
    
    if (activeFile) {
      $rawText = activeFile.content;
    } else {
      $rawText = '';
    }
  }

  $: if (editor && editorInitialized) {
    editor.updateOptions({
      wordWrap: $editorStore.wordWrap ? 'on' : 'off',
      renderWhitespace: $editorStore.showInvisibles ? 'all' : 'none',
      fontSize: $editorStore.fontSize
    });
  }
  
  const handleMonaco = async (monaco: any) => {
    if (monaco && editorRef) {
      // Store Monaco instance in the theme store
      monacoThemeStore.setMonaco(monaco);
      
      // Get the current theme from config or store
      const config = await configStore.load();
      const currentTheme = config?.monaco_editor_theme || $monacoThemeStore || 'vs-dark';
      
      // Update the store to match config
      monacoThemeStore.set(currentTheme);
      
      // Set the theme if it's a custom theme
      if (currentTheme && currentTheme !== 'vs' && currentTheme !== 'vs-dark' && currentTheme !== 'hc-black') {
        await monacoThemeStore.setTheme(currentTheme);
      }
      
      editor = monaco.editor.create(editorRef, {
        value: '',
        language: 'markdown',
        theme: currentTheme || 'vs-dark',
        fontSize: $editorStore.fontSize,
        wordWrap: $editorStore.wordWrap ? 'on' : 'off',
        renderWhitespace: $editorStore.showInvisibles ? 'all' : 'none',
        minimap: {
          enabled: false
        }
      });

      const activeFile = $fileStore.files.find(f => f.id === $fileStore.activeFileId);
      if (activeFile) {
        isSystemChange = true;
        editor.setValue(activeFile.content);
        lastContentHash = hashContent(activeFile.content);
        setTimeout(() => {
          isSystemChange = false;
        }, 10);
      }

      editorInitialized = true;

      editor.getModel().onDidChangeContent(() => {
        const value = editor.getValue();
        const currentHash = hashContent(value);
        
        $rawText = value;
        
        if ($fileStore.activeFileId) {
          fileStore.updateFile($fileStore.activeFileId, {
            content: value,
            modified: new Date()
          });
          
          if (!isSystemChange && currentHash !== lastContentHash) {
            const activeFile = $fileStore.files.find(f => f.id === $fileStore.activeFileId);
            if (activeFile) {
              if (!activeFile.path && value.trim() !== '') {
                fileStore.markAsModified($fileStore.activeFileId);
              }
              else if (activeFile.path) {
                fileStore.markAsModified($fileStore.activeFileId);
              }
            }
            lastContentHash = currentHash;
          }
        }
        
        const hasCarriageReturn = value.includes('\r');
        const hasLineFeed = value.includes('\n');
        
        if (hasCarriageReturn && hasLineFeed) {
          editorStore.setLineEnding('CRLF');
        } else if (hasLineFeed) {
          editorStore.setLineEnding('LF');
        } else if (hasCarriageReturn) {
          editorStore.setLineEnding('CR');
        }
        
        const lines = editor.getModel().getLineCount();
        const length = value.length;
        editorStore.setStats(length, lines);
      });

      editor.onDidChangeCursorPosition((e: any) => {
        editorStore.setCursor(e.position.lineNumber, e.position.column);

        if ($fileStore.activeFileId) {
          fileStore.updateFile($fileStore.activeFileId, {
            cursor: {
              line: e.position.lineNumber,
              column: e.position.column
            }
          });
        }
      });

      editor.getModel().onDidChangeLanguage((e: any) => {
        editorStore.setLanguage(e.newLanguage);
      });

      const resizeObserver = new ResizeObserver(() => {
        editor.layout();
      });

      resizeObserver.observe(containerRef);

      return () => {
        resizeObserver.disconnect();
      };
    }
  }

  $: {
    () => {
      editor && editor.dispose();
    }
  }
</script>

<div class="w-full h-full flex flex-col">
  <div class="w-full h-full relative" bind:this={containerRef}>
    <EasyMonacoEditor onLoad={handleMonaco}>
      <div class="h-full w-full absolute inset-0" bind:this={editorRef}></div>
    </EasyMonacoEditor>
  </div>

  <div class="flex h-6 w-full px-2 items-center text-xs sticky bottom-0 shadow-xl">
    <div class="flex text-left gap-4 truncate sticky left-0 w-full shrink-0 z-10">
      <span>{$editorStore.language}</span>
      <span>|</span>
      <span>Length: {$editorStore.stats.length}</span>
      <span>|</span>
      <span>Lines: {$editorStore.stats.lines}</span>
    </div>
    <div class="flex-1"></div>
    <div class="flex text-right gap-4 sticky right-4 shrink-0 z-10">
      <span>Ln {$editorStore.cursor.line}, Col {$editorStore.cursor.column}</span>
      <span>|</span>
      <span>{$editorStore.lineEnding}</span>
      <span>|</span>
      <span>{$editorStore.encoding}</span>
    </div>
  </div>
</div>
