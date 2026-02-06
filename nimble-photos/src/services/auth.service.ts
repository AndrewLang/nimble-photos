import { HttpClient } from '@angular/common/http';
import { computed, inject, Injectable, signal } from '@angular/core';
import { Router } from '@angular/router';
import { jwtDecode } from 'jwt-decode';
import { catchError, finalize, map, Observable, shareReplay, switchMap, tap, throwError } from 'rxjs';
import { LoginRequest, LoginResponse, RegisterRequest, RegistrationStatus } from '../models/auth.model';
import { JwtClaims } from '../models/jwt-claims.model';
import { User } from '../models/user.model';
import { API_BASE_URL } from './api.config';

@Injectable({
    providedIn: 'root',
})
export class AuthService {
    private readonly http = inject(HttpClient);
    private readonly router = inject(Router);

    private readonly apiBase = API_BASE_URL;
    private readonly TOKEN_KEY = 'mtx_access_token';
    private readonly REFRESH_TOKEN_KEY = 'mtx_refresh_token';
    private readonly USER_KEY = 'mtx_user';
    private refreshInFlightRequest: Observable<string> | null = null;

    readonly currentUser = signal<User | null>(this.getStoredUser());
    readonly isAuthenticated = computed(() => !!this.currentUser());
    readonly isAdmin = computed(() => {
        const user = this.currentUser();
        return user?.roles?.includes('admin') ?? false;
    });

    constructor() { }

    login(request: LoginRequest): Observable<User> {
        return this.http.post<LoginResponse>(`${this.apiBase}/auth/login`, request).pipe(
            tap((response) => {
                this.setTokens(response.accessToken, response.refreshToken);
            }),
            switchMap(() => this.getProfile()),
            tap((user) => {
                this.setUser(user);
            })
        );
    }

    getProfile(): Observable<User> {
        return this.http.get<any>(`${this.apiBase}/auth/me`).pipe(
            map(profile => {
                const token = this.getAccessToken();
                let roles: string[] = [];

                if (token) {
                    try {
                        const decoded = jwtDecode<JwtClaims>(token);
                        if (decoded.roles) {
                            roles = decoded.roles;
                        }
                    } catch (e) {
                        // console.error('Failed to decode token', e);
                    }
                }

                const user: User = {
                    id: profile.id,
                    email: profile.email,
                    displayName: profile.displayName,
                    createdAt: new Date().toISOString(),
                    emailVerified: true,
                    roles: roles
                };
                return user;
            })
        );
    }

    register(request: RegisterRequest): Observable<any> {
        return this.http.post(`${this.apiBase}/auth/register`, request);
    }

    getRegistrationStatus(): Observable<RegistrationStatus> {
        return this.http.get<RegistrationStatus>(`${this.apiBase}/auth/registration-status`);
    }

    logout(): void {
        const refreshToken = this.getRefreshToken();
        if (refreshToken) {
            this.http.post(`${this.apiBase}/auth/logout`, { refreshToken }).subscribe({
                next: () => this.clearLocalSession(),
                error: () => this.clearLocalSession()
            });
        } else {
            this.clearLocalSession();
        }
    }

    refreshAccessToken(): Observable<string> {
        const refreshToken = this.getRefreshToken();
        if (!refreshToken) {
            return throwError(() => new Error('Refresh token not found.'));
        }

        if (!this.refreshInFlightRequest) {
            this.refreshInFlightRequest = this.http
                .post<LoginResponse>(`${this.apiBase}/auth/refresh`, { refreshToken })
                .pipe(
                    tap((response) => {
                        this.setTokens(response.accessToken, response.refreshToken);
                    }),
                    map(response => response.accessToken),
                    catchError(err => {
                        this.clearLocalSession();
                        return throwError(() => err);
                    }),
                    finalize(() => {
                        this.refreshInFlightRequest = null;
                    }),
                    shareReplay(1)
                );
        }

        return this.refreshInFlightRequest;
    }

    handleAuthFailure(): void {
        this.clearLocalSession();
    }

    private clearLocalSession(): void {
        localStorage.removeItem(this.TOKEN_KEY);
        localStorage.removeItem(this.REFRESH_TOKEN_KEY);
        localStorage.removeItem(this.USER_KEY);

        this.currentUser.set(null);
        this.router.navigate(['/login']);
    }

    private setTokens(accessToken: string, refreshToken: string): void {
        localStorage.setItem(this.TOKEN_KEY, accessToken);
        localStorage.setItem(this.REFRESH_TOKEN_KEY, refreshToken);
    }

    private setUser(user: User): void {
        localStorage.setItem(this.USER_KEY, JSON.stringify(user));
        this.currentUser.set(user);
    }

    private getStoredUser(): User | null {
        const userJson = localStorage.getItem(this.USER_KEY);
        if (userJson) {
            try {
                return JSON.parse(userJson);
            } catch {
                return null;
            }
        }
        return null;
    }

    getAccessToken(): string | null {
        return localStorage.getItem(this.TOKEN_KEY);
    }

    getRefreshToken(): string | null {
        return localStorage.getItem(this.REFRESH_TOKEN_KEY);
    }
}
