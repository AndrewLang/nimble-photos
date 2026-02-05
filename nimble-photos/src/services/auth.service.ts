import { HttpClient } from '@angular/common/http';
import { computed, inject, Injectable, signal } from '@angular/core';
import { Router } from '@angular/router';
import { jwtDecode } from 'jwt-decode';
import { map, Observable, switchMap, tap } from 'rxjs';
import { LoginRequest, LoginResponse, RegisterRequest } from '../models/auth.model';
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

    logout(): void {
        const refreshToken = localStorage.getItem(this.REFRESH_TOKEN_KEY);
        if (refreshToken) {
            this.http.post(`${this.apiBase}/auth/logout`, { refreshToken }).subscribe({
                next: () => this.clearLocalSession(),
                error: () => this.clearLocalSession()
            });
        } else {
            this.clearLocalSession();
        }
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
}
