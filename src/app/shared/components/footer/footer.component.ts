import { Component, Input } from '@angular/core';
import { AlertsResponse, TauriService } from '../../../core/services';

@Component({
  selector: 'app-footer',
  templateUrl: './footer.component.html',
  styleUrls: ['./footer.component.scss'],
  standalone: false,
})
export class FooterComponent {
  @Input() alerts: AlertsResponse | null = null;
  @Input() authenticated = false;

  constructor(private tauriService: TauriService) {}

  getLastUpdate(): string {
    return new Date().toLocaleTimeString();
  }

  openGitHub(): void {
    const url = `https://github.com/stephanebouget/github-security-alerts`;
    this.tauriService.openExternalLink(url);
  }
}
