import { ApplicationConfig, provideAppInitializer, inject } from '@angular/core';
import { provideRouter, withComponentInputBinding, withInMemoryScrolling } from '@angular/router';
import { provideHttpClient, withInterceptors, withFetch } from '@angular/common/http';
import { routes } from './app.routes';
import { credentialsInterceptor } from './core/interceptors/credentials.interceptor';
import { Auth } from './core/services/auth';
import { baseUrlInterceptor } from './core/interceptors/base-url.interceptor';

// WebMCP tools are registered where the state/behavior they expose lives, not here:
// - App-wide tools (cart, current user) are declared via `declareWebMcpTool` in the
//   constructors of `CartStore` and `Auth`, so they're always available.
// - Page-scoped tools (browsing, checkout, orders) are registered via `provideWebMcpTools`
//   in the relevant route's `providers` array, so they inherit that route's guards and are
//   only available while the user is actually on that page. See each feature's
//   `*.webmcp-tools.ts` file.
export const appConfig: ApplicationConfig = {
  providers: [
    provideRouter(
      routes,
      withComponentInputBinding(),
      withInMemoryScrolling({
        scrollPositionRestoration: 'enabled',
        anchorScrolling: 'enabled',
      }),
    ),
    provideHttpClient(withInterceptors([baseUrlInterceptor, credentialsInterceptor]), withFetch()),
    provideAppInitializer(() => inject(Auth).init()),
  ],
};
