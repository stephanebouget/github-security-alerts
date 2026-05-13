/*
 * Software Name : GitHub Security Alerts
 * SPDX-FileCopyrightText: Copyright (c) Orange
 * SPDX-License-Identifier: MIT
 *
 * This software is distributed under the MIT,
 * see the "LICENSE.txt" file for more details or https://opensource.org/license/mit
 *
 * Software description: A modern desktop application that monitors security vulnerabilities across your GitHub repositories in real-time.
 */

import { Component, EventEmitter, Input, Output } from '@angular/core';
import {
  AlertsResponse,
  RepoAlerts,
  TauriService,
} from '../../../core/services';

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
    return this.alerts.total_alerts === 0
      ? 'ti ti-check'
      : 'ti ti-alert-triangle';
  }

  onShowRepos(): void {
    this.showRepos.emit();
  }

  openRepoOnGitHub(repoFullName: string): void {
    const url = `https://github.com/${repoFullName}/security/dependabot`;
    this.tauriService.openExternalLink(url);
  }

  prodAlerts(repo: RepoAlerts): number {
    return repo.alerts - repo.dev_alerts;
  }

  totalProdAlerts(): number {
    if (!this.alerts) return 0;
    return this.alerts.repos.reduce(
      (sum, repo) => sum + this.prodAlerts(repo),
      0,
    );
  }

  prodBadgeTitle(repo: RepoAlerts): string | null {
    if (repo.error) return repo.error;
    if (!repo.dependabot_enabled) return 'Dependabot not enabled';
    const prod = this.prodAlerts(repo);
    if (prod > 0)
      return `${prod} alert(s) from dependencies — high runtime risk`;
    return null;
  }
}
