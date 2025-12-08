import { Component, EventEmitter, Input, Output } from '@angular/core';
import { AlertsResponse, TauriService } from '../../../core/services';

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

  constructor(private tauriService: TauriService) {}

  getAlertIcon(): string {
    if (!this.alerts) return 'ti ti-circle-filled';
    return this.alerts.total_alerts === 0 ? 'ti ti-check' : 'ti ti-alert-triangle';
  }

  onShowRepos(): void {
    this.showRepos.emit();
  }

  openRepoOnGitHub(repoFullName: string): void {
    const url = `https://github.com/${repoFullName}/security/dependabot`;
    this.tauriService.openExternalLink(url);
  }
}
