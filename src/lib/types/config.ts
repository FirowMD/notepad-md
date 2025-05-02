import type { Theme } from '../stores/theme';

export interface AppConfig {
  colorscheme?: Theme;
  recent_files?: string[];
  opened_files?: string[];
  font_size?: number;
  word_wrap?: boolean;
  show_invisibles?: boolean;
  default_encoding?: string;
}