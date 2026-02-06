import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';

import { AdminDashboardUser, UpdateUserRolesRequest } from '../models/security-admin.model';
import { API_BASE_URL } from './api.config';

@Injectable({
    providedIn: 'root',
})
export class SecurityAdminService {
    private readonly apiBase = API_BASE_URL;

    constructor(private readonly http: HttpClient) { }

    getUsers(): Observable<AdminDashboardUser[]> {
        return this.http.get<AdminDashboardUser[]>(`${this.apiBase}/admin/users`);
    }

    updateUserRoles(userId: string, payload: UpdateUserRolesRequest): Observable<AdminDashboardUser> {
        return this.http.put<AdminDashboardUser>(`${this.apiBase}/admin/users/${userId}/roles`, payload);
    }
}
