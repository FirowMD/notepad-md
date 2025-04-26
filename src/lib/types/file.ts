export interface FileInfo {
  id: string;
  path: string;
  name: string;
  content: string;
  encoding: string;
  language: string;
  created: Date;
  modified: Date;
  cursor: {
    line: number;
    column: number;
  };
  stats: {
    lines: number;
    length: number;
  };
}