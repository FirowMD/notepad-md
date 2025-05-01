import { writable } from 'svelte/store';
import { configStore } from './configStore';

interface EditorState {
  cursor: {
    line: number;
    column: number;
  };
  stats: {
    length: number;
    lines: number;
  };
  language: string;
  encoding: string;
  lineEnding: 'CRLF' | 'LF' | 'CR';
  wordWrap: boolean;
  showInvisibles: boolean;
  fontSize: number;
}

function createEditorStore() {
  const { subscribe, set, update } = writable<EditorState>({
    cursor: { line: 1, column: 1 },
    stats: { length: 0, lines: 0 },
    language: 'plaintext',
    encoding: 'utf-8',
    lineEnding: 'CRLF',
    wordWrap: false,
    showInvisibles: false,
    fontSize: 14
  });

  return {
    subscribe,
    setCursor: (line: number, column: number) => 
      update(state => ({ ...state, cursor: { line, column } })),
    setStats: (length: number, lines: number) => 
      update(state => ({ ...state, stats: { length, lines } })),
    setLanguage: (language: string) => 
      update(state => ({ ...state, language })),
    setEncoding: (encoding: string) => 
      update(state => ({ ...state, encoding })),
    setWordWrap: (enabled: boolean) => 
      update(state => {
        configStore.save({ word_wrap: enabled });
        return { ...state, wordWrap: enabled };
      }),
    setShowInvisibles: (enabled: boolean) => 
      update(state => {
        configStore.save({ show_invisibles: enabled });
        return { ...state, showInvisibles: enabled };
      }),
    setLineEnding: (ending: 'CRLF' | 'LF' | 'CR') =>
      update(state => ({ ...state, lineEnding: ending })),
    setFontSize: (size: number) => 
      update(state => {
        configStore.save({ font_size: size });
        return { ...state, fontSize: size };
      })
  };
}

export const editorStore = createEditorStore();