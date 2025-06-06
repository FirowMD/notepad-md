import { writable } from 'svelte/store';

export type Language = string;

interface LanguageExtension {
  [key: string]: string;
}

export const languageExtensions: LanguageExtension = {
  'txt': 'plaintext',
  'md': 'markdown',
  'html': 'html',
  'htm': 'html',
  'css': 'css',
  'scss': 'scss',
  'less': 'less',
  'js': 'javascript',
  'jsx': 'jsx',
  'ts': 'typescript',
  'tsx': 'tsx',
  'json': 'json',
  'vue': 'vue',
  'svelte': 'svelte',
  'yml': 'yaml',
  'yaml': 'yaml',
  'toml': 'toml',
  'ini': 'ini',
  'env': 'env',
  'conf': 'config',
  'config': 'config',
  'editorconfig': 'editorconfig',
  'sh': 'bash',
  'bash': 'bash',
  'bat': 'bat',
  'cmd': 'bat',
  'ps1': 'powershell',
  'psm1': 'powershell',
  'psd1': 'powershell',
  'py': 'python',
  'rb': 'ruby',
  'pl': 'perl',
  'lua': 'lua',
  'go': 'go',
  'rs': 'rust',
  'cpp': 'cpp',
  'c': 'c',
  'h': 'c',
  'hpp': 'cpp',
  'java': 'java',
  'cs': 'csharp',
  'php': 'php',
  'swift': 'swift',
  'kt': 'kotlin',
  'scala': 'scala',
  'dart': 'dart',
  'r': 'r',
  'hs': 'haskell',
  'ex': 'elixir',
  'erl': 'erlang',
  'fs': 'fsharp',
  'nim': 'nim',
  'zig': 'zig',
  'vb': 'vbnet',
  'groovy': 'groovy',
  'clj': 'clojure',
  'scm': 'scheme',
  'ml': 'ocaml',
  'jl': 'julia',
  'dockerfile': 'dockerfile',
  'tf': 'terraform',
  'hcl': 'terraform',
  'gitignore': 'gitignore',
  'gradle': 'gradle',
  'bazel': 'bazel',
  'cmake': 'cmake',
  'vagrantfile': 'vagrantfile',
  'sql': 'sql',
  'stata': 'stata',
  'sas': 'sas',
  'm': 'matlab',
  'nb': 'wolfram',
  'asm': 'nasm',
  's': 'gas',
  'hex': 'hex',
  'gdb': 'gdb',
  'feature': 'gherkin',
  'rst': 'restructuredtext',
  'adoc': 'asciidoc',
  'tex': 'latex',
  'bib': 'bibtex',
  'log': 'log',
  'diff': 'diff',
  'patch': 'patch',
  'puml': 'plantuml',
  'mmd': 'mermaid',
  'xml': 'xml',
  'xaml': 'xml',
  'xhtml': 'html',
  'svg': 'xml',
  'plist': 'xml',
  'reg': 'registry',
  'vbs': 'vb',
  'wsf': 'xml',
  'razor': 'razor',
  'cshtml': 'razor',
  'vbhtml': 'razor',
  'aspx': 'aspx',
  'ascx': 'aspx',
  'asmx': 'aspx',
  'ashx': 'aspx'
};

export const availableLanguages = [
  'plaintext', 'ansible', 'asciidoc', 'assembly', 'aspx', 'bat', 'bash', 'bazel', 'bibtex', 
  'c', 'chef', 'clojure', 'cmake', 'config', 'cpp', 'csharp', 'css', 'dart', 'diff', 
  'dockerfile', 'editorconfig', 'elixir', 'env', 'erlang', 'fsharp', 'gas', 'gdb', 
  'gherkin', 'gitignore', 'go', 'gradle', 'groovy', 'haskell', 'hex', 'html', 'ini', 
  'java', 'javascript', 'julia', 'json', 'jsx', 'kotlin', 'kubernetes', 'latex', 'less', 
  'log', 'lua', 'makefile', 'markdown', 'matlab', 'mermaid', 'nasm', 'nim', 'nix', 
  'ocaml', 'patch', 'perl', 'php', 'plantuml', 'powershell', 'properties', 'puppet', 
  'python', 'r', 'razor', 'registry', 'rego', 'restructuredtext', 'ruby', 'rust', 'sas', 
  'scala', 'scheme', 'scss', 'sql', 'stata', 'svelte', 'swift', 'terraform', 'toml', 
  'tsx', 'typescript', 'vagrantfile', 'vb', 'vbnet', 'vue', 'wolfram', 'xml', 'yaml', 'zig'
];


export function getLanguageFromExtension(ext: string): string {
  return languageExtensions[ext] || 'plaintext';
}