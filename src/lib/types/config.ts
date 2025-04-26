import type { Theme } from '../stores/theme';

export interface AppConfig {
  colorscheme?: Theme;
  recent_files?: string[];
  opened_files?: string[];
  font_size?: number;
}