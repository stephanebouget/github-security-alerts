import { Component, OnInit, OnDestroy } from '@angular/core';
import { TranslateService } from '@ngx-translate/core';
import { APP_CONFIG } from '../environments/environment';
import {
  TauriService,
  AlertsResponse,
  AuthStatus,
  RepoInfo,
  OwnerInfo,
  UpdateService,
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
  oauthLoading = false;
  existingToken: string | null = null;

  // Owners & Repos state
  owners: OwnerAccordion[] = [];
  ownersLoading = false;
  searchQuery = '';

  // Track selected repos in memory to avoid losing them when accordions aren't loaded
  private selectedReposFullNames: Set<string> = new Set();

  // Alerts state
  alerts: AlertsResponse | null = null;
  alertsLoading = false;

  // UI state
  currentView: ViewMode = 'alerts';
  error = '';
  refreshInterval: any;
  refreshIntervalMinutes: number = 60; // Default to 1 hour

  constructor(
    private tauriService: TauriService,
    private updateService: UpdateService,
    private translate: TranslateService
  ) {
    translate.setFallbackLang('en');
    console.log('APP_CONFIG', APP_CONFIG);
  }

  private async loadRefreshIntervalFromConfig(): Promise<void> {
    try {
      this.refreshIntervalMinutes = await this.tauriService.getRefreshInterval();
    } catch (error) {
      console.warn('Failed to load refresh interval from config, using default:', error);
      this.refreshIntervalMinutes = 60; // Default to 1 hour
    }
  }

  private async saveRefreshIntervalToConfig(): Promise<void> {
    try {
      await this.tauriService.setRefreshInterval(this.refreshIntervalMinutes);
    } catch (error) {
      console.error('Failed to save refresh interval to config:', error);
    }
  }

  async ngOnInit() {
    await this.loadRefreshIntervalFromConfig();
    await this.checkAuthStatus();

    // Start automatic update checking with the same interval as alerts refresh
    this.updateService.startAutomaticUpdateCheck(this.refreshIntervalMinutes);
  }

  async checkAuthStatus() {
    try {
      const status: AuthStatus = await this.tauriService.getAuthStatus();
      this.authenticated = status.authenticated;
      this.username = status.username;

      // Always get the token to display it masked when not authenticated
      try {
        this.existingToken = await this.tauriService.getToken();
      } catch (err) {
        console.error('Error getting token:', err);
        this.existingToken = null;
      }

      if (this.authenticated) {
        await this.fetchAlerts();
        this.startAutoRefresh();
      }
    } catch (err) {
      console.error('Error checking auth status:', err);
      // Try to get the token even if auth status check failed
      try {
        this.existingToken = await this.tauriService.getToken();
      } catch (tokenErr) {
        console.error('Error getting token:', tokenErr);
        this.existingToken = null;
      }
    }
  }

  async login(token: string) {
    if (!token.trim()) {
      this.error = 'Please enter a GitHub token';
      return;
    }

    this.authLoading = true;
    this.error = '';

    try {
      await this.tauriService.setToken(token.trim());
      await this.checkAuthStatus();

      if (this.authenticated) {
        await this.loadOwners();
        this.currentView = 'repos';
      }
    } catch (err) {
      this.error = `Authentication failed: ${err}`;
    } finally {
      this.authLoading = false;
    }
  }

  onLogin(token: string): void {
    this.login(token);
  }

  async onStartOAuth(): Promise<void> {
    if (!this.tauriService.isTauri) {
      this.error = 'OAuth is only available in the desktop app';
      return;
    }

    this.oauthLoading = true;
    this.error = '';

    // Timeout after 30 seconds
    const timeoutMs = 30000;
    const timeoutPromise = new Promise<never>((_, reject) => {
      setTimeout(
        () => reject(new Error('Connection timeout. Please try again.')),
        timeoutMs
      );
    });

    try {
      // Get the OAuth URL and open it
      const oauthUrl = await this.tauriService.startOAuthFlow();

      await this.tauriService.openExternalLink(oauthUrl);

      // Start listening for the callback with timeout
      const accessToken = await Promise.race([
        this.tauriService.completeOAuthFlow(),
        timeoutPromise,
      ]);

      // Verify auth status
      await this.checkAuthStatus();

      if (this.authenticated) {
        await this.loadOwners();
        this.currentView = 'repos';
      }
    } catch (err) {
      console.error('OAuth error:', err);
      this.error = `OAuth authentication failed: ${err}`;
    } finally {
      this.oauthLoading = false;
    }
  }

  onViewChange(view: ViewMode): void {
    if (view === 'repos') {
      this.showRepos();
    } else if (view === 'settings') {
      this.showSettings();
    } else {
      this.showAlerts();
    }
  }

  async logout() {
    try {
      await this.tauriService.logout();
      this.authenticated = false;
      this.username = null;
      this.owners = [];
      this.alerts = null;
      this.selectedReposFullNames.clear();
      this.stopAutoRefresh();
      await this.tauriService.updateTrayIcon(0);
    } catch (err) {
      this.error = `Failed to logout: ${err}`;
    }
  }

  openCreateTokenPage() {
    const tokenUrl =
      'https://github.com/settings/tokens/new?description=Github-Security-Alerts&scopes=read%3Auser%2Cnotifications%2Crepo%2Cread%3Aorg';
    this.tauriService.openExternalLink(tokenUrl).catch((err) => {
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

      // Load previously selected repos from storage
      const selectedRepos = await this.tauriService.getSelectedRepos();
      this.selectedReposFullNames = new Set(selectedRepos);

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
        // Restore selection state from memory
        repos.forEach((repo) => {
          repo.selected = this.selectedReposFullNames.has(repo.full_name);
        });
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

    // Update in-memory tracking
    if (repo.selected) {
      this.selectedReposFullNames.add(repo.full_name);
    } else {
      this.selectedReposFullNames.delete(repo.full_name);
    }

    await this.saveSelectedRepos();
  }

  async saveSelectedRepos() {
    // Use the in-memory set of selected repos, which includes repos from
    // accordions that haven't been loaded yet
    const selectedRepos = Array.from(this.selectedReposFullNames);

    try {
      await this.tauriService.setSelectedRepos(selectedRepos);
    } catch (err) {
      this.error = `Failed to save selection: ${err}`;
    }
  }

  async selectAllForOwner(ownerAccordion: OwnerAccordion) {
    ownerAccordion.repos.forEach((r) => {
      r.selected = true;
      this.selectedReposFullNames.add(r.full_name);
    });
    await this.saveSelectedRepos();
  }

  async selectNoneForOwner(ownerAccordion: OwnerAccordion) {
    ownerAccordion.repos.forEach((r) => {
      r.selected = false;
      this.selectedReposFullNames.delete(r.full_name);
    });
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
    // Use the in-memory set to get accurate count, even for repos not yet loaded
    return this.selectedReposFullNames.size;
  }

  getSelectedCountForOwner(ownerAccordion: OwnerAccordion): number {
    return ownerAccordion.repos.filter((r) => r.selected).length;
  }

  startAutoRefresh() {
    this.stopAutoRefresh();

    // If interval is 0, don't start auto-refresh
    if (this.refreshIntervalMinutes === 0) {
      return;
    }

    const intervalMs = this.refreshIntervalMinutes * 60 * 1000;
    this.refreshInterval = setInterval(() => {
      this.fetchAlerts();
    }, intervalMs);
  }

  onRefreshIntervalChange(minutes: number): void {
    this.refreshIntervalMinutes = minutes;
    this.saveRefreshIntervalToConfig();
    this.startAutoRefresh();

    // Also update the update service interval
    this.updateService.updateRefreshInterval(minutes);
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

  ngOnDestroy() {
    this.stopAutoRefresh();
    this.updateService.stopAutomaticUpdateCheck();
  }
}
