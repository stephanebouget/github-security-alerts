import { Component, EventEmitter, Input, Output } from '@angular/core';
import { RepoInfo, OwnerInfo } from '../../../core/services';

export interface OwnerAccordion {
  owner: OwnerInfo;
  expanded: boolean;
  loading: boolean;
  loaded: boolean;
  repos: RepoInfo[];
}

@Component({
  selector: 'app-repos-panel',
  templateUrl: './repos-panel.component.html',
  styleUrls: ['./repos-panel.component.scss'],
  standalone: false,
})
export class ReposPanelComponent {
  @Input() owners: OwnerAccordion[] = [];
  @Input() ownersLoading = false;
  @Input() searchQuery = '';

  @Output() searchQueryChange = new EventEmitter<string>();
  @Output() toggleOwner = new EventEmitter<OwnerAccordion>();
  @Output() toggleRepo = new EventEmitter<RepoInfo>();
  @Output() selectAllForOwner = new EventEmitter<OwnerAccordion>();
  @Output() selectNoneForOwner = new EventEmitter<OwnerAccordion>();
  @Output() done = new EventEmitter<void>();

  get selectedCount(): number {
    return this.owners.reduce(
      (sum, o) => sum + o.repos.filter((r) => r.selected).length,
      0
    );
  }

  getSelectedCountForOwner(ownerAccordion: OwnerAccordion): number {
    return ownerAccordion.repos.filter((r) => r.selected).length;
  }

  getFilteredRepos(ownerAccordion: OwnerAccordion): RepoInfo[] {
    if (!this.searchQuery.trim()) {
      return ownerAccordion.repos;
    }
    const query = this.searchQuery.toLowerCase();
    return ownerAccordion.repos.filter(
      (r) =>
        r.name.toLowerCase().includes(query) ||
        r.owner.toLowerCase().includes(query)
    );
  }

  onSearchChange(value: string): void {
    this.searchQueryChange.emit(value);
  }

  onToggleOwner(ownerAccordion: OwnerAccordion): void {
    this.toggleOwner.emit(ownerAccordion);
  }

  onToggleRepo(repo: RepoInfo, event: Event): void {
    event.stopPropagation();
    this.toggleRepo.emit(repo);
  }

  onSelectAll(ownerAccordion: OwnerAccordion, event: Event): void {
    event.stopPropagation();
    this.selectAllForOwner.emit(ownerAccordion);
  }

  onSelectNone(ownerAccordion: OwnerAccordion, event: Event): void {
    event.stopPropagation();
    this.selectNoneForOwner.emit(ownerAccordion);
  }

  onDone(): void {
    this.done.emit();
  }
}
