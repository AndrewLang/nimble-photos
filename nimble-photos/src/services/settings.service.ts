import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';

import { DashboardSetting } from '../models/dashboard-settings.model';

@Injectable({
    providedIn: 'root',
})
export class SettingsService {
    private readonly apiBase = 'http://localhost:8080/api';

    constructor(private readonly http: HttpClient) { }

    getSettings(): Observable<DashboardSetting[]> {
        return this.http.get<DashboardSetting[]>(`${this.apiBase}/dashboard/settings`);
    }

    updateSetting(key: string, value: unknown): Observable<DashboardSetting> {
        return this.http.put<DashboardSetting>(`${this.apiBase}/dashboard/settings/${key}`, { value });
    }
}
