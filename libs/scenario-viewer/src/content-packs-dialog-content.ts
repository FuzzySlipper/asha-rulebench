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
      button, input, textarea, .file-label {
        background: var(--arb-surface); border: 1px solid var(--arb-border); border-radius: 6px;
        color: var(--arb-text); min-height: 34px; padding: 7px 10px;
      }
      button, .file-label { cursor: pointer; }
      button[aria-pressed="true"] { border-color: var(--arb-accent); box-shadow: inset 3px 0 0 var(--arb-accent); }
      button:disabled { cursor: default; opacity: 0.55; }
      .file-label { align-items: center; display: inline-flex; }
      .file-label input { inline-size: 1px; opacity: 0; position: absolute; }
      .state, .meta, .fingerprint, .path { color: var(--arb-muted); font-size: 0.82rem; overflow-wrap: anywhere; }
      .workspace { display: grid; gap: 14px; grid-template-columns: minmax(210px, 0.34fr) minmax(0, 1fr); }
      .editor-grid { display: grid; gap: 10px; grid-template-columns: repeat(2, minmax(0, 1fr)); }
      .editor-grid label, .editor { display: grid; gap: 5px; }
      .editor { grid-column: 1 / -1; }
      textarea { box-sizing: border-box; font-family: ui-monospace, SFMono-Regular, Consolas, monospace; min-height: 320px; resize: vertical; width: 100%; }
      .pack-list, .detail, .panel, .audit { display: grid; gap: 9px; }
      .pack-list { align-content: start; }
      .pack-button { display: grid; gap: 3px; min-width: 0; overflow-wrap: anywhere; text-align: left; }
      .workspace > * { min-width: 0; }
      .detail, .panel { border-left: 3px solid var(--arb-border); padding-left: 12px; }
      .active { color: var(--arb-accent); font-weight: 700; }
      .diagnostics, .definition-list, .dependency-list, .audit-list {
        display: grid; gap: 7px; list-style: none; margin: 0; padding: 0;
      }
      .diagnostic { display: grid; gap: 4px; grid-template-columns: minmax(65px, auto) minmax(160px, 0.4fr) minmax(220px, 1fr); }
      .error { color: #9b2c2c; font-weight: 700; }
      .warning { color: #8a5a00; font-weight: 700; }
      .definition-list { grid-template-columns: repeat(auto-fit, minmax(170px, 1fr)); }
      .definition-list li, .audit-list li, .declaration { background: color-mix(in srgb, var(--arb-surface) 88%, transparent); border-radius: 5px; padding: 7px 9px; }
      .declaration { display: grid; gap: 4px; }
      pre { background: #10141c; border-radius: 6px; color: #e8eef8; margin: 0; max-height: 280px; overflow: auto; padding: 10px; white-space: pre-wrap; }
      details summary { cursor: pointer; }
      @media (max-width: 760px) {
        .dialog-content { padding: 14px 16px 24px; }
        .workspace, .diagnostic, .editor-grid { grid-template-columns: 1fr; }
        .editor { grid-column: auto; }
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

      <section class="panel" aria-label="Authored action draft editor">
        <div>
          <h3>Authoring draft</h3>
          <p class="state">Start from a Rust-owned v3 template, clone stored content, or load JSON. TypeScript does not interpret rules.</p>
        </div>
        <div class="editor-grid">
          <label>
            New pack id
            <input #draftId aria-label="New pack id" [value]="draftIdentity().id" (input)="setDraftId(draftId.value)" />
          </label>
          <label>
            New version
            <input #draftVersion aria-label="New pack version" [value]="draftIdentity().version" (input)="setDraftVersion(draftVersion.value)" />
          </label>
        </div>
        <div class="toolbar">
          <button type="button" [disabled]="draft().kind === 'loading'" (click)="startTemplate()">Start Rust template</button>
          <button type="button" [disabled]="selectedReference() === null || draft().kind === 'loading'" (click)="cloneSelected()">Clone selected pack</button>
          <label class="file-label">
            Load JSON file
            <input type="file" accept="application/json,.json" (change)="stageFile($event)" />
          </label>
        </div>
        @switch (draft().kind) {
          @case ("loading") { <p class="state" aria-busy="true">Rust is preparing the authored content draft.</p> }
          @case ("error") { <p class="state" role="alert">{{ draft().error.code }} · {{ draft().error.message }}</p> }
          @case ("data") {
            <p><strong>{{ draft().value.sourceLabel }}</strong> · {{ draft().value.identityLabel }}</p>
            <p class="state">{{ draft().value.identityExpectation }}</p>
          }
        }
        <label class="editor">
          Authored JSON draft
          <textarea
            #draftEditor
            aria-label="Authored JSON draft"
            spellcheck="false"
            [value]="draftPayload() ?? ''"
            (input)="updateDraft(draftEditor.value)"
          ></textarea>
        </label>
        @if (draftSyntax().kind === "error") {
          <p role="alert"><strong>JSON syntax error</strong> · {{ draftSyntax().message }}</p>
        } @else {
          <p class="state"><strong>JSON syntax</strong> · {{ draftSyntax().message }}</p>
        }
        <div class="toolbar">
          <button type="button" [disabled]="draftSyntax().kind !== 'valid' || validation().kind === 'loading'" (click)="validateDraft()">Validate with Rust</button>
          <button type="button" [disabled]="!canImportDraft() || importAttempt().kind === 'loading'" (click)="importPack(false)">Import validated draft</button>
          <button type="button" [disabled]="!canImportDraft() || diff().kind === 'loading'" (click)="comparePack()">Compare replacement</button>
          @if (diff().kind === "data") {
            <button type="button" [disabled]="importAttempt().kind === 'loading'" (click)="importPack(true)">Confirm replacement</button>
          }
        </div>
        <section aria-label="Rust semantic validation">
          <h4>Rust semantic validation</h4>
          @switch (validation().kind) {
            @case ("idle") { <p class="state">Not validated. JSON syntax alone does not claim semantic validity.</p> }
            @case ("loading") { <p class="state" aria-busy="true">Rust is decoding, resolving, and validating without persistence.</p> }
            @case ("error") { <p role="alert">{{ validation().error.code }} · {{ validation().error.message }}</p> }
            @case ("data") {
              <div class="summary"><strong>{{ validation().value.statusLabel }}</strong><span>{{ validation().value.packLabel }}</span></div>
              @if (validation().value.errorMessage !== null) { <p role="alert">{{ validation().value.errorMessage }}</p> }
              <ul class="diagnostics">
                @for (diagnostic of validation().value.diagnostics; track diagnostic.code + diagnostic.locationLabel) {
                  <li class="diagnostic"><span [class.error]="diagnostic.severityLabel === 'Error'" [class.warning]="diagnostic.severityLabel === 'Warning'">{{ diagnostic.severityLabel }}</span><code>{{ diagnostic.code }}</code><span><span class="path">{{ diagnostic.locationLabel }}</span><br />{{ diagnostic.message }}</span></li>
                }
              </ul>
            }
          }
        </section>
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
          @case ("loading") { <p class="state">Importing the validated draft through Rust authority.</p> }
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
                  <h4>Authored abilities</h4>
                  @if (review().value.abilities.length === 0) { <p class="state">No authored abilities.</p> }
                  @for (ability of review().value.abilities; track ability.id) {
                    <article class="declaration"><strong>{{ ability.name }} · {{ ability.id }}</strong><span class="meta">{{ ability.kind }} · {{ ability.tags.join(', ') || 'No tags' }}</span><span>{{ ability.summary }}</span></article>
                  }
                  <h4>Authored modifiers</h4>
                  @if (review().value.modifiers.length === 0) { <p class="state">No authored modifiers.</p> }
                  @for (modifier of review().value.modifiers; track modifier.id) {
                    <article class="declaration"><strong>{{ modifier.label }} · {{ modifier.id }}</strong><span>{{ modifier.summary }}</span><span class="meta">{{ modifier.tenure }} · {{ modifier.stacking }} · {{ modifier.duration }}</span><span class="meta">{{ modifier.statAdjustments.join(' · ') || 'No stat adjustments' }}</span></article>
                  }
                  <h4>Authored actions</h4>
                  @if (review().value.actions.length === 0) { <p class="state">No authored actions.</p> }
                  @for (action of review().value.actions; track action.id) {
                    <article class="declaration"><strong>{{ action.name }} · {{ action.id }}</strong><span class="meta">Ability {{ action.abilityId }}</span><span>{{ action.targeting }}</span><span>{{ action.check }}</span><span class="meta">Costs: {{ action.resourceCosts.join(' · ') || 'none' }}</span><span class="meta">Effects: {{ action.effects.join(' · ') || 'none' }}</span><span>{{ action.actionText }}</span><span>{{ action.effectText }}</span></article>
                  }
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
  protected readonly validation = computed(() => this.store.validation());
  protected readonly draft = computed(() => this.store.draft());
  protected readonly draftPayload = computed(() => this.store.draftPayload());
  protected readonly draftIdentity = computed(() => this.store.draftIdentity());
  protected readonly draftSyntax = computed(() => this.store.draftSyntax());
  protected readonly canImportDraft = computed(() => this.store.canImportDraft());
  protected readonly selectedReference = computed(() => this.store.selectedReference());

  ngOnInit(): void { void this.store.loadWorkspace(); }
  protected reload(): void { void this.store.loadWorkspace(); }
  protected importPack(replace: boolean): void { void this.store.importStaged(replace); }
  protected startTemplate(): void { void this.store.startTemplateDraft(); }
  protected cloneSelected(): void { void this.store.cloneSelectedDraft(); }
  protected validateDraft(): void { void this.store.validateDraft(); }
  protected updateDraft(payload: string): void { this.store.updateDraftPayload(payload); }
  protected setDraftId(id: string): void { this.store.setDraftIdentity(id, this.draftIdentity().version); }
  protected setDraftVersion(version: string): void { this.store.setDraftIdentity(this.draftIdentity().id, version); }
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
