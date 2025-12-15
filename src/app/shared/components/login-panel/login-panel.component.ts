import { Component, EventEmitter, Input, Output } from '@angular/core';

@Component({
  selector: 'app-login-panel',
  templateUrl: './login-panel.component.html',
  styleUrls: ['./login-panel.component.scss'],
  standalone: false,
})
export class LoginPanelComponent {
  @Input() authLoading = false;
  @Input() oauthLoading = false;
  @Input() error = '';

  @Output() login = new EventEmitter<string>();
  @Output() openTokenPage = new EventEmitter<void>();
  @Output() startOAuth = new EventEmitter<void>();

  tokenInput = '';

  onLogin(): void {
    this.login.emit(this.tokenInput);
  }

  onOpenTokenPage(): void {
    this.openTokenPage.emit();
  }

  onStartOAuth(): void {
    this.startOAuth.emit();
  }
}
