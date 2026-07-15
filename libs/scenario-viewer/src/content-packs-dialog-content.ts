import { Component, InjectionToken, computed, inject } from "@angular/core";
import type { OnInit } from "@angular/core";
import {
  browserTextFileInput,
  type TextFileInputPort,
} from "@asha-rulebench/platform";
import type { RulebenchContentPackReferenceDto } from "@asha-rulebench/protocol";
import { ContentWorkbenchStore } from "@asha-rulebench/store";

const TEXT_FILE_INPUT = new InjectionToken<TextFileInputPort>("TEXT_FILE_INPUT", {
  factory: browserTextFileInput,
});

@Component({
  selector: "arb-content-packs-dialog-content",
  standalone: true,
  styles: [
    `
      :host { display: block; }
      .dialog-content { display: grid; gap: 16px; padding: 18px 44px 28px; }
      .heading, .toolbar, .summary, .actions { align-items: center; display: flex; flex-wrap: wrap; gap: 8px; }
      .heading { justify-content: space-between; }
      h2, h3, h4, p { margin: 0; }
      h2 { font-size: 1rem; }
      h3 { font-size: 0.92rem; }
      h4 { font-size: 0.82rem; letter-spacing: 0.04em; text-transform: uppercase; }
      button, .file-label {
        background: var(--arb-surface); border: 1px solid var(--arb-border); border-radius: 6px;
        color: var(--arb-text); cursor: pointer; min-height: 34px; padding: 7px 10px;
      }
      button[aria-pressed="true"] { border-color: var(--arb-accent); box-shadow: inset 3px 0 0 var(--arb-accent); }
      button:disabled { cursor: default; opacity: 0.55; }
      .file-label { align-items: center; display: inline-flex; }
      .file-label input { inline-size: 1px; opacity: 0; position: absolute; }
      .state, .meta, .fingerprint, .path { color: var(--arb-muted); font-size: 0.82rem; overflow-wrap: anywhere; }
      .workspace { display: grid; gap: 14px; grid-template-columns: minmax(210px, 0.34fr) minmax(0, 1fr); }
      .pack-list, .detail, .panel, .audit { display: grid; gap: 9px; }
      .pack-list { align-content: start; }
      .pack-button { display: grid; gap: 3px; text-align: left; }
      .detail, .panel { border-left: 3px solid var(--arb-border); padding-left: 12px; }
      .active { color: var(--arb-accent); font-weight: 700; }
      .diagnostics, .definition-list, .dependency-list, .audit-list {
        display: grid; gap: 7px; list-style: none; margin: 0; padding: 0;
      }
      .diagnostic { display: grid; gap: 4px; grid-template-columns: minmax(65px, auto) minmax(160px, 0.4fr) minmax(220px, 1fr); }
      .error { color: #9b2c2c; font-weight: 700; }
      .warning { color: #8a5a00; font-weight: 700; }
      .definition-list { grid-template-columns: repeat(auto-fit, minmax(170px, 1fr)); }
      .definition-list li, .audit-list li { background: color-mix(in srgb, var(--arb-surface) 88%, transparent); border-radius: 5px; padding: 7px 9px; }
      pre { background: #10141c; border-radius: 6px; color: #e8eef8; margin: 0; max-height: 280px; overflow: auto; padding: 10px; white-space: pre-wrap; }
      details summary { cursor: pointer; }
      @media (max-width: 760px) {
        .dialog-content { padding: 14px 16px 24px; }
        .workspace, .diagnostic { grid-template-columns: 1fr; }
        .toolbar > *, .actions > * { flex: 1 1 auto; text-align: center; }
      }
    `,
  ],
  template: `
    <section class="dialog-content" aria-label="Live authored content workspace">
      <div class="heading">
        <div>
          <h2>Live Authored Content</h2>
          <p class="state">Rust validates, canonicalizes, persists, and activates exact pack sets.</p>
        </div>
        <button type="button" [disabled]="workspace().kind === 'loading'" (click)="reload()">Refresh host</button>
      </div>

      <section class="panel" aria-label="Import authored content">
        <div class="toolbar">
          <label class="file-label">
            Choose JSON pack
            <input type="file" accept="application/json,.json" (change)="stageFile($event)" />
          </label>
          <button type="button" [disabled]="stagedPayload() === null || importAttempt().kind === 'loading'" (click)="importPack(false)">Import</button>
          <button type="button" [disabled]="stagedPayload() === null || diff().kind === 'loading'" (click)="comparePack()">Compare replacement</button>
          @if (diff().kind === "data") {
            <button type="button" [disabled]="importAttempt().kind === 'loading'" (click)="importPack(true)">Confirm replacement</button>
          }
        </div>
        @if (stagedPayload() !== null) { <p class="state">Authored payload staged locally; semantic validation has not run in TypeScript.</p> }
        @switch (diff().kind) {
          @case ("loading") { <p class="state">Rust is canonicalizing and comparing the candidate.</p> }
          @case ("error") { <p class="state" role="alert">{{ diff().error.message }}</p> }
          @case ("data") {
            <div class="summary"><strong>{{ diff().value.summaryLabel }}</strong><span class="meta">{{ diff().value.beforeLabel }} → {{ diff().value.afterLabel }}</span></div>
            @if (diff().value.metadataChanges.length > 0) { <p class="meta">Metadata: {{ diff().value.metadataChanges.join(', ') }}</p> }
            @if (diff().value.definitionChanges.length > 0) {
              <ul class="definition-list">
                @for (change of diff().value.definitionChanges; track change.kind + change.id) { <li><strong>{{ change.change }}</strong> {{ change.kind }} / {{ change.id }}</li> }
              </ul>
            }
          }
        }
        @switch (importAttempt().kind) {
          @case ("loading") { <p class="state">Importing and persisting through Rust authority.</p> }
          @case ("error") { <p class="state" role="alert">{{ importAttempt().error.message }}</p> }
          @case ("data") {
            <div class="summary"><strong>{{ importAttempt().value.statusLabel }}</strong><span>{{ importAttempt().value.packLabel }}</span></div>
            @if (importAttempt().value.errorMessage !== null) { <p class="state" role="alert">{{ importAttempt().value.errorMessage }}</p> }
            <ul class="diagnostics">
              @for (diagnostic of importAttempt().value.diagnostics; track diagnostic.code + diagnostic.locationLabel) {
                <li class="diagnostic"><span [class.error]="diagnostic.severityLabel === 'Error'" [class.warning]="diagnostic.severityLabel === 'Warning'">{{ diagnostic.severityLabel }}</span><code>{{ diagnostic.code }}</code><span><span class="path">{{ diagnostic.locationLabel }}</span><br />{{ diagnostic.message }}</span></li>
              }
            </ul>
          }
        }
      </section>

      @switch (workspace().kind) {
        @case ("loading") { <p class="state">Loading durable content repository.</p> }
        @case ("error") { <p class="state" role="alert">{{ workspace().error.message }}</p> }
        @case ("idle") { <p class="state">Durable content repository has not been loaded.</p> }
        @case ("data") {
          <div class="workspace">
            <div class="pack-list" aria-label="Stored authored packs">
              @if (workspace().value.packs.length === 0) { <p class="state">No authored packs stored.</p> }
              @for (pack of workspace().value.packs; track pack.fingerprintLabel) {
                <button class="pack-button" type="button" [attr.aria-pressed]="selectedReference()?.fingerprint.value === pack.reference.fingerprint.value" (click)="selectPack(pack.reference)">
                  <strong>{{ pack.title }}</strong><span>{{ pack.identityLabel }}</span><span class="meta" [class.active]="pack.active">{{ pack.statusLabel }} · {{ pack.rulesetLabel }}</span>
                </button>
              }
            </div>
            <section class="detail" aria-label="Selected authored pack review">
              @switch (review().kind) {
                @case ("loading") { <p class="state">Loading authored payload and canonical receipt.</p> }
                @case ("error") { <p class="state" role="alert">{{ review().error.message }}</p> }
                @case ("idle") { <p class="state">Select a stored pack to review it.</p> }
                @case ("data") {
                  <div class="summary"><h3>{{ review().value.pack.title }}</h3><strong [class.active]="review().value.pack.active">{{ review().value.pack.statusLabel }}</strong></div>
                  <p>{{ review().value.pack.summary }}</p>
                  <p class="meta">{{ review().value.pack.provenanceLabel }} · {{ review().value.pack.rulesetLabel }}</p>
                  <p class="fingerprint">{{ review().value.pack.fingerprintLabel }}</p>
                  <div class="actions">
                    <button type="button" [disabled]="review().value.pack.active" (click)="activate()">Activate exact set</button>
                    <button type="button" [disabled]="!review().value.pack.active" (click)="deactivate()">Deactivate</button>
                    <button type="button" [disabled]="review().value.pack.active" (click)="deletePack()">Delete inactive pack</button>
                  </div>
                  <h4>Dependencies</h4>
                  @if (review().value.pack.dependencies.length === 0) { <p class="state">No dependencies.</p> }
                  <ul class="dependency-list">@for (dependency of review().value.pack.dependencies; track dependency.fingerprintLabel) { <li>{{ dependency.identityLabel }}<br /><span class="fingerprint">{{ dependency.fingerprintLabel }}</span></li> }</ul>
                  <h4>Canonical Definitions</h4>
                  <ul class="definition-list">@for (definition of review().value.pack.definitions; track definition.kind + definition.id) { <li><strong>{{ definition.kind }}</strong><br />{{ definition.id }}</li> }</ul>
                  <details><summary>Authored payload</summary><pre>{{ review().value.authoredPayload }}</pre></details>
                }
              }
            </section>
          </div>
          <section class="audit" aria-label="Content lifecycle audit">
            <h3>Lifecycle Audit</h3>
            @if (workspace().value.audit.length === 0) { <p class="state">No authored content lifecycle events recorded.</p> }
            <ul class="audit-list">@for (entry of workspace().value.audit; track entry.sequenceLabel) { <li><strong>{{ entry.sequenceLabel }} · {{ entry.operationLabel }}</strong><br /><span class="fingerprint">{{ entry.packLabel }}</span><br />{{ entry.detail }}</li> }</ul>
          </section>
        }
      }
    </section>
  `,
})
export class ContentPacksDialogContentComponent implements OnInit {
  private readonly store = inject(ContentWorkbenchStore);
  private readonly fileInput = inject(TEXT_FILE_INPUT);
  protected readonly workspace = computed(() => this.store.workspace());
  protected readonly importAttempt = computed(() => this.store.importAttempt());
  protected readonly review = computed(() => this.store.review());
  protected readonly diff = computed(() => this.store.diff());
  protected readonly stagedPayload = computed(() => this.store.stagedPayload());
  protected readonly selectedReference = computed(() => this.store.selectedReference());

  ngOnInit(): void { void this.store.loadWorkspace(); }
  protected reload(): void { void this.store.loadWorkspace(); }
  protected importPack(replace: boolean): void { void this.store.importStaged(replace); }
  protected comparePack(): void { void this.store.compareStaged(); }
  protected selectPack(reference: RulebenchContentPackReferenceDto): void { void this.store.selectPack(reference); }
  protected activate(): void { void this.store.activateSelected(); }
  protected deactivate(): void { void this.store.deactivateSelected(); }
  protected deletePack(): void { void this.store.deleteSelected(); }
  protected async stageFile(event: Event): Promise<void> {
    const input = event.target;
    if (!(input instanceof HTMLInputElement)) return;
    const file = input.files?.item(0);
    if (file === null || file === undefined) return;
    const loaded = await this.fileInput.readText(file);
    this.store.stagePayload(loaded.text);
    input.value = "";
  }
}
