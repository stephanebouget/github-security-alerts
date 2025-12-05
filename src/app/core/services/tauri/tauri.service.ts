import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

export interface RepoAlerts {
  name: string;
  alerts: number;
}

export interface AlertsResponse {
  total_alerts: number;
  repos: RepoAlerts[];
}

@Injectable({
  providedIn: 'root',
})
export class TauriService {
  constructor() {}

  get isTauri(): boolean {
    return !!(window && window.__TAURI__);
  }

  async getGitHubSecurityAlerts(githubToken: string): Promise<AlertsResponse> {
    try {
      const response = await invoke<AlertsResponse>(
        'get_github_security_alerts',
        {
          githubToken: githubToken,
        }
      );
      return response;
    } catch (error) {
      console.error('Error fetching GitHub security alerts:', error);
      throw error;
    }
  }

  async updateTrayIcon(alertCount: number): Promise<void> {
    try {
      await invoke('update_tray_icon', {
        alertCount: alertCount,
      });
    } catch (error) {
      console.error('Error updating tray icon:', error);
    }
  }
}
