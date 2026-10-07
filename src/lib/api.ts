import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  DiskInfo,
  Category,
  ScanItem,
  ScannedProject,
  CleanResult,
  ScanProgress,
  CleanProgress,
  Settings,
  HistoryEntry,
} from './types';

export const api = {
  // Disk & System
  async getDiskInfo(): Promise<DiskInfo> {
    return invoke<DiskInfo>('get_disk_info');
  },

  async checkFullDiskAccess(): Promise<boolean> {
    return invoke<boolean>('check_full_disk_access');
  },

  // Categories
  async listCategories(): Promise<Category[]> {
    return invoke<Category[]>('list_categories');
  },

  // Scanning
  async scanGlobal(scanId: string): Promise<ScanItem[]> {
    return invoke<ScanItem[]>('scan_global', { scanId });
  },

  async scanProjects(scanId: string, roots: string[] = []): Promise<ScannedProject[]> {
    return invoke<ScannedProject[]>('scan_projects', { scanId, roots });
  },

  async cancelScan(scanId: string): Promise<void> {
    return invoke<void>('cancel_scan', { scanId });
  },

  // Cleaning
  async cleanItems(itemIds: string[]): Promise<CleanResult> {
    return invoke<CleanResult>('clean_items', { itemIds });
  },

  // Settings
  async getSettings(): Promise<Settings> {
    return invoke<Settings>('get_settings');
  },

  async saveSettings(settings: Settings): Promise<Settings> {
    return invoke<Settings>('save_settings', { settings });
  },

  // History
  async getHistory(): Promise<HistoryEntry[]> {
    return invoke<HistoryEntry[]>('get_history');
  },

  // Open in file manager
  async openInFileManager(itemId: string): Promise<void> {
    return invoke<void>('open_in_file_manager', { itemId });
  },

  // Event Listeners
  onScanProgress(callback: (payload: ScanProgress) => void): Promise<UnlistenFn> {
    return listen<ScanProgress>('scan://progress', (event) => callback(event.payload));
  },

  onCleanProgress(callback: (payload: CleanProgress) => void): Promise<UnlistenFn> {
    return listen<CleanProgress>('clean://progress', (event) => callback(event.payload));
  },
};
