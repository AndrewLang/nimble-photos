import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';

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
}

