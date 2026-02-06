import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable, map } from 'rxjs';

import { DashboardSetting } from '../models/dashboard.settings.model';
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

    updateSetting(key: string, value: unknown): Observable<DashboardSetting> {
        return this.http.put<DashboardSetting>(`${this.apiBase}/dashboard/settings/${key}`, { value });
    }

    uploadLogo(dataUrl: string): Observable<DashboardSetting> {
        return this.http.post<DashboardSetting>(`${this.apiBase}/dashboard/settings/site.logo/upload`, {
            dataUrl,
        });
    }

    getLogoUrl(): Observable<string | null> {
        return this.getSettingByName('site.logo').pipe(
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
