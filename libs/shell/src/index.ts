import { Component, computed, inject } from '@angular/core';
import type { OnInit } from '@angular/core';
import type { Routes } from '@angular/router';
import { SessionStore } from '@asha-rulebench/store';

@Component({
  selector: 'arb-shell-home',
  standalone: true,
  styles: [
    `
      :host {
        display: grid;
        min-height: 100vh;
        place-items: center;
        padding: 24px;
      }

      main {
        display: grid;
        gap: 12px;
        max-width: 720px;
        width: min(100%, 720px);
      }

      h1,
      p {
        margin: 0;
      }

      h1 {
        font-size: clamp(2rem, 6vw, 3.75rem);
        line-height: 1;
      }

      .status {
        border-left: 4px solid var(--arb-accent);
        padding-left: 12px;
      }
    `,
  ],
  template: `
    <main>
      <h1>ASHA Rulebench</h1>
      <p>Rule workbench shell online</p>
      <section class="status" aria-label="Scenario status">
        @switch (scenario().kind) {
          @case ('idle') { <p>Scenario idle</p> }
          @case ('loading') { <p>Loading scenario</p> }
          @case ('data') { <p>{{ scenario().value.title }}</p> }
          @case ('error') { <p>{{ scenario().error.message }}</p> }
        }
      </section>
    </main>
  `,
})
export class ShellHomeComponent implements OnInit {
  private readonly sessionStore = inject(SessionStore);
  protected readonly scenario = computed(() => this.sessionStore.scenario());

  ngOnInit(): void {
    void this.sessionStore.loadScenario();
  }
}

export const shellRoutes: Routes = [{ path: '', component: ShellHomeComponent }];
