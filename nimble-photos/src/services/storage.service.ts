import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';

import { API_BASE_URL } from './api.config';
import { CreateStorageLocationRequest, StorageDiskInfo, StorageLocation } from '../models/storage.model';

@Injectable({
    providedIn: 'root',
})
export class StorageService {
    private readonly apiBase = API_BASE_URL;

    constructor(private readonly http: HttpClient) {}

    getDisks(): Observable<StorageDiskInfo[]> {
        return this.http.get<StorageDiskInfo[]>(`${this.apiBase}/storage/disks`);
    }

    getLocations(): Observable<StorageLocation[]> {
        return this.http.get<StorageLocation[]>(`${this.apiBase}/storage/locations`);
    }

    createLocation(request: CreateStorageLocationRequest): Observable<StorageLocation> {
        return this.http.post<StorageLocation>(`${this.apiBase}/storage/locations`, request);
    }

    setDefault(id: string): Observable<StorageLocation[]> {
        return this.http.put<StorageLocation[]>(`${this.apiBase}/storage/locations/${id}/default`, {});
    }
}
