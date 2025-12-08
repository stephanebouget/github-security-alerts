import { Component, OnInit } from '@angular/core';
import { AlertsResponse, TauriService } from '../core/services';

@Component({
  selector: 'app-home',
  templateUrl: './home.component.html',
  styleUrls: ['./home.component.scss'],
  standalone: false,
})
export class HomeComponent implements OnInit {
  alerts: AlertsResponse | null = null;
  loading: boolean = false;
  error: string = '';
  refreshInterval: any;

  intervalTimeout: number = 60 * 60 * 1000; // 1 hour

  constructor(private tauriService: TauriService) {}

  ngOnInit() {
    // Start fetching alerts - the backend handles authentication
    this.fetchAlerts();
    this.refreshInterval = setInterval(() => {
      this.fetchAlerts();
    }, this.intervalTimeout);
  }

  async fetchAlerts() {
    this.loading = true;
    this.error = '';

    try {
      const response = await this.tauriService.getGitHubSecurityAlerts();
      this.alerts = response;

      await this.tauriService.updateTrayIcon(response.total_alerts);
    } catch (err) {
      this.error = `Failed to fetch alerts: ${err}`;
      console.error('Error fetching alerts:', err);
    } finally {
      this.loading = false;
    }
  }

  getAlertIcon(): string {
    if (!this.alerts) return 'ti ti-circle-filled';
    return this.alerts.total_alerts === 0 ? 'ti ti-circle-check-filled' : 'ti ti-alert-circle-filled';
  }

  ngOnDestroy() {
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval);
    }
  }
}
