import {
  ChangeDetectionStrategy,
  Component,
  computed,
  signal,
  viewChild,
} from "@angular/core";
import {
  ApplicationMenubarComponent,
  type ApplicationMenuGroup,
  type ApplicationMenuItem,
  WorkbenchPanelComponent,
} from "@asha-rulebench/components";

type EvidenceTab = "combat" | "events" | "trace" | "audit";

@Component({
  selector: "arb-workbench-shell",
  imports: [ApplicationMenubarComponent, WorkbenchPanelComponent],
  templateUrl: "./workbench-shell.component.html",
  styleUrl: "./workbench-shell.component.css",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WorkbenchShellComponent {
  private readonly gridPanel = viewChild.required<WorkbenchPanelComponent>("gridPanel");
  private readonly initiativePanel = viewChild.required<WorkbenchPanelComponent>("initiativePanel");
  private readonly statusPanel = viewChild.required<WorkbenchPanelComponent>("statusPanel");
  private readonly logPanel = viewChild.required<WorkbenchPanelComponent>("logPanel");
  private readonly actionsPanel = viewChild.required<WorkbenchPanelComponent>("actionsPanel");
  private readonly unitsPanel = viewChild.required<WorkbenchPanelComponent>("unitsPanel");

  protected readonly gridCells = Array.from({ length: 96 });
  protected readonly menuStatus = signal("");
  protected readonly menuGroups: readonly ApplicationMenuGroup[] = [
    { id: "file", label: "File", items: [] },
    {
      id: "scenario",
      label: "Scenario",
      items: [
        { id: "focus-grid", label: "Combat grid" },
        { id: "focus-initiative", label: "Initiative" },
      ],
    },
    {
      id: "run",
      label: "Run",
      items: [
        { id: "focus-status", label: "Turn status" },
        { id: "focus-actions", label: "Available actions" },
        { id: "focus-current-actor", label: "Current actor", disabled: true },
      ],
    },
    { id: "replay", label: "Replay", items: [{ id: "focus-log", label: "Evidence log" }] },
    { id: "view", label: "View", items: [{ id: "focus-units", label: "Active units" }] },
    { id: "preferences", label: "Preferences", items: [] },
  ];
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

  protected invokeMenuItem(item: ApplicationMenuItem): void {
    const target = this.panelForCommand(item.id);
    if (target === null) return;
    target.focus();
    this.menuStatus.set(`Focused ${item.label}`);
  }

  private panelForCommand(commandId: string): WorkbenchPanelComponent | null {
    switch (commandId) {
      case "focus-grid":
        return this.gridPanel();
      case "focus-initiative":
        return this.initiativePanel();
      case "focus-status":
        return this.statusPanel();
      case "focus-log":
        return this.logPanel();
      case "focus-actions":
        return this.actionsPanel();
      case "focus-units":
        return this.unitsPanel();
      default:
        return null;
    }
  }
}
