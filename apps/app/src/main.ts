import { bootstrapApplication } from '@angular/platform-browser';
import { provideRouter } from '@angular/router';
import { shellRoutes } from '@asha-rulebench/shell';
import { provideRulebenchStoreKernel } from '@asha-rulebench/store';
import { AppComponent } from './app.component';

bootstrapApplication(AppComponent, {
  providers: [provideRouter(shellRoutes), provideRulebenchStoreKernel()],
}).catch((error: unknown) => {
  console.error(error);
});
