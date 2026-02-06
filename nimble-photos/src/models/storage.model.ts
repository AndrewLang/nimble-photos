export interface StorageDiskInfo {
    name: string;
    mountPoint: string;
    totalBytes: number;
    availableBytes: number;
}

export interface StorageLocation {
    id: string;
    label: string;
    path: string;
    isDefault: boolean;
    createdAt: string;
    disk?: StorageDiskInfo | null;
}

export interface CreateStorageLocationRequest {
    label: string;
    path: string;
    isDefault?: boolean;
}

export interface UpdateStorageLocationRequest {
    label?: string;
    path?: string;
    isDefault?: boolean;
}
