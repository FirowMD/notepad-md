export interface FileInfo {
  id: string;
  path: string;
  name: string;
  content: string;
  encoding: string;
  language: string;
  created: Date;
  modified: Date;
  isModified: boolean; // New field to track if file has unsaved changes
  cursor: {
    line: number;
    column: number;
  };
  stats: {
    lines: number;
    length: number;
  };
}