export type Risk = 'safe' | 'caution';
export type Method = 'trash' | 'delete' | 'command';
export type CategoryGroup = 'global' | 'project';

export interface DiskInfo {
  total: number;
  available: number;
  mountPoint: string;
}

export interface Category {
  id: string;
  name: string;
  description: string;
  group: CategoryGroup;
  method: Method;
  risk: Risk;
  available: boolean;
  unavailableReason?: string;
}

export interface ScanItem {
  id: string;
  categoryId: string;
  path: string;
  bytes: number;
  projectName?: string;
  lastActivity?: number;
  stale?: boolean;
  note?: string;
}

export interface ProjectArtifact {
  id: string;
  categoryId: string;
  name: string;
  path: string;
  bytes: number;
  risk: Risk;
  method: Method;
  note?: string;
}

export interface ScannedProject {
  id: string;
  name: string;
  path: string;
  projectType: string;
  lastActivity: number;
  stale: boolean;
  totalBytes: number;
  artifacts: ProjectArtifact[];
}

export interface CleanFailedItem {
  id: string;
  error: string;
}

export interface CleanResult {
  freedBytes: number;
  succeeded: string[];
  failed: CleanFailedItem[];
  dryRun: boolean;
}

export interface ScanProgress {
  scanId: string;
  phase: string;
  currentPath: string;
  itemsFound: number;
  bytesFound: number;
}

export interface CleanProgress {
  done: number;
  total: number;
  currentPath: string;
}

export interface Settings {
  projectRoots: string[];
  staleThresholdDays: number;
  enabledCategories: string[];
  excludePaths: string[];
  maxScanDepth: number;
  dryRun: boolean;
}

export interface HistoryCleanedItem {
  categoryId: string;
  path: string;
  bytes: number;
}

export interface HistoryEntry {
  id: string;
  timestamp: number;
  freedBytes: number;
  itemCount: number;
  dryRun: boolean;
  items: HistoryCleanedItem[];
}
