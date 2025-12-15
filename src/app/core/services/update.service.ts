import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

export interface UpdateInfo {
  updateAvailable: boolean;
  currentVersion: string;
  newVersion?: string;
}

@Injectable({
  providedIn: 'root'
})
export class UpdateService {
  private updateCheckInterval?: number;
  private readonly CHECK_INTERVAL = 1000 * 60 * 60; // Check every hour

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
      return updateAvailable;
    } catch (error) {
      console.error('Failed to check for updates:', error);
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
      throw error;
    }
  }

  startAutomaticUpdateCheck(): void {
    if (!this.isTauri) {
      console.log('Automatic updates not supported in dev mode');
      return;
    }

    // Clear existing interval if any
    this.stopAutomaticUpdateCheck();

    // Check immediately
    this.performSilentUpdateCheck();

    // Set up periodic checks
    this.updateCheckInterval = window.setInterval(() => {
      this.performSilentUpdateCheck();
    }, this.CHECK_INTERVAL);

    console.log('Automatic update checking started');
  }

  stopAutomaticUpdateCheck(): void {
    if (this.updateCheckInterval) {
      clearInterval(this.updateCheckInterval);
      this.updateCheckInterval = undefined;
      console.log('Automatic update checking stopped');
    }
  }

  private async performSilentUpdateCheck(): Promise<void> {
    try {
      console.log('Performing silent update check...');
      const updateAvailable = await this.checkForUpdates();
      
      if (updateAvailable) {
        console.log('Silent update available, installing automatically...');
        
        // Wait a bit to ensure the app is in a stable state
        await new Promise(resolve => setTimeout(resolve, 5000));
        
        // Install the update silently
        await this.installUpdate();
        
        console.log('Silent update installation initiated');
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
      newVersion: updateAvailable ? 'Available' : undefined
    };
  }
}