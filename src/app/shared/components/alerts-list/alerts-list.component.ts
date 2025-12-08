import { Component, EventEmitter, Input, Output } from '@angular/core';
import { AlertsResponse } from '../../../core/services';

@Component({
  selector: 'app-alerts-list',
  templateUrl: './alerts-list.component.html',
  styleUrls: ['./alerts-list.component.scss'],
  standalone: false,
})
export class AlertsListComponent {
  @Input() alerts: AlertsResponse | null = null;
  @Input() alertsLoading = false;
  @Input() error = '';

  @Output() showRepos = new EventEmitter<void>();

  getAlertIcon(): string {
    if (!this.alerts) return '🔵';
    return this.alerts.total_alerts === 0 ? '✓' : '⚠';
  }

  onShowRepos(): void {
    this.showRepos.emit();
  }
}
