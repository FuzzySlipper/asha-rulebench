import { Component, computed, inject } from '@angular/core';
import type { OnInit } from '@angular/core';
import type { Routes } from '@angular/router';
import { SessionStore } from '@template/store';

@Component({
  selector: 'tpl-shell-home',
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
        border-left: 4px solid var(--tpl-accent);
        padding-left: 12px;
      }
    `,
  ],
  template: `
    <main>
      <h1>UI Pattern Bootstrap</h1>
      <p>Layer skeleton online</p>
      <section class="status" aria-label="Session status">
        @switch (status().kind) {
          @case ('idle') { <p>Session idle</p> }
          @case ('loading') { <p>Loading session</p> }
          @case ('data') { <p>{{ status().value.label }}</p> }
          @case ('error') { <p>{{ status().error.message }}</p> }
        }
      </section>
    </main>
  `,
})
export class ShellHomeComponent implements OnInit {
  private readonly sessionStore = inject(SessionStore);
  protected readonly status = computed(() => this.sessionStore.status());

  ngOnInit(): void {
    void this.sessionStore.load();
  }
}

export const shellRoutes: Routes = [{ path: '', component: ShellHomeComponent }];
