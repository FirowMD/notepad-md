export interface FileInfo {
  id: string;
  path: string;
  name: string;
  content: string;
  encoding: string;
  language: string;
  created: Date;
  modified: Date;
  fileSystemModified?: Date; // File system's last modified time
  isModified: boolean;
  hash: string;
  cursor: {
    line: number;
    column: number;
  };
  stats: {
    lines: number;
    length: number;
  };
}