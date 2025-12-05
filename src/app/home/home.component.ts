import { Component, OnInit } from '@angular/core';
import { AlertsResponse, TauriService } from '../core/services';

@Component({
  selector: 'app-home',
  templateUrl: './home.component.html',
  styleUrls: ['./home.component.scss'],
  standalone: false,
})
export class HomeComponent implements OnInit {
  githubToken: string = '';
  alerts: AlertsResponse | null = null;
  loading: boolean = false;
  error: string = '';
  refreshInterval: any;

  intervalTimeout: number = 60 * 60 * 1000; // 5 minutes

  constructor(private tauriService: TauriService) {}

  ngOnInit() {
    // Load the GitHub token from localStorage
    const savedToken = localStorage.getItem('github_token');
    if (savedToken) {
      this.githubToken = savedToken;
      this.fetchAlerts();
      this.refreshInterval = setInterval(() => {
        this.fetchAlerts();
      }, this.intervalTimeout);
    }
  }

  saveToken() {
    if (this.githubToken.trim()) {
      localStorage.setItem('github_token', this.githubToken);
      this.fetchAlerts();
      if (this.refreshInterval) {
        clearInterval(this.refreshInterval);
      }
      this.refreshInterval = setInterval(() => {
        this.fetchAlerts();
      }, this.intervalTimeout);
    }
  }

  async fetchAlerts() {
    if (!this.githubToken.trim()) {
      this.error = 'Please enter a GitHub token';
      return;
    }

    this.loading = true;
    this.error = '';

    try {
      const response = await this.tauriService.getGitHubSecurityAlerts(
        this.githubToken
      );
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
    if (!this.alerts) return '🔵';
    return this.alerts.total_alerts === 0 ? '🟢' : '🔴';
  }

  ngOnDestroy() {
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval);
    }
  }
}
