import { Component, EventEmitter, Input, Output, OnInit, OnDestroy } from '@angular/core';
import { TauriService, UpdateService } from '../../../core/services';

@Component({
  selector: 'app-settings-panel',
  templateUrl: './settings-panel.component.html',
  styleUrls: ['./settings-panel.component.scss'],
  standalone: false,
})
export class SettingsPanelComponent implements OnInit, OnDestroy {
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

  private autoHideTimeout?: number;

  onSelectFocus(event: Event): void {
    // Prevent window from closing when dropdown is opened
    if (this.tauriService.isTauri) {
      this.tauriService.pauseAutoHide().catch(err => 
        console.warn('Failed to pause auto-hide:', err)
      );
      
      // Clear any existing timeout
      if (this.autoHideTimeout) {
        clearTimeout(this.autoHideTimeout);
      }
      
      // Set a safety timeout to resume auto-hide after 10 seconds
      this.autoHideTimeout = window.setTimeout(() => {
        this.tauriService.resumeAutoHide().catch(err => 
          console.warn('Failed to resume auto-hide (timeout):', err)
        );
      }, 10000);
    }
  }

  onSelectBlur(event: Event): void {
    // Allow window to close normally when dropdown is closed
    if (this.tauriService.isTauri) {
      // Clear the safety timeout
      if (this.autoHideTimeout) {
        clearTimeout(this.autoHideTimeout);
        this.autoHideTimeout = undefined;
      }
      
      this.tauriService.resumeAutoHide().catch(err => 
        console.warn('Failed to resume auto-hide:', err)
      );
    }
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
      // Get current state from UpdateService instead of calling checkForUpdates
      this.updateAvailable = this.updateService.updateAvailable;
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

  ngOnDestroy(): void {
    // Clean up timeout on component destruction
    if (this.autoHideTimeout) {
      clearTimeout(this.autoHideTimeout);
      
      // Ensure auto-hide is resumed
      if (this.tauriService.isTauri) {
        this.tauriService.resumeAutoHide().catch(err => 
          console.warn('Failed to resume auto-hide on destroy:', err)
        );
      }
    }
  }
}
