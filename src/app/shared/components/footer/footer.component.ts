import { Component, Input } from '@angular/core';
import { AlertsResponse } from '../../../core/services';

@Component({
  selector: 'app-footer',
  templateUrl: './footer.component.html',
  styleUrls: ['./footer.component.scss'],
  standalone: false,
})
export class FooterComponent {
  @Input() alerts: AlertsResponse | null = null;
  @Input() authenticated = false;

  getLastUpdate(): string {
    return new Date().toLocaleTimeString();
  }

  openGitHub(): void {
    window.open(
      'https://github.com/stephanebouget/github-security-alerts',
      '_blank'
    );
  }
}
