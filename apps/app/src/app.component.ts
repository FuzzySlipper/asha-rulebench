import { Component } from '@angular/core';
import { RouterOutlet } from '@angular/router';

@Component({
  imports: [RouterOutlet],
  selector: 'tpl-root',
  styles: [
    `
      :host {
        display: block;
        min-height: 100vh;
      }
    `,
  ],
  template: `<router-outlet />`,
})
export class AppComponent {}
