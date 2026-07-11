import { ChangeDetectionStrategy, Component, input } from "@angular/core";

@Component({
  selector: "arb-workbench-panel",
  imports: [],
  templateUrl: "./workbench-panel.component.html",
  styleUrl: "./workbench-panel.component.css",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WorkbenchPanelComponent {
  readonly panelNumber = input.required<number>();
  readonly panelTitle = input.required<string>();
  readonly compact = input(false);
}
