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

import {
  Component,
  EventEmitter,
  Input,
  Output,
  OnInit,
  OnDestroy,
} from '@angular/core';
import { UpdateService } from '../../../core/services';

type ViewMode = 'alerts' | 'repos' | 'settings';

@Component({
  selector: 'app-header',
  templateUrl: './header.component.html',
  styleUrls: ['./header.component.scss'],
  standalone: false,
})
export class HeaderComponent implements OnInit, OnDestroy {
  @Input() authenticated = false;
  @Input() alertsLoading = false;
  @Input() currentView: ViewMode = 'alerts';

  @Output() refresh = new EventEmitter<void>();
  @Output() viewChange = new EventEmitter<ViewMode>();

  isUpdating = false;

  get updateAvailable(): boolean {
    return this.updateService.updateAvailable;
  }

  constructor(private updateService: UpdateService) {}

  ngOnInit(): void {}

  ngOnDestroy(): void {
    // Nothing to clean up
  }

  async onUpdateClick(): Promise<void> {
    if (this.isUpdating) return;

    this.isUpdating = true;
    try {
      await this.updateService.installUpdate();
    } catch (error) {
      console.error('Update failed:', error);
    } finally {
      this.isUpdating = false;
    }
  }

  onRefresh(): void {
    this.refresh.emit();
  }

  showAlerts(): void {
    this.viewChange.emit('alerts');
  }

  showRepos(): void {
    this.viewChange.emit('repos');
  }

  showSettings(): void {
    this.viewChange.emit('settings');
  }

  async minimizeApp(): Promise<void> {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('minimize_window');
    } catch (error) {
      console.error('Failed to minimize window:', error);
    }
  }
}
