import { ChangeDetectionStrategy, Component, effect, input, output, viewChild } from '@angular/core';
import type { ElementRef } from '@angular/core';

@Component({
  selector: 'arb-application-dialog',
  imports: [],
  templateUrl: './application-dialog.component.html',
  styleUrl: './application-dialog.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ApplicationDialogComponent {
  private readonly dialog = viewChild<ElementRef<HTMLDialogElement>>('dialog');

  readonly dialogId = input.required<string>();
  readonly dialogTitle = input.required<string>();
  readonly dialogDescription = input.required<string>();
  readonly open = input(false);
  readonly closeRequested = output<void>();

  constructor() {
    effect(() => {
      const dialog = this.dialog()?.nativeElement;
      if (dialog === undefined) return;
      if (this.open() && !dialog.open) dialog.showModal();
      if (!this.open() && dialog.open) dialog.close();
    });
  }

  protected titleId(): string {
    return `${this.dialogId()}-title`;
  }

  protected descriptionId(): string {
    return `${this.dialogId()}-description`;
  }

  protected requestClose(): void {
    this.closeRequested.emit();
  }

  protected onCancel(event: Event): void {
    event.preventDefault();
    this.requestClose();
  }
}
