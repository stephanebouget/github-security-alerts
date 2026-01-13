import {
  Component,
  EventEmitter,
  Input,
  Output,
  OnInit,
  OnChanges,
} from '@angular/core';

@Component({
  selector: 'app-login-panel',
  templateUrl: './login-panel.component.html',
  styleUrls: ['./login-panel.component.scss'],
  standalone: false,
})
export class LoginPanelComponent implements OnInit, OnChanges {
  @Input() authLoading = false;
  @Input() oauthLoading = false;
  @Input() error = '';
  @Input() existingToken: string | null = null;

  @Output() login = new EventEmitter<string>();
  @Output() openTokenPage = new EventEmitter<void>();
  @Output() startOAuth = new EventEmitter<void>();

  tokenInput = '';
  private actualToken = '';
  showingMaskedToken = false;

  ngOnInit(): void {
    this.updateTokenDisplay();
  }

  ngOnChanges(): void {
    this.updateTokenDisplay();
  }

  private updateTokenDisplay(): void {
    if (this.existingToken) {
      // Show masked token with stars
      this.tokenInput = '•'.repeat(Math.min(this.existingToken.length, 32));
      this.actualToken = this.existingToken;
      this.showingMaskedToken = true;
    } else {
      this.tokenInput = '';
      this.actualToken = '';
      this.showingMaskedToken = false;
    }
  }

  onTokenFocus(): void {
    if (this.showingMaskedToken) {
      // Clear the masked display when user wants to enter a new token
      this.tokenInput = '';
      this.actualToken = '';
      this.showingMaskedToken = false;
    }
  }

  onTokenInput(): void {
    if (this.showingMaskedToken) {
      this.showingMaskedToken = false;
      this.actualToken = '';
    }
    this.actualToken = this.tokenInput;
  }

  onLogin(): void {
    const tokenToEmit = this.showingMaskedToken
      ? this.existingToken
      : this.tokenInput;
    if (tokenToEmit) {
      this.login.emit(tokenToEmit);
    }
  }

  onOpenTokenPage(): void {
    this.openTokenPage.emit();
  }

  onStartOAuth(): void {
    this.startOAuth.emit();
  }
}
