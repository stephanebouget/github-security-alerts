import { Component, OnInit, OnDestroy } from '@angular/core';
import { TranslateService } from '@ngx-translate/core';
import { APP_CONFIG } from '../environments/environment';
import { TauriService, AlertsResponse } from './core/services';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss'],
  standalone: false,
})
export class AppComponent implements OnInit, OnDestroy {
  githubToken: string = '';
  alerts: AlertsResponse | null = null;
  loading: boolean = false;
  error: string = '';
  refreshInterval: any;
  showSettings: boolean = false;

  constructor(
    private tauriService: TauriService,
    private translate: TranslateService
  ) {
    translate.setFallbackLang('en');
    console.log('APP_CONFIG', APP_CONFIG);

    if (this.tauriService.isTauri) {
      console.log('Run in Tauri');
    } else {
      console.log('Run in browser');
    }
  }

  ngOnInit() {
    const savedToken = localStorage.getItem('github_token');
    if (savedToken) {
      this.githubToken = savedToken;
      this.showSettings = false;
      this.fetchAlerts();
      this.startAutoRefresh();
    } else {
      this.showSettings = true;
    }
  }

  startAutoRefresh() {
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval);
    }
    this.refreshInterval = setInterval(() => {
      this.fetchAlerts();
    }, 5 * 60 * 1000);
  }

  saveToken() {
    if (this.githubToken.trim()) {
      localStorage.setItem('github_token', this.githubToken);
      this.showSettings = false;
      this.fetchAlerts();
      this.startAutoRefresh();
    }
  }

  toggleSettings() {
    this.showSettings = !this.showSettings;
  }

  async fetchAlerts() {
    if (!this.githubToken.trim()) {
      this.error = 'Please enter a GitHub token';
      this.showSettings = true;
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
    return this.alerts.total_alerts === 0 ? '✓' : '⚠';
  }

  getLastUpdate(): string {
    return new Date().toLocaleTimeString();
  }

  ngOnDestroy() {
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval);
    }
  }
}
