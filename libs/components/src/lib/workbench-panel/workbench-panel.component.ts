import { ChangeDetectionStrategy, Component, input, viewChild } from "@angular/core";
import type { ElementRef } from "@angular/core";

@Component({
  selector: "arb-workbench-panel",
  imports: [],
  templateUrl: "./workbench-panel.component.html",
  styleUrl: "./workbench-panel.component.css",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WorkbenchPanelComponent {
  private readonly panel = viewChild.required<ElementRef<HTMLElement>>("panel");
  readonly panelNumber = input.required<number>();
  readonly panelTitle = input.required<string>();
  readonly compact = input(false);
  readonly overlayTools = input(false);

  focus(): void {
    this.panel().nativeElement.focus();
  }
}
