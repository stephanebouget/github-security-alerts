import { Component, EventEmitter, Input, Output } from '@angular/core';

@Component({
  selector: 'app-settings-panel',
  templateUrl: './settings-panel.component.html',
  styleUrls: ['./settings-panel.component.scss'],
  standalone: false,
})
export class SettingsPanelComponent {
  @Input() username: string | null = null;
  @Input() refreshInterval: number = 60; // in minutes

  @Output() logout = new EventEmitter<void>();
  @Output() refreshIntervalChange = new EventEmitter<number>();

  onLogout(): void {
    this.logout.emit();
  }

  onRefreshIntervalChange(event: Event): void {
    const value = parseInt((event.target as HTMLSelectElement).value, 10);
    this.refreshInterval = value;
    this.refreshIntervalChange.emit(value);
  }
}
