import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';

import { DashboardClient } from '../models/client.model';
import { API_BASE_URL } from './api.config';

@Injectable({
    providedIn: 'root',
})
export class ClientAdminService {
    private readonly apiBase = API_BASE_URL;

    constructor(private readonly http: HttpClient) { }

    listClients(): Observable<DashboardClient[]> {
        return this.http.get<DashboardClient[]>(`${this.apiBase}/clients`);
    }

    approveClient(clientId: string): Observable<DashboardClient> {
        return this.http.put<DashboardClient>(`${this.apiBase}/clients/${clientId}/approve`, {});
    }

    revokeClient(clientId: string): Observable<DashboardClient> {
        return this.http.put<DashboardClient>(`${this.apiBase}/clients/${clientId}/revoke`, {});
    }
}
