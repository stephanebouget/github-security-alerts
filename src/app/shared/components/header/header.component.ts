import { Component, EventEmitter, Input, Output } from '@angular/core';

type ViewMode = 'alerts' | 'repos' | 'settings';

@Component({
  selector: 'app-header',
  templateUrl: './header.component.html',
  styleUrls: ['./header.component.scss'],
  standalone: false,
})
export class HeaderComponent {
  @Input() authenticated = false;
  @Input() alertsLoading = false;
  @Input() currentView: ViewMode = 'alerts';

  @Output() refresh = new EventEmitter<void>();
  @Output() viewChange = new EventEmitter<ViewMode>();

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
