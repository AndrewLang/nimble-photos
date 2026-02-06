import { HttpErrorResponse, HttpInterceptorFn } from '@angular/common/http';
import { inject } from '@angular/core';
import { catchError, switchMap, throwError } from 'rxjs';
import { AuthService } from '../services/auth.service';

export const authInterceptor: HttpInterceptorFn = (req, next) => {
    const authService = inject(AuthService);
    const token = authService.getAccessToken();
    const shouldAttachToken = !isAuthEndpoint(req.url);

    const requestWithAuth = shouldAttachToken && token
        ? req.clone({
            headers: req.headers.set('Authorization', `Bearer ${token}`)
        })
        : req;

    return next(requestWithAuth).pipe(
        catchError((error: unknown) => {
            if (!(error instanceof HttpErrorResponse) || error.status !== 401) {
                return throwError(() => error);
            }

            if (isAuthEndpoint(req.url) || req.headers.has('x-auth-retry')) {
                return throwError(() => error);
            }

            if (!authService.getRefreshToken()) {
                authService.handleAuthFailure();
                return throwError(() => error);
            }

            return authService.refreshAccessToken().pipe(
                switchMap((newToken) => {
                    const retried = req.clone({
                        headers: req.headers
                            .set('Authorization', `Bearer ${newToken}`)
                            .set('x-auth-retry', '1')
                    });
                    return next(retried);
                }),
                catchError((refreshError: unknown) => throwError(() => refreshError))
            );
        })
    );
};

function isAuthEndpoint(url: string): boolean {
    return url.includes('/auth/login')
        || url.includes('/auth/register')
        || url.includes('/auth/refresh')
        || url.includes('/auth/logout');
}
