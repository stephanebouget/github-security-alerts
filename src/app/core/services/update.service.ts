import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

export interface UpdateInfo {
  updateAvailable: boolean;
  currentVersion: string;
  newVersion?: string;
}

@Injectable({
  providedIn: 'root',
})
export class UpdateService {
  private updateCheckInterval?: number;
  private _updateAvailable = false;
  private _refreshIntervalMinutes = 60; // Default to 1 hour

  get updateAvailable(): boolean {
    return this._updateAvailable;
  }

  constructor() {}

  get isTauri(): boolean {
    return !!(window && (window as any).__TAURI__);
  }

  async getCurrentVersion(): Promise<string> {
    if (!this.isTauri) {
      return 'dev-mode';
    }

    try {
      return await invoke<string>('get_current_version');
    } catch (error) {
      console.error('Failed to get current version:', error);
      return 'unknown';
    }
  }

  async checkForUpdates(): Promise<boolean> {
    if (!this.isTauri) {
      console.log('Updates not supported in dev mode');
      return false;
    }

    try {
      const updateAvailable = await invoke<boolean>('check_for_updates');
      console.log('Update check result:', updateAvailable);
      this._updateAvailable = updateAvailable;
      return updateAvailable;
    } catch (error) {
      console.error('Failed to check for updates:', error);

      // Log platform-specific issues
      const errorString = String(error);
      if (
        errorString.includes('invalid updater binary format') ||
        errorString.includes('binary format')
      ) {
        console.warn(
          'Update check failed due to binary format issues. This may indicate a problem with the GitHub release assets for your platform.'
        );
      }

      this._updateAvailable = false;
      return false;
    }
  }

  async installUpdate(): Promise<void> {
    if (!this.isTauri) {
      console.log('Updates not supported in dev mode');
      return;
    }

    try {
      await invoke('install_update');
      console.log('Update installation started');
    } catch (error) {
      console.error('Failed to install update:', error);

      // Provide user-friendly error messages for common issues
      const errorString = String(error);
      if (
        errorString.includes('invalid updater binary format') ||
        errorString.includes('binary format')
      ) {
        throw new Error(
          'Update failed: Binary format incompatible with your operating system. Please download manually from GitHub.'
        );
      } else if (errorString.includes('signature')) {
        throw new Error(
          'Update failed: Security signature verification failed. Please download manually from GitHub.'
        );
      } else {
        throw error;
      }
    }
  }

  startAutomaticUpdateCheck(refreshIntervalMinutes?: number): void {
    if (!this.isTauri) {
      console.log('Automatic updates not supported in dev mode');
      return;
    }

    if (refreshIntervalMinutes !== undefined) {
      this._refreshIntervalMinutes = refreshIntervalMinutes;
    }

    // Clear existing interval if any
    this.stopAutomaticUpdateCheck();

    // Check immediately
    this.performSilentUpdateCheck();

    // Set up periodic checks using the configurable interval
    const intervalMs = this._refreshIntervalMinutes * 60 * 1000;
    this.updateCheckInterval = window.setInterval(() => {
      this.performSilentUpdateCheck();
    }, intervalMs);

    console.log(
      `Automatic update checking started (every ${this._refreshIntervalMinutes} minutes)`
    );
  }

  stopAutomaticUpdateCheck(): void {
    if (this.updateCheckInterval) {
      clearInterval(this.updateCheckInterval);
      this.updateCheckInterval = undefined;
      console.log('Automatic update checking stopped');
    }
  }

  updateRefreshInterval(refreshIntervalMinutes: number): void {
    this._refreshIntervalMinutes = refreshIntervalMinutes;
    // Restart with new interval
    if (this.updateCheckInterval) {
      this.startAutomaticUpdateCheck();
    }
  }

  private async performSilentUpdateCheck(): Promise<void> {
    try {
      console.log('Performing silent update check...');
      const updateAvailable = await this.checkForUpdates();

      if (updateAvailable) {
        console.log('Update available - showing notification in UI');
      } else {
        console.log('No updates available');
      }
    } catch (error) {
      console.error('Silent update check failed:', error);
    }
  }

  async getUpdateInfo(): Promise<UpdateInfo> {
    const currentVersion = await this.getCurrentVersion();
    const updateAvailable = await this.checkForUpdates();

    return {
      updateAvailable,
      currentVersion,
      newVersion: updateAvailable ? 'Available' : undefined,
    };
  }
}
