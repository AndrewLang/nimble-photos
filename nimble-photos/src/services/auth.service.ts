import { HttpClient } from '@angular/common/http';
import { Injectable, signal, computed } from '@angular/core';
import { Router } from '@angular/router';
import { Observable, tap } from 'rxjs';
import { LoginRequest, LoginResponse, RegisterRequest } from '../models/auth.model';
import { User } from '../models/user.model';

@Injectable({
    providedIn: 'root',
})
export class AuthService {
    private readonly apiBase = 'http://localhost:8080/api';
    private readonly TOKEN_KEY = 'mtx_access_token';
    private readonly REFRESH_TOKEN_KEY = 'mtx_refresh_token';
    private readonly USER_KEY = 'mtx_user';

    readonly currentUser = signal<User | null>(this.getStoredUser());
    readonly isAuthenticated = computed(() => !!this.currentUser());

    constructor(
        private readonly http: HttpClient,
        private readonly router: Router
    ) { }

    login(request: LoginRequest): Observable<LoginResponse> {
        return this.http.post<LoginResponse>(`${this.apiBase}/auth/login`, request).pipe(
            tap((response) => {
                this.setTokens(response.accessToken, response.refreshToken);
                // In a real app, you might want to fetch user profile after login
                // For now, we'll simulate setting a user if the backend doesn't return it in login
                const dummyUser: User = {
                    id: '1',
                    email: request.email,
                    displayName: request.email.split('@')[0],
                    createdAt: new Date().toISOString(),
                    emailVerified: true
                };
                this.setUser(dummyUser);
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
