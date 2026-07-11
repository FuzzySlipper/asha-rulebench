import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  computed,
  ElementRef,
  HostListener,
  inject,
  input,
  output,
  signal,
  viewChildren,
} from '@angular/core';

export interface ApplicationMenuItem {
  readonly id: string;
  readonly label: string;
  readonly disabled?: boolean;
}

export interface ApplicationMenuGroup {
  readonly id: string;
  readonly label: string;
  readonly items: readonly ApplicationMenuItem[];
}

@Component({
  selector: 'arb-application-menubar',
  imports: [],
  templateUrl: './application-menubar.component.html',
  styleUrl: './application-menubar.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ApplicationMenubarComponent {
  private readonly host = inject<ElementRef<HTMLElement>>(ElementRef);
  private readonly changeDetector = inject(ChangeDetectorRef);
  private readonly menuTriggers = viewChildren<ElementRef<HTMLButtonElement>>('menuTrigger');
  private readonly menuItems = viewChildren<ElementRef<HTMLButtonElement>>('menuItem');

  readonly groups = input.required<readonly ApplicationMenuGroup[]>();
  readonly busy = input(false);
  readonly statusMessage = input('');
  readonly itemInvoked = output<ApplicationMenuItem>();

  protected readonly visibleGroups = computed(() => this.groups().filter((group) => group.items.length > 0));
  protected readonly activeGroupIndex = signal(0);
  protected readonly openGroupIndex = signal<number | null>(null);
  private typeahead = '';
  private typeaheadAt = 0;

  protected triggerId(group: ApplicationMenuGroup): string {
    return `arb-menu-trigger-${group.id}`;
  }

  protected menuId(group: ApplicationMenuGroup): string {
    return `arb-menu-${group.id}`;
  }

  protected onTriggerFocus(index: number): void {
    this.activeGroupIndex.set(index);
  }

  protected onTriggerClick(index: number): void {
    if (this.openGroupIndex() === index) {
      this.closeMenu(true);
      return;
    }
    this.openMenu(index, 'first');
  }

  protected onTriggerKeydown(event: KeyboardEvent, index: number): void {
    switch (event.key) {
      case 'ArrowLeft':
        event.preventDefault();
        this.focusGroup(index - 1);
        return;
      case 'ArrowRight':
        event.preventDefault();
        this.focusGroup(index + 1);
        return;
      case 'Home':
        event.preventDefault();
        this.focusGroup(0);
        return;
      case 'End':
        event.preventDefault();
        this.focusGroup(this.visibleGroups().length - 1);
        return;
      case 'ArrowDown':
      case 'Enter':
      case ' ':
        event.preventDefault();
        this.openMenu(index, 'first');
        return;
      case 'ArrowUp':
        event.preventDefault();
        this.openMenu(index, 'last');
        return;
      case 'Escape':
        if (this.openGroupIndex() !== null) {
          event.preventDefault();
          this.closeMenu(true);
        }
        return;
      default:
        this.focusGroupByTypeahead(event);
    }
  }

  protected onItemClick(item: ApplicationMenuItem): void {
    this.invokeItem(item);
  }

  protected onItemKeydown(event: KeyboardEvent, item: ApplicationMenuItem, index: number): void {
    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        this.focusItemFrom(index, 1);
        return;
      case 'ArrowUp':
        event.preventDefault();
        this.focusItemFrom(index, -1);
        return;
      case 'Home':
        event.preventDefault();
        this.focusBoundaryItem('first');
        return;
      case 'End':
        event.preventDefault();
        this.focusBoundaryItem('last');
        return;
      case 'ArrowRight':
        event.preventDefault();
        this.openAdjacentMenu(1);
        return;
      case 'ArrowLeft':
        event.preventDefault();
        this.openAdjacentMenu(-1);
        return;
      case 'Enter':
      case ' ':
        event.preventDefault();
        this.invokeItem(item);
        return;
      case 'Escape':
        event.preventDefault();
        this.closeMenu(true);
        return;
      case 'Tab':
        this.closeMenu(false);
        return;
      default:
        this.focusItemByTypeahead(event);
    }
  }

  @HostListener('document:pointerdown', ['$event'])
  protected onDocumentPointerDown(event: PointerEvent): void {
    if (!this.host.nativeElement.contains(event.target as Node)) this.closeMenu(false);
  }

  @HostListener('document:focusin', ['$event'])
  protected onDocumentFocusIn(event: FocusEvent): void {
    if (!this.host.nativeElement.contains(event.target as Node)) this.closeMenu(false);
  }

  private openMenu(groupIndex: number, boundary: 'first' | 'last'): void {
    if (this.visibleGroups().length === 0) return;
    const normalizedIndex = this.normalizeIndex(groupIndex, this.visibleGroups().length);
    this.activeGroupIndex.set(normalizedIndex);
    this.openGroupIndex.set(normalizedIndex);
    this.changeDetector.detectChanges();
    this.focusBoundaryItem(boundary);
  }

  private openAdjacentMenu(offset: -1 | 1): void {
    const openIndex = this.openGroupIndex();
    if (openIndex !== null) this.openMenu(openIndex + offset, 'first');
  }

  private closeMenu(restoreTriggerFocus: boolean): void {
    const openIndex = this.openGroupIndex();
    if (openIndex === null) return;
    this.openGroupIndex.set(null);
    this.resetTypeahead();
    this.changeDetector.detectChanges();
    if (restoreTriggerFocus) this.menuTriggers()[openIndex]?.nativeElement.focus();
  }

  private focusGroup(index: number): void {
    const groupCount = this.visibleGroups().length;
    if (groupCount === 0) return;
    const normalizedIndex = this.normalizeIndex(index, groupCount);
    this.activeGroupIndex.set(normalizedIndex);
    this.menuTriggers()[normalizedIndex]?.nativeElement.focus();
  }

  private focusBoundaryItem(boundary: 'first' | 'last'): void {
    const items = this.currentItems();
    if (items.length === 0) return;
    const startIndex = boundary === 'first' ? 0 : items.length - 1;
    const offset = boundary === 'first' ? 1 : -1;
    const index = this.findEnabledItem(items, startIndex, offset);
    if (index !== null) this.focusItem(index);
  }

  private focusItemFrom(index: number, offset: -1 | 1): void {
    const items = this.currentItems();
    const nextIndex = this.findEnabledItem(items, index + offset, offset);
    if (nextIndex !== null) this.focusItem(nextIndex);
  }

  private focusItem(index: number): void {
    this.menuItems()[index]?.nativeElement.focus();
  }

  private invokeItem(item: ApplicationMenuItem): void {
    if (item.disabled === true) return;
    this.itemInvoked.emit(item);
    this.closeMenu(true);
  }

  private focusGroupByTypeahead(event: KeyboardEvent): void {
    const query = this.nextTypeaheadQuery(event);
    if (query === null) return;
    const matchIndex = this.visibleGroups().findIndex((group) => group.label.toLocaleLowerCase().startsWith(query));
    if (matchIndex >= 0) this.focusGroup(matchIndex);
  }

  private focusItemByTypeahead(event: KeyboardEvent): void {
    const query = this.nextTypeaheadQuery(event);
    if (query === null) return;
    const matchIndex = this.currentItems().findIndex(
      (item) => item.disabled !== true && item.label.toLocaleLowerCase().startsWith(query),
    );
    if (matchIndex >= 0) this.focusItem(matchIndex);
  }

  private nextTypeaheadQuery(event: KeyboardEvent): string | null {
    if (event.key.length !== 1 || event.altKey || event.ctrlKey || event.metaKey) return null;
    const now = Date.now();
    this.typeahead = now - this.typeaheadAt > 700 ? event.key : this.typeahead + event.key;
    this.typeaheadAt = now;
    return this.typeahead.toLocaleLowerCase();
  }

  private resetTypeahead(): void {
    this.typeahead = '';
    this.typeaheadAt = 0;
  }

  private currentItems(): readonly ApplicationMenuItem[] {
    const groupIndex = this.openGroupIndex();
    return groupIndex === null ? [] : this.visibleGroups()[groupIndex]?.items ?? [];
  }

  private findEnabledItem(items: readonly ApplicationMenuItem[], startIndex: number, offset: -1 | 1): number | null {
    for (let step = 0; step < items.length; step += 1) {
      const candidate = this.normalizeIndex(startIndex + step * offset, items.length);
      const item = items[candidate];
      if (item !== undefined && item.disabled !== true) return candidate;
    }
    return null;
  }

  private normalizeIndex(index: number, length: number): number {
    return ((index % length) + length) % length;
  }
}
