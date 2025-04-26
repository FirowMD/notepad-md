<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import EasyMonacoEditor from '@cloudparker/easy-monaco-editor-svelte';
  import { writable } from 'svelte/store';
  import { editorStore } from './stores/editor';
  import { fileStore } from './stores/files';

  const rawText = writable('');

  let editorRef: HTMLDivElement;
  let containerRef: HTMLDivElement;
  let editor: any;

  $: {
    const activeFile = $fileStore.files.find(f => f.id === $fileStore.activeFileId);
    if (activeFile) {
      $rawText = activeFile.content;
      if (editor) {
        const currentValue = editor.getValue();
        if (currentValue !== activeFile.content) {
          editor.setValue(activeFile.content);
        }
        // Update editor language to match file's language
        editorStore.setLanguage(activeFile.language);
        editor.getModel().setLanguage(activeFile.language);
      }
    } else {
      $rawText = '';
      if (editor) {
        editor.setValue('');
        editor.getModel().setLanguage('plaintext');
      }
    }
  }

  $: if (editor) {
    editor.updateOptions({
      wordWrap: $editorStore.wordWrap ? 'on' : 'off',
      renderWhitespace: $editorStore.showInvisibles ? 'all' : 'none',
      fontSize: $editorStore.fontSize
    });
  }
  
  const handleMonaco = (monaco: any) => {
    if (monaco && editorRef) {
      editor = monaco.editor.create(editorRef, {
        value: $rawText,
        language: 'markdown',
        theme: 'vs-dark',
        fontSize: $editorStore.fontSize,
        wordWrap: $editorStore.wordWrap ? 'on' : 'off',
        renderWhitespace: $editorStore.showInvisibles ? 'all' : 'none',
        minimap: {
          enabled: false
        }
      });

      editor.getModel().onDidChangeContent(() => {
        const value = editor.getValue();
        $rawText = value;
        
        if ($fileStore.activeFileId) {
          fileStore.updateFile($fileStore.activeFileId, {
            content: value,
            modified: new Date()
          });
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
      });

      editor.getModel().onDidChangeLanguage((e: any) => {
        editorStore.setLanguage(e.newLanguage);
      });

      rawText.subscribe((newValue) => {
        if (editor && editor.getValue() !== newValue) {
          editor.setValue(newValue);
        }
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

  <div class="flex h-6 w-full px-2 preset-filled-primary-500 items-center text-xs sticky bottom-0">
    <div class="flex text-left gap-4 sticky left-0 w-full shrink-0 bg-primary-500 z-10">
      <span>{$editorStore.language}</span>
      <span>|</span>
      <span>Length: {$editorStore.stats.length}</span>
      <span>|</span>
      <span>Lines: {$editorStore.stats.lines}</span>
    </div>
    <div class="flex-1"></div>
    <div class="flex text-right gap-4 sticky right-4 shrink-0 bg-primary-500 z-10">
      <span>Ln {$editorStore.cursor.line}, Col {$editorStore.cursor.column}</span>
      <span>|</span>
      <span>{$editorStore.lineEnding}</span>
      <span>|</span>
      <span>{$editorStore.encoding}</span>
    </div>
  </div>
</div>
