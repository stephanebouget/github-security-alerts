import { Component, OnInit, OnDestroy } from '@angular/core';
import { TranslateService } from '@ngx-translate/core';
import { APP_CONFIG } from '../environments/environment';
import {
  TauriService,
  AlertsResponse,
  AuthStatus,
  RepoInfo,
  OwnerInfo,
} from './core/services';

type ViewMode = 'alerts' | 'repos' | 'settings';

interface OwnerAccordion {
  owner: OwnerInfo;
  expanded: boolean;
  loading: boolean;
  loaded: boolean;
  repos: RepoInfo[];
}

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss'],
  standalone: false,
})
export class AppComponent implements OnInit, OnDestroy {
  // Auth state
  authenticated = false;
  username: string | null = null;
  authLoading = false;
  tokenInput = '';

  // Owners & Repos state
  owners: OwnerAccordion[] = [];
  ownersLoading = false;
  searchQuery = '';

  // Alerts state
  alerts: AlertsResponse | null = null;
  alertsLoading = false;

  // UI state
  currentView: ViewMode = 'alerts';
  error = '';
  refreshInterval: any;

  constructor(
    private tauriService: TauriService,
    private translate: TranslateService
  ) {
    translate.setFallbackLang('en');
    console.log('APP_CONFIG', APP_CONFIG);
  }

  async ngOnInit() {
    await this.checkAuthStatus();
  }

  async checkAuthStatus() {
    try {
      const status: AuthStatus = await this.tauriService.getAuthStatus();
      this.authenticated = status.authenticated;
      this.username = status.username;

      if (this.authenticated) {
        await this.fetchAlerts();
        this.startAutoRefresh();
      }
    } catch (err) {
      console.error('Error checking auth status:', err);
    }
  }

  async login() {
    if (!this.tokenInput.trim()) {
      this.error = 'Please enter a GitHub token';
      return;
    }

    this.authLoading = true;
    this.error = '';

    try {
      await this.tauriService.setToken(this.tokenInput.trim());
      await this.checkAuthStatus();

      if (this.authenticated) {
        this.tokenInput = '';
        await this.loadOwners();
        this.currentView = 'repos';
      }
    } catch (err) {
      this.error = `Authentication failed: ${err}`;
    } finally {
      this.authLoading = false;
    }
  }

  async logout() {
    try {
      await this.tauriService.logout();
      this.authenticated = false;
      this.username = null;
      this.owners = [];
      this.alerts = null;
      this.stopAutoRefresh();
      await this.tauriService.updateTrayIcon(0);
    } catch (err) {
      this.error = `Failed to logout: ${err}`;
    }
  }

  openCreateTokenPage() {
    const tokenUrl = 'https://github.com/settings/tokens/new?description=Github-Security-Alerts&scopes=read%3Auser%2Cnotifications%2Crepo%2Cread%3Aorg';
    this.tauriService.openExternalLink(tokenUrl).catch(err => {
      console.error('Failed to open URL:', err);
      this.error = 'Failed to open GitHub token page';
    });
  }

  async loadOwners() {
    this.ownersLoading = true;
    this.error = '';
    try {
      const ownerInfos = await this.tauriService.getOwners();
      this.owners = ownerInfos.map((owner, index) => ({
        owner,
        expanded: index === 0, // Expand first (user's personal repos)
        loading: false,
        loaded: false,
        repos: [],
      }));

      // Auto-load repos for the first owner (user's personal repos)
      if (this.owners.length > 0) {
        await this.toggleOwner(this.owners[0]);
      }
    } catch (err) {
      this.error = `Failed to load owners: ${err}`;
    } finally {
      this.ownersLoading = false;
    }
  }

  async toggleOwner(ownerAccordion: OwnerAccordion) {
    // If closing, just toggle
    if (ownerAccordion.expanded && ownerAccordion.loaded) {
      ownerAccordion.expanded = false;
      return;
    }

    // Open accordion
    ownerAccordion.expanded = true;

    // If not loaded, fetch repos
    if (!ownerAccordion.loaded) {
      ownerAccordion.loading = true;
      try {
        const repos = await this.tauriService.getReposForOwner(
          ownerAccordion.owner.name,
          ownerAccordion.owner.is_user
        );
        ownerAccordion.repos = repos;
        ownerAccordion.loaded = true;
      } catch (err) {
        this.error = `Failed to load repos for ${ownerAccordion.owner.name}: ${err}`;
      } finally {
        ownerAccordion.loading = false;
      }
    }
  }

  async toggleRepo(repo: RepoInfo) {
    repo.selected = !repo.selected;
    await this.saveSelectedRepos();
  }

  async saveSelectedRepos() {
    const allRepos = this.owners.flatMap((o) => o.repos);
    const selectedRepos = allRepos
      .filter((r: RepoInfo) => r.selected)
      .map((r: RepoInfo) => r.full_name);

    try {
      await this.tauriService.setSelectedRepos(selectedRepos);
    } catch (err) {
      this.error = `Failed to save selection: ${err}`;
    }
  }

  async selectAllForOwner(ownerAccordion: OwnerAccordion) {
    ownerAccordion.repos.forEach((r) => (r.selected = true));
    await this.saveSelectedRepos();
  }

  async selectNoneForOwner(ownerAccordion: OwnerAccordion) {
    ownerAccordion.repos.forEach((r) => (r.selected = false));
    await this.saveSelectedRepos();
  }

  getFilteredRepos(ownerAccordion: OwnerAccordion): RepoInfo[] {
    if (!this.searchQuery.trim()) {
      return ownerAccordion.repos;
    }
    const query = this.searchQuery.toLowerCase();
    return ownerAccordion.repos.filter(
      (r) =>
        r.name.toLowerCase().includes(query) ||
        r.owner.toLowerCase().includes(query)
    );
  }

  get totalReposCount(): number {
    return this.owners.reduce((sum, o) => sum + o.repos.length, 0);
  }

  get selectedCount(): number {
    return this.owners.reduce(
      (sum, o) => sum + o.repos.filter((r) => r.selected).length,
      0
    );
  }

  getSelectedCountForOwner(ownerAccordion: OwnerAccordion): number {
    return ownerAccordion.repos.filter((r) => r.selected).length;
  }

  startAutoRefresh() {
    this.stopAutoRefresh();
    this.refreshInterval = setInterval(() => {
      this.fetchAlerts();
    }, 5 * 60 * 1000);
  }

  stopAutoRefresh() {
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval);
      this.refreshInterval = null;
    }
  }

  async fetchAlerts() {
    if (!this.authenticated) {
      return;
    }

    this.alertsLoading = true;
    this.error = '';

    try {
      const response = await this.tauriService.getGitHubSecurityAlerts();
      this.alerts = response;
      await this.tauriService.updateTrayIcon(response.total_alerts);
    } catch (err) {
      this.error = `Failed to fetch alerts: ${err}`;
      console.error('Error fetching alerts:', err);
    } finally {
      this.alertsLoading = false;
    }
  }

  showAlerts() {
    this.currentView = 'alerts';
    this.fetchAlerts();
  }

  async showRepos() {
    this.currentView = 'repos';
    if (this.owners.length === 0) {
      await this.loadOwners();
    }
  }

  showSettings() {
    this.currentView = 'settings';
  }

  getAlertIcon(): string {
    if (!this.alerts) return '🔵';
    return this.alerts.total_alerts === 0 ? '✓' : '⚠';
  }

  getLastUpdate(): string {
    return new Date().toLocaleTimeString();
  }

  ngOnDestroy() {
    this.stopAutoRefresh();
  }
}
