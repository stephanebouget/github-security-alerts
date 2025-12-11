import { Component, OnInit, OnDestroy } from '@angular/core';
import { GitHubRateLimitService, GitHubRateLimits } from '../../../core/services';
import { interval, Subscription } from 'rxjs';
import { startWith, switchMap } from 'rxjs/operators';

@Component({
  selector: 'app-rate-limit-status',
  templateUrl: './rate-limit-status.component.html',
  styleUrls: ['./rate-limit-status.component.scss'],
  standalone: false,
})
export class RateLimitStatusComponent implements OnInit, OnDestroy {
  rateLimits: GitHubRateLimits | null = null;
  isLoading = true;
  error: string | null = null;
  private refreshSubscription?: Subscription;

  constructor(private rateLimitService: GitHubRateLimitService) {}

  ngOnInit(): void {
    this.loadRateLimits();
    
    // Refresh every 5 minutes
    this.refreshSubscription = interval(5 * 60 * 1000)
      .pipe(
        startWith(0),
        switchMap(() => this.loadRateLimitsAsync())
      )
      .subscribe();
  }

  ngOnDestroy(): void {
    if (this.refreshSubscription) {
      this.refreshSubscription.unsubscribe();
    }
  }

  private async loadRateLimits(): Promise<void> {
    try {
      this.isLoading = true;
      this.error = null;
      this.rateLimits = await this.rateLimitService.getRateLimits();
    } catch (error) {
      this.error = error instanceof Error ? error.message : 'Failed to load rate limits';
      console.error('Error loading rate limits:', error);
    } finally {
      this.isLoading = false;
    }
  }

  private async loadRateLimitsAsync(): Promise<void> {
    try {
      this.rateLimits = await this.rateLimitService.getRateLimits();
      this.error = null;
    } catch (error) {
      this.error = error instanceof Error ? error.message : 'Failed to load rate limits';
      console.error('Error loading rate limits:', error);
    }
  }

  async refresh(): Promise<void> {
    this.isLoading = true;
    try {
      this.error = null;
      this.rateLimits = await this.rateLimitService.getRateLimits();
    } catch (error) {
      this.error = error instanceof Error ? error.message : 'Failed to load rate limits';
      console.error('Error loading rate limits:', error);
    } finally {
      this.isLoading = false;
    }
  }

  getUsagePercentage(remaining: number, limit: number): number {
    const used = limit - remaining;
    return this.rateLimitService.calculateUsagePercentage(used, limit);
  }

  formatResetTime(resetAt: string | number): string {
    return this.rateLimitService.formatResetTime(resetAt);
  }

  getStatusColor(percentage: number): string {
    if (percentage >= 90) return 'danger';
    if (percentage >= 70) return 'warning';
    return 'normal';
  }
}