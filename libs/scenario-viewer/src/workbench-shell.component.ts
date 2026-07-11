import {
  ChangeDetectionStrategy,
  Component,
  computed,
  signal,
} from "@angular/core";
import { WorkbenchPanelComponent } from "@asha-rulebench/components";

type EvidenceTab = "combat" | "events" | "trace" | "audit";

@Component({
  selector: "arb-workbench-shell",
  imports: [WorkbenchPanelComponent],
  templateUrl: "./workbench-shell.component.html",
  styleUrl: "./workbench-shell.component.css",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WorkbenchShellComponent {
  protected readonly gridCells = Array.from({ length: 96 });
  protected readonly selectedEvidenceTab = signal<EvidenceTab>("combat");
  protected readonly evidenceTabLabel = computed(() => {
    switch (this.selectedEvidenceTab()) {
      case "combat":
        return "Combat log";
      case "events":
        return "Accepted DomainEvents";
      case "trace":
        return "Rule resolution trace";
      case "audit":
        return "Command audit";
    }
  });

  protected selectEvidenceTab(tab: EvidenceTab): void {
    this.selectedEvidenceTab.set(tab);
  }
}
