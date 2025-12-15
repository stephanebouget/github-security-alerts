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

  updateAvailable = false;
  isUpdating = false;
  private updateCheckInterval?: number;

  constructor(private updateService: UpdateService) {}

  ngOnInit(): void {
    // Start checking for updates periodically
    this.checkUpdateStatus();
    this.updateCheckInterval = window.setInterval(() => {
      this.checkUpdateStatus();
    }, 60000); // Check every minute for UI updates
  }

  ngOnDestroy(): void {
    if (this.updateCheckInterval) {
      clearInterval(this.updateCheckInterval);
    }
  }

  private async checkUpdateStatus(): Promise<void> {
    if (this.updateService.isTauri) {
      await this.updateService.checkForUpdates();
      this.updateAvailable = this.updateService.updateAvailable;
    }
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
}
