import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
} from "@angular/core";
import { LiveCombatStore } from "@asha-rulebench/store";

@Component({
  selector: "arb-capability-manifest-dialog-content",
  changeDetection: ChangeDetectionStrategy.OnPush,
  styles: [
    `
      :host,
      .capability-manifest {
        display: grid;
        gap: 1rem;
        min-width: 0;
      }

      h2,
      p {
        margin: 0;
      }

      dl {
        display: grid;
        grid-template-columns: minmax(7rem, auto) minmax(0, 1fr);
        margin: 0;
      }

      dt,
      dd {
        border-bottom: 1px solid var(--arb-border);
        margin: 0;
        padding: 0.35rem 0.5rem;
      }

      dt {
        color: var(--arb-muted);
      }

      dd,
      p {
        overflow-wrap: anywhere;
      }

      .capability-table {
        border-collapse: collapse;
        font-size: 0.82rem;
        min-width: 52rem;
        width: 100%;
      }

      .capability-table-scroll {
        max-width: 100%;
        min-width: 0;
        overflow-x: auto;
        width: 100%;
      }

      th,
      td {
        border: 1px solid var(--arb-border);
        padding: 0.35rem 0.45rem;
        text-align: left;
        vertical-align: top;
      }

      th {
        color: var(--arb-muted);
      }

      .capability-id {
        font-family: monospace;
      }
    `,
  ],
  template: `
    <section
      class="capability-manifest"
      aria-label="Executable capability evidence"
    >
      @switch (capabilities().kind) {
        @case ("data") {
          <p>
            This is evidence projected from Rust registries and the current host
            composition. It does not bypass runtime validation.
          </p>
          <dl>
            <dt>Manifest</dt>
            <dd>
              {{ capabilities().value.manifestId }} v{{
                capabilities().value.manifestVersion
              }}
            </dd>
            <dt>Host</dt>
            <dd>{{ capabilities().value.hostLabel }}</dd>
            <dt>Protocol</dt>
            <dd>{{ capabilities().value.protocolLabel }}</dd>
            <dt>Operation vocabulary</dt>
            <dd>
              pipeline {{ capabilities().value.operationVocabularyVersion }} ·
              effects {{ capabilities().value.effectVocabularyVersion }}
            </dd>
            <dt>Recovery</dt>
            <dd>{{ capabilities().value.recoveryLabel }}</dd>
            <dt>Authority viewer</dt>
            <dd>{{ capabilities().value.authorityViewerLabel }}</dd>
            <dt>ASHA revision</dt>
            <dd class="capability-id">
              {{ capabilities().value.governedAshaRevision }}
            </dd>
            <dt>Registry</dt>
            <dd>
              {{ capabilities().value.providers.length }} providers ·
              {{ capabilities().value.rulesetLabels.length }} rulesets ·
              {{ capabilities().value.packageLabels.length }} packages ·
              {{ capabilities().value.scenarioCount }} scenarios
            </dd>
            <dt>Providers</dt>
            <dd>
              @for (
                provider of capabilities().value.providers;
                track provider.providerLabel
              ) {
                <div>
                  <span class="capability-id">{{ provider.providerLabel }}</span>
                  → {{ provider.rulesetLabel }} ·
                  {{ provider.compatibilityLabel }} ·
                  {{ provider.capabilityCount }} capabilities
                </div>
              }
            </dd>
          </dl>
          <div
            class="capability-table-scroll"
            role="region"
            tabindex="0"
            aria-label="Scrollable capability matrix"
          >
            <table class="capability-table">
              <caption>
                Executable support matrix
              </caption>
              <thead>
                <tr>
                  <th scope="col">Capability</th>
                  <th scope="col">Kind</th>
                  <th scope="col">Version</th>
                  <th scope="col">Current support</th>
                  <th scope="col">Evidence owner</th>
                </tr>
              </thead>
              <tbody>
                @for (
                  capability of capabilities().value.capabilities;
                  track capability.id
                ) {
                  <tr>
                    <td class="capability-id">{{ capability.id }}</td>
                    <td>{{ capability.kindLabel }}</td>
                    <td>{{ capability.version }}</td>
                    <td>{{ capability.support.supportLabel }}</td>
                    <td>{{ capability.evidence.join(", ") }}</td>
                  </tr>
                }
              </tbody>
            </table>
          </div>
        }
        @case ("loading") {
          <p role="status">Loading Rust capability evidence</p>
        }
        @case ("error") {
          <p role="alert">{{ capabilities().error.message }}</p>
          <button type="button" (click)="refresh()">Retry</button>
        }
        @case ("idle") {
          <p>No capability evidence loaded.</p>
          <button type="button" (click)="refresh()">Load capabilities</button>
        }
      }
    </section>
  `,
})
export class CapabilityManifestDialogContentComponent {
  private readonly store = inject(LiveCombatStore);
  protected readonly capabilities = computed(() => this.store.capabilities());

  protected refresh(): void {
    void this.store.loadCapabilities();
  }
}
