import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

export interface RepoAlerts {
  name: string;
  alerts: number;
  dependabot_enabled: boolean;
}

export interface AlertsResponse {
  total_alerts: number;
  repos: RepoAlerts[];
}

export interface RepoInfo {
  full_name: string;
  name: string;
  owner: string;
  selected: boolean;
}

export interface AuthStatus {
  authenticated: boolean;
  username: string | null;
}

export interface OwnerInfo {
  name: string;
  is_user: boolean;
}

@Injectable({
  providedIn: 'root',
})
export class TauriService {
  constructor() {}

  get isTauri(): boolean {
    return !!(window && (window as any).__TAURI__);
  }

  // Token Authentication
  async setToken(token: string): Promise<void> {
    return invoke('set_token', { token });
  }

  async getAuthStatus(): Promise<AuthStatus> {
    return invoke<AuthStatus>('get_auth_status');
  }

  async getToken(): Promise<string | null> {
    return invoke<string | null>('get_token');
  }

  async logout(): Promise<void> {
    return invoke('logout');
  }

  // Repository Management
  async getOwners(): Promise<OwnerInfo[]> {
    return invoke<OwnerInfo[]>('get_owners');
  }

  async getReposForOwner(owner: string, isUser: boolean): Promise<RepoInfo[]> {
    return invoke<RepoInfo[]>('get_repos_for_owner', { owner, isUser });
  }

  async setSelectedRepos(repos: string[]): Promise<void> {
    return invoke('set_selected_repos', { repos });
  }

  async getSelectedRepos(): Promise<string[]> {
    return invoke<string[]>('get_selected_repos');
  }

  // Security Alerts
  async getGitHubSecurityAlerts(): Promise<AlertsResponse> {
    return invoke<AlertsResponse>('get_github_security_alerts');
  }

  async updateTrayIcon(alertCount: number): Promise<void> {
    return invoke('update_tray_icon', { alertCount });
  }

  async openExternalLink(url: string): Promise<void> {
    if (this.isTauri) {
      const { open } = await import('@tauri-apps/plugin-shell');

      try {
        await open(url);
      } catch (error) {
        console.error('Erreur Tauri:', error);
        window.open(url, '_blank');
      }
    } else {
      window.open(url, '_blank');
    }
  }

  // Platform Detection
  isWindows(): boolean {
    // Simple platform detection that works both in Tauri and browser
    return (
      navigator.platform.toLowerCase().includes('win') ||
      navigator.userAgent.toLowerCase().includes('windows')
    );
  }

  // System Settings
  async openTaskbarSettings(): Promise<void> {
    return invoke('open_taskbar_settings');
  }

  async flashTrayIcon(): Promise<void> {
    return invoke('flash_tray_icon');
  }

  // OAuth Authentication
  async startOAuthFlow(): Promise<string> {
    return invoke<string>('start_oauth_flow');
  }

  async completeOAuthFlow(): Promise<string> {
    return invoke<string>('complete_oauth_flow');
  }
}
