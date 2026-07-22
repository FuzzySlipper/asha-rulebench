import {
  ChangeDetectionStrategy,
  Component,
  effect,
  input,
  output,
  viewChild,
} from '@angular/core';
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
  private readonly closeButton =
    viewChild<ElementRef<HTMLButtonElement>>('closeButton');
  private restoreFocusTo: HTMLElement | null = null;

  readonly dialogId = input.required<string>();
  readonly dialogTitle = input.required<string>();
  readonly dialogDescription = input.required<string>();
  readonly open = input(false);
  readonly closeRequested = output<void>();

  constructor() {
    effect(() => {
      const dialog = this.dialog()?.nativeElement;
      if (dialog === undefined) return;
      if (this.open() && !dialog.open) {
        this.restoreFocusTo = focusableActiveElement(dialog);
        dialog.showModal();
        this.closeButton()?.nativeElement.focus();
      }
      if (!this.open() && dialog.open) {
        dialog.close();
        if (this.restoreFocusTo?.isConnected === true) {
          this.restoreFocusTo.focus();
        }
        this.restoreFocusTo = null;
      }
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

function focusableActiveElement(dialog: HTMLDialogElement): HTMLElement | null {
  const activeElement = dialog.ownerDocument.activeElement;
  const htmlElement = dialog.ownerDocument.defaultView?.HTMLElement;
  return htmlElement !== undefined && activeElement instanceof htmlElement
    ? activeElement
    : null;
}
