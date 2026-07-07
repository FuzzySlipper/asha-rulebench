import { bootstrapApplication } from '@angular/platform-browser';
import { provideRouter } from '@angular/router';
import { shellRoutes } from '@template/shell';
import { provideTemplateStoreKernel } from '@template/store';
import { AppComponent } from './app.component';

bootstrapApplication(AppComponent, {
  providers: [provideRouter(shellRoutes), provideTemplateStoreKernel()],
}).catch((error: unknown) => {
  console.error(error);
});
