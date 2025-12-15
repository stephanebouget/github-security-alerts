import { Component, EventEmitter, Input, Output, OnInit } from '@angular/core';
import { TauriService, UpdateService } from '../../../core/services';

@Component({
  selector: 'app-settings-panel',
  templateUrl: './settings-panel.component.html',
  styleUrls: ['./settings-panel.component.scss'],
  standalone: false,
})
export class SettingsPanelComponent implements OnInit {
  @Input() username: string | null = null;
  @Input() refreshInterval: number = 60; // in minutes

  @Output() logout = new EventEmitter<void>();
  @Output() refreshIntervalChange = new EventEmitter<number>();

  isWindows = false;
  currentVersion = '';
  updateAvailable = false;
  isCheckingForUpdates = false;
  isInstallingUpdate = false;

  constructor(
    private tauriService: TauriService,
    public updateService: UpdateService
  ) {}

  async ngOnInit(): Promise<void> {
    this.isWindows = this.tauriService.isWindows();
    await this.loadUpdateInfo();
  }

  onLogout(): void {
    this.logout.emit();
  }

  onRefreshIntervalChange(event: Event): void {
    const value = parseInt((event.target as HTMLSelectElement).value, 10);
    this.refreshInterval = value;
    this.refreshIntervalChange.emit(value);
  }

  async openTaskbarSettings(): Promise<void> {
    if (this.isWindows) {
      try {
        await this.tauriService.openTaskbarSettings();
      } catch (error) {
        console.error('Failed to open taskbar settings:', error);
      }
    }
  }

  async loadUpdateInfo(): Promise<void> {
    try {
      this.currentVersion = await this.updateService.getCurrentVersion();
      this.updateAvailable = await this.updateService.checkForUpdates();
    } catch (error) {
      console.error('Failed to load update info:', error);
    }
  }

  async checkForUpdates(): Promise<void> {
    this.isCheckingForUpdates = true;
    try {
      this.updateAvailable = await this.updateService.checkForUpdates();
      if (!this.updateAvailable) {
        // Show a toast or notification that no updates are available
        console.log('No updates available');
      }
    } catch (error) {
      console.error('Failed to check for updates:', error);
    } finally {
      this.isCheckingForUpdates = false;
    }
  }

  async installUpdate(): Promise<void> {
    this.isInstallingUpdate = true;
    try {
      await this.updateService.installUpdate();
      console.log('Update installation started');
    } catch (error) {
      console.error('Failed to install update:', error);
    } finally {
      this.isInstallingUpdate = false;
    }
  }
}
