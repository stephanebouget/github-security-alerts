import { Component, EventEmitter, Input, Output, OnInit } from '@angular/core';
import { TauriService } from '../../../core/services';

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

  constructor(private tauriService: TauriService) {}

  ngOnInit(): void {
    this.isWindows = this.tauriService.isWindows();
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
}
