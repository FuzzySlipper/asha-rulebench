import { Component, computed, inject } from "@angular/core";
import type { OnInit } from "@angular/core";
import { ContentStore } from "@asha-rulebench/store";

@Component({
  selector: "arb-content-packs-dialog-content",
  standalone: true,
  styles: [
    `
      :host {
        display: block;
      }
      .dialog-content {
        display: grid;
        gap: 12px;
        padding: 16px 44px;
      }
      .heading,
      .toolbar,
      .pack-list,
      .summary {
        align-items: center;
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
      }
      h2,
      h3,
      p {
        margin: 0;
      }
      h2 {
        font-size: 0.95rem;
      }
      h3 {
        font-size: 0.88rem;
      }
      button {
        background: var(--arb-surface);
        border: 1px solid var(--arb-border);
        border-radius: 6px;
        color: var(--arb-text);
        cursor: pointer;
        min-height: 34px;
        padding: 7px 10px;
      }
      button[aria-pressed="true"] {
        border-color: var(--arb-accent);
        box-shadow: inset 3px 0 0 var(--arb-accent);
      }
      button:disabled {
        cursor: default;
        opacity: 0.55;
      }
      .pack-button {
        display: grid;
        gap: 3px;
        min-width: 210px;
        text-align: left;
      }
      .meta,
      .state,
      .fingerprint,
      .path {
        color: var(--arb-muted);
        font-size: 0.82rem;
        overflow-wrap: anywhere;
      }
      .detail {
        border-left: 3px solid var(--arb-border);
        display: grid;
        gap: 9px;
        padding-left: 12px;
      }
      .diagnostics {
        display: grid;
        gap: 7px;
        list-style: none;
        margin: 0;
        padding: 0;
      }
      .diagnostic {
        display: grid;
        gap: 2px;
        grid-template-columns: minmax(72px, auto) minmax(170px, 0.45fr) minmax(
            220px,
            1fr
          );
      }
      .error {
        color: #9b2c2c;
        font-weight: 700;
      }
      .warning {
        color: #8a5a00;
        font-weight: 700;
      }
      @media (max-width: 760px) {
        .dialog-content {
          padding: 14px 16px;
        }
        .diagnostic {
          grid-template-columns: 1fr;
        }
      }
    `,
  ],
  template: `
    <section class="dialog-content" aria-label="Content pack controls">
      <div class="heading">
        <h2>Content Packs</h2>
        <div class="toolbar">
          <button
            type="button"
            [disabled]="imports().kind === 'loading'"
            (click)="reloadImports()"
          >
            Load imports
          </button>
          <button
            type="button"
            [disabled]="validation().kind === 'loading'"
            (click)="loadValidation()"
          >
            Validate active scenario
          </button>
        </div>
      </div>

      @switch (imports().kind) {
        @case ("loading") {
          <p class="state">Loading content imports</p>
        }
        @case ("error") {
          <p class="state" role="alert">{{ imports().error.message }}</p>
        }
        @case ("idle") {
          <p class="state">Content imports idle</p>
        }
        @case ("data") {
          <div class="pack-list" aria-label="Imported content packs">
            @for (item of imports().value; track item.exampleId) {
              <button
                class="pack-button"
                type="button"
                [attr.aria-pressed]="selectedImportId() === item.exampleId"
                (click)="selectImport(item.exampleId)"
              >
                <strong>{{ item.packLabel }}</strong>
                <span class="meta"
                  >{{ item.statusLabel }} · {{ item.errorCount }} errors ·
                  {{ item.warningCount }} warnings</span
                >
              </button>
            }
          </div>
          @if (selectedImport(); as item) {
            <section class="detail" aria-label="Selected content pack review">
              <div class="summary">
                <h3>{{ item.packLabel }}</h3>
                <span>{{ item.statusLabel }}</span>
              </div>
              <p class="fingerprint">{{ item.fingerprintLabel }}</p>
              @if (item.diagnostics.length === 0) {
                <p class="state">No import diagnostics</p>
              }
              <ul class="diagnostics">
                @for (
                  diagnostic of item.diagnostics;
                  track diagnostic.code + diagnostic.locationLabel
                ) {
                  <li class="diagnostic">
                    <span
                      [class.error]="diagnostic.severityLabel === 'Error'"
                      [class.warning]="diagnostic.severityLabel === 'Warning'"
                      >{{ diagnostic.severityLabel }}</span
                    >
                    <code>{{ diagnostic.code }}</code>
                    <span
                      ><span class="path">{{ diagnostic.locationLabel }}</span
                      ><br />{{ diagnostic.message }}</span
                    >
                  </li>
                }
              </ul>
            </section>
          }
        }
      }

      @switch (validation().kind) {
        @case ("loading") {
          <p class="state">Validating active scenario</p>
        }
        @case ("error") {
          <p class="state" role="alert">{{ validation().error.message }}</p>
        }
        @case ("data") {
          <section class="detail" aria-label="Content validation review">
            <div class="summary">
              <h3>{{ validation().value.scenarioTitle }}</h3>
              <span>{{ validation().value.statusLabel }}</span>
            </div>
            <p class="meta">
              {{ validation().value.errorCount }} errors ·
              {{ validation().value.warningCount }} warnings
            </p>
            @if (validation().value.diagnostics.length === 0) {
              <p class="state">No validation diagnostics</p>
            }
            <ul class="diagnostics">
              @for (
                diagnostic of validation().value.diagnostics;
                track diagnostic.code + diagnostic.sourceLabel
              ) {
                <li class="diagnostic">
                  <span>{{ diagnostic.severityLabel }}</span
                  ><code>{{ diagnostic.code }}</code
                  ><span
                    ><span class="path">{{ diagnostic.sourceLabel }}</span
                    ><br />{{ diagnostic.message }}</span
                  >
                </li>
              }
            </ul>
          </section>
        }
      }
    </section>
  `,
})
export class ContentPacksDialogContentComponent implements OnInit {
  private readonly store = inject(ContentStore);
  protected readonly imports = computed(() => this.store.imports());
  protected readonly validation = computed(() => this.store.validation());
  protected readonly selectedImportId = computed(() =>
    this.store.selectedImportId(),
  );
  protected readonly selectedImport = computed(() => {
    const imports = this.imports();
    return imports.kind === "data"
      ? (imports.value.find(
          (item) => item.exampleId === this.selectedImportId(),
        ) ?? null)
      : null;
  });

  ngOnInit(): void {
    void this.initialize();
  }
  protected reloadImports(): void {
    void this.store.loadImportExamples();
  }
  protected loadValidation(): void {
    void this.store.loadValidation();
  }
  protected selectImport(exampleId: string): void {
    this.store.selectImport(exampleId);
  }
  private async initialize(): Promise<void> {
    await this.store.loadImportExamples();
    await this.store.loadValidation();
  }
}
