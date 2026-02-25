import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable, catchError, first, map, of } from 'rxjs';

import { DashboardSetting } from '../models/dashboard.settings.model';
import { SettingNames } from '../models/setting.names';
import { API_BASE_URL } from './api.config';

@Injectable({
    providedIn: 'root',
})
export class SettingsService {
    private readonly apiBase = API_BASE_URL;

    constructor(private readonly http: HttpClient) { }

    getSettings(): Observable<DashboardSetting[]> {
        return this.http.get<DashboardSetting[]>(`${this.apiBase}/dashboard/settings`);
    }

    getSettingByName(key: string): Observable<DashboardSetting> {
        return this.http.get<DashboardSetting>(`${this.apiBase}/dashboard/settings/${key}`);
    }

    getSetting<T>(
        name: string,
        apply: (value: T) => void
    ): void {
        this.getSettingByName(name)
            .pipe(
                first(),
                catchError(() => of(null))
            )
            .subscribe(setting => {
                if (setting?.value != null) {
                    apply(setting.value as T);
                }
            });
    }

    updateSetting(key: string, value: unknown): Observable<DashboardSetting> {
        return this.http.put<DashboardSetting>(`${this.apiBase}/dashboard/settings/${key}`, { value });
    }

    uploadLogo(dataUrl: string): Observable<DashboardSetting> {
        return this.http.post<DashboardSetting>(`${this.apiBase}/dashboard/settings/logo/upload`, {
            dataUrl,
        });
    }

    getLogoUrl(): Observable<string | null> {
        return this.getSettingByName(SettingNames.SiteLogo).pipe(
            map((setting) => {
                const raw = typeof setting?.value === 'string' ? setting.value : '';
                const value = this.buildLogoUrl(raw);
                return typeof value === 'string' && value.trim().length ? value : null;
            }),
        );
    }

    buildLogoUrl(path: string): string {
        if (typeof path !== 'string') {
            return '';
        }
        const trimmed = path.trim();
        if (!trimmed.length) {
            return '';
        }
        if (trimmed.startsWith('http://') || trimmed.startsWith('https://') || trimmed.startsWith('data:')) {
            return trimmed;
        }
        if (trimmed.startsWith(this.apiBase)) {
            return trimmed;
        }
        return `${this.apiBase}${trimmed}`;
    }
}
