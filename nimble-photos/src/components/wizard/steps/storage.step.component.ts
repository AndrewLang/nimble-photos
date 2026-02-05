import { Component, computed, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { finalize } from 'rxjs';

import { StorageService } from '../../../services/storage.service';
import { StorageDiskInfo, StorageLocation } from '../../../models/storage.model';

@Component({
    selector: 'mtx-storage-step',
    imports: [ReactiveFormsModule],
    templateUrl: './storage.step.component.html',
})
export class StorageStepComponent {
    private readonly fb = inject(FormBuilder);
    private readonly storageService = inject(StorageService);

    readonly disks = signal<StorageDiskInfo[]>([]);
    readonly locations = signal<StorageLocation[]>([]);
    readonly loading = signal(true);
    readonly saving = signal(false);
    readonly error = signal<string | null>(null);
    readonly showCreateForm = signal(false);
    readonly diskMenuOpen = signal(false);
    readonly selectedDiskLabel = signal('');

    readonly storageForm = this.fb.nonNullable.group({
        label: ['', [Validators.required, Validators.minLength(2)]],
        diskMount: ['', [Validators.required]],
        folderName: ['Nimble Photos', [Validators.required, Validators.minLength(2)]],
    });

    readonly selectedDisk = computed(() => {
        const mount = this.storageForm.get('diskMount')?.value ?? '';
        if (!mount) {
            return null;
        }
        return this.disks().find(disk => disk.mountPoint === mount) || null;
    });

    constructor() {
        this.loadData();
    }

    loadData(): void {
        this.loading.set(true);
        this.error.set(null);

        this.storageService.getDisks().subscribe({
            next: (disks) => {
                this.disks.set(disks);
                if (disks.length === 1) {
                    this.storageForm.get('diskMount')?.setValue(disks[0].mountPoint);
                    this.selectedDiskLabel.set(`${disks[0].name} (${disks[0].mountPoint})`);
                }
            },
        });

        this.storageService
            .getLocations()
            .pipe(finalize(() => this.loading.set(false)))
            .subscribe({
                next: (locations) => this.locations.set(locations),
                error: (err) => {
                    this.error.set(err.error?.message || 'Failed to load storage locations.');
                },
            });
    }

    openCreate(): void {
        this.showCreateForm.set(true);
    }

    cancelCreate(): void {
        this.showCreateForm.set(false);
        this.storageForm.reset({ label: '', diskMount: this.disks()[0]?.mountPoint ?? '', folderName: 'Nimble Photos' });
    }

    createLocation(): void {
        if (this.storageForm.invalid || this.saving()) {
            this.storageForm.markAllAsTouched();
            return;
        }

        this.saving.set(true);
        this.error.set(null);

        const { label, diskMount, folderName } = this.storageForm.getRawValue();
        const path = this.buildPath(diskMount, folderName);

        this.storageService
            .createLocation({ label: label.trim(), path })
            .pipe(finalize(() => this.saving.set(false)))
            .subscribe({
                next: (location) => {
                    this.locations.update((current) => [...current, location]);
                    if (location.isDefault) {
                        this.locations.update((current) =>
                            current.map((entry) =>
                                entry.id === location.id
                                    ? entry
                                    : { ...entry, isDefault: false },
                            ),
                        );
                    }
                    this.cancelCreate();
                },
                error: (err) => {
                    this.error.set(err.error?.message || 'Failed to add storage location.');
                },
            });
    }

    setDefault(location: StorageLocation): void {
        if (this.saving()) {
            return;
        }
        this.saving.set(true);
        this.storageService
            .setDefault(location.id)
            .pipe(finalize(() => this.saving.set(false)))
            .subscribe({
                next: (locations) => this.locations.set(locations),
                error: (err) => {
                    this.error.set(err.error?.message || 'Failed to update default storage.');
                },
            });
    }

    selectDisk(disk: StorageDiskInfo): void {
        this.storageForm.get('diskMount')?.setValue(disk.mountPoint);
        this.selectedDiskLabel.set(`${disk.name} (${disk.mountPoint})`);
        this.diskMenuOpen.set(false);
    }

    toggleDiskMenu(): void {
        this.diskMenuOpen.set(!this.diskMenuOpen());
    }

    closeDiskMenu(): void {
        this.diskMenuOpen.set(false);
    }

    formatBytes(bytes: number): string {
        if (!Number.isFinite(bytes) || bytes <= 0) {
            return '0 B';
        }
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
        const value = bytes / Math.pow(1024, index);
        return `${value.toFixed(value >= 10 || index === 0 ? 0 : 1)} ${units[index]}`;
    }

    private buildPath(mountPoint: string, folderName: string): string {
        const trimmedFolder = folderName.trim().replace(/^[/\\]+/, '');
        if (!trimmedFolder) {
            return mountPoint;
        }
        const separator = mountPoint.endsWith('\\') || mountPoint.endsWith('/') ? '' : '\\';
        return `${mountPoint}${separator}${trimmedFolder}`;
    }
}
