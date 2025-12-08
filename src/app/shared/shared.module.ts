import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { TranslateModule } from '@ngx-translate/core';

import {
  PageNotFoundComponent,
  HeaderComponent,
  LoginPanelComponent,
  SettingsPanelComponent,
  ReposPanelComponent,
  AlertsListComponent,
  FooterComponent,
} from './components/';
import { WebviewDirective } from './directives/';
import { FormsModule } from '@angular/forms';

@NgModule({
  declarations: [
    PageNotFoundComponent,
    WebviewDirective,
    HeaderComponent,
    LoginPanelComponent,
    SettingsPanelComponent,
    ReposPanelComponent,
    AlertsListComponent,
    FooterComponent,
  ],
  imports: [CommonModule, TranslateModule, FormsModule],
  exports: [
    TranslateModule,
    WebviewDirective,
    FormsModule,
    HeaderComponent,
    LoginPanelComponent,
    SettingsPanelComponent,
    ReposPanelComponent,
    AlertsListComponent,
    FooterComponent,
  ]
})
export class SharedModule {}
