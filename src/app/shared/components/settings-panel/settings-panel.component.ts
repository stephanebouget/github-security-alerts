import { Component, EventEmitter, Input, Output } from '@angular/core';

@Component({
  selector: 'app-settings-panel',
  templateUrl: './settings-panel.component.html',
  styleUrls: ['./settings-panel.component.scss'],
  standalone: false,
})
export class SettingsPanelComponent {
  @Input() username: string | null = null;

  @Output() logout = new EventEmitter<void>();

  onLogout(): void {
    this.logout.emit();
  }
}
