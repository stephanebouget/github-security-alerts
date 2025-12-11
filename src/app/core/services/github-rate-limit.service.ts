import { Injectable } from '@angular/core';
import { TauriService } from './tauri/tauri.service';

export interface RateLimit {
  limit: number;
  remaining: number;
  resetAt: string | number;
  used: number;
}

export interface GraphQLRateLimit {
  limit: number;
  remaining: number;
  resetAt: string | number;
  cost: number;
}

export interface GitHubRateLimits {
  core: RateLimit;
  search: RateLimit;
  graphql: GraphQLRateLimit;
}

@Injectable({
  providedIn: 'root',
})
export class GitHubRateLimitService {
  private readonly API_BASE_URL = 'https://api.github.com';

  constructor(private tauriService: TauriService) {}

  /**
   * Get GitHub API rate limits
   */
  async getRateLimits(): Promise<GitHubRateLimits | null> {
    try {
      const authStatus = await this.tauriService.getAuthStatus();
      if (!authStatus.authenticated) {
        return null;
      }

      // Get rate limits from REST API
      const restResponse = await this.makeAuthenticatedRequest(
        `${this.API_BASE_URL}/rate_limit`
      );

      // Get GraphQL rate limits
      const graphqlResponse = await this.makeGraphQLRequest();

      // Debug logging
      console.log('REST API Response:', restResponse);
      console.log('GraphQL Response:', graphqlResponse);

      return {
        core: {
          limit: restResponse.rate.limit,
          remaining: restResponse.rate.remaining,
          resetAt: restResponse.rate.reset,
          used:
            restResponse.rate.used ||
            restResponse.rate.limit - restResponse.rate.remaining,
        },
        search: {
          limit: restResponse.resources.search.limit,
          remaining: restResponse.resources.search.remaining,
          resetAt: restResponse.resources.search.reset,
          used:
            restResponse.resources.search.used ||
            restResponse.resources.search.limit -
              restResponse.resources.search.remaining,
        },
        graphql: graphqlResponse.rateLimit,
      };
    } catch (error) {
      console.error('Error fetching rate limits:', error);
      throw error;
    }
  }

  /**
   * Make authenticated REST API request
   */
  private async makeAuthenticatedRequest(url: string): Promise<any> {
    const token = await this.getStoredToken();

    if (!token) {
      throw new Error('No authentication token available');
    }

    const response = await fetch(url, {
      headers: {
        Authorization: `token ${token}`,
        Accept: 'application/vnd.github.v3+json',
        'User-Agent': 'GitHubSecurityAlerts',
      },
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Make GraphQL request for rate limit info
   */
  private async makeGraphQLRequest(): Promise<any> {
    const token = await this.getStoredToken();

    if (!token) {
      throw new Error('No authentication token available');
    }

    const query = `
      query {
        viewer {
          login
        }
        rateLimit {
          limit
          cost
          remaining
          resetAt
        }
      }
    `;

    const response = await fetch(`${this.API_BASE_URL}/graphql`, {
      method: 'POST',
      headers: {
        Authorization: `token ${token}`,
        'Content-Type': 'application/json',
        'User-Agent': 'GitHubSecurityAlerts',
      },
      body: JSON.stringify({ query }),
    });

    if (!response.ok) {
      throw new Error(
        `GraphQL HTTP ${response.status}: ${response.statusText}`
      );
    }

    const result = await response.json();

    if (result.errors) {
      throw new Error(`GraphQL Error: ${result.errors[0].message}`);
    }

    return result.data;
  }

  /**
   * Get stored authentication token from Tauri backend
   */
  private async getStoredToken(): Promise<string | null> {
    try {
      return await this.tauriService.getToken();
    } catch (error) {
      console.error('Error getting token:', error);
      return null;
    }
  }

  /**
   * Calculate usage percentage
   */
  calculateUsagePercentage(used: number, limit: number): number {
    if (limit === 0) return 0;
    return Math.round((used / limit) * 100);
  }

  /**
   * Format reset time to readable string
   */
  formatResetTime(resetAt: string | number): string {
    try {
      let resetDate: Date;

      // Handle different formats: ISO string, Unix timestamp, or Unix timestamp in seconds
      if (typeof resetAt === 'number') {
        // Unix timestamp (GitHub API sometimes returns this)
        resetDate = new Date(
          resetAt > 1000000000000 ? resetAt : resetAt * 1000
        );
      } else if (typeof resetAt === 'string') {
        // Try parsing as ISO string first, then as number
        if (/^\d+$/.test(resetAt)) {
          // String containing only numbers - treat as Unix timestamp
          const timestamp = parseInt(resetAt, 10);
          resetDate = new Date(
            timestamp > 1000000000000 ? timestamp : timestamp * 1000
          );
        } else {
          // ISO string
          resetDate = new Date(resetAt);
        }
      } else {
        return 'Unknown';
      }

      // Check if the date is valid
      if (isNaN(resetDate.getTime())) {
        console.warn('Invalid reset date:', resetAt);
        return 'Unknown';
      }

      const now = new Date();
      const diffMs = resetDate.getTime() - now.getTime();

      if (diffMs <= 0) {
        return 'Now';
      }

      const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
      const diffMinutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));

      if (diffHours > 0) {
        return `${diffHours}h ${diffMinutes}m`;
      } else {
        return `${diffMinutes}m`;
      }
    } catch (error) {
      console.error('Error formatting reset time:', error, 'Input:', resetAt);
      return 'Unknown';
    }
  }
}
