import { ApplicationConfig, provideBrowserGlobalErrorListeners } from '@angular/core';
import { provideHttpClient } from '@angular/common/http';
import { provideRouter, withViewTransitions } from '@angular/router';

import { routes } from './app.routes';

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideHttpClient(),
    provideRouter(routes, withViewTransitions({
      onViewTransitionCreated: ({ transition, to, from }) => {
        // Skip transition if navigating to the same route config (e.g. changing params)
        const toRoute = to.firstChild;
        const fromRoute = from.firstChild;
        if (toRoute && fromRoute && toRoute.routeConfig === fromRoute.routeConfig) {
          transition.skipTransition();
        }
      }
    }))
  ]
};
