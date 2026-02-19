import { Component, Input, computed, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { finalize } from 'rxjs';

import { Formatter } from '../../models/formatters';
import { StorageDiskInfo, StorageLocation } from '../../models/storage.model';
import { DialogService } from '../../services/dialog.service';
import { StorageService } from '../../services/storage.service';
import { DropdownComponent } from '../dropdown/dropdown.component';
import { ConfirmDialogComponent } from '../shared/confirm-dialog/confirm.dialog.component';

@Component({
    selector: 'mtx-storage-manage',
    imports: [ReactiveFormsModule, DropdownComponent],
    templateUrl: './storage.manage.component.html',
})
export class StorageManageComponent {
    @Input() showHeading = true;

    private readonly fb = inject(FormBuilder);
    private readonly storageService = inject(StorageService);
    private readonly dialogService = inject(DialogService);

    readonly disks = signal<StorageDiskInfo[]>([]);
    readonly locations = signal<StorageLocation[]>([]);
    readonly loading = signal(true);
    readonly saving = signal(false);
    readonly error = signal<string | null>(null);
    readonly showCreateForm = signal(false);
    readonly selectedDiskMount = signal('');
    readonly editingId = signal<string | null>(null);
    readonly editSelectedDiskMount = signal('');

    readonly storageForm = this.fb.nonNullable.group({
        label: ['', [Validators.required, Validators.minLength(2)]],
        diskMount: ['', [Validators.required]],
        folderName: ['Nimble Photos', [Validators.required, Validators.minLength(2)]],
        categoryTemplate: ['hash', [Validators.required]],
    });

    readonly editForm = this.fb.nonNullable.group({
        label: ['', [Validators.required, Validators.minLength(2)]],
        diskMount: ['', [Validators.required]],
        folderName: ['', [Validators.required, Validators.minLength(2)]],
        categoryTemplate: ['hash', [Validators.required]],
    });

    readonly selectedDisk = computed(() => {
        const mount = this.selectedDiskMount();
        if (!mount) {
            return null;
        }
        return this.disks().find(disk => disk.mountPoint === mount) || null;
    });

    readonly selectedDiskLabel = computed(() => {
        const disk = this.selectedDisk();
        return disk ? `${disk.name} (${disk.mountPoint})` : '';
    });

    readonly editingLocation = computed(() => {
        const id = this.editingId();
        if (!id) {
            return null;
        }
        return this.locations().find(location => location.id === id) || null;
    });

    readonly editSelectedDisk = computed(() => {
        const mount = this.editSelectedDiskMount();
        if (!mount) {
            return null;
        }
        return this.disks().find(disk => disk.mountPoint === mount) || null;
    });

    readonly editSelectedDiskLabel = computed(() => {
        const disk = this.editSelectedDisk();
        return disk ? `${disk.name} (${disk.mountPoint})` : '';
    });
    readonly formatBytes = Formatter.formatBytes;
    readonly formatAvailablePercent = Formatter.formatAvailablePercent;
    readonly categoryTemplates = [
        { value: 'hash', label: 'Hash' },
        { value: 'date', label: 'Date' },
    ] as const;
    readonly diskLabel = (option: unknown): string => {
        const disk = option as StorageDiskInfo;
        return `${disk.name} (${disk.mountPoint})`;
    };
    readonly diskCapacity = (option: unknown): string => {
        const disk = option as StorageDiskInfo;
        return `${this.formatBytes(disk.availableBytes)} free / ${this.formatBytes(disk.totalBytes)} total`;
    };
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
                    this.selectedDiskMount.set(disks[0].mountPoint);
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
        this.storageForm.reset({
            label: '',
            diskMount: this.disks()[0]?.mountPoint ?? '',
            folderName: 'Nimble Photos',
            categoryTemplate: 'hash',
        });
        if (this.disks().length === 1) {
            this.selectedDiskMount.set(this.disks()[0].mountPoint);
        }
    }

    createLocation(): void {
        if (this.storageForm.invalid || this.saving()) {
            this.storageForm.markAllAsTouched();
            return;
        }

        this.saving.set(true);
        this.error.set(null);

        const { label, diskMount, folderName, categoryTemplate } = this.storageForm.getRawValue();
        const path = this.buildPath(diskMount, folderName);

        this.storageService
            .createLocation({ label: label.trim(), path, categoryTemplate })
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

    startEdit(location: StorageLocation): void {
        const diskMount = this.resolveDiskMount(location.path);
        const folderName = this.extractFolderName(location.path, diskMount);
        this.editingId.set(location.id);
        this.editForm.reset({
            label: location.label,
            diskMount,
            folderName,
            categoryTemplate: location.categoryTemplate || 'hash',
        });
        this.editSelectedDiskMount.set(diskMount);
        this.showCreateForm.set(false);
    }

    cancelEdit(): void {
        this.editingId.set(null);
    }

    saveEdit(location: StorageLocation): void {
        if (this.editForm.invalid || this.saving()) {
            this.editForm.markAllAsTouched();
            return;
        }

        this.saving.set(true);
        this.error.set(null);

        const { label, diskMount, folderName, categoryTemplate } = this.editForm.getRawValue();
        const path = this.buildPath(diskMount, folderName);

        this.storageService
            .updateLocation(location.id, {
                label: label.trim(),
                path: path.trim(),
                categoryTemplate,
            })
            .pipe(finalize(() => this.saving.set(false)))
            .subscribe({
                next: (locations) => {
                    this.locations.set(locations);
                    this.cancelEdit();
                },
                error: (err) => {
                    this.error.set(err.error?.message || 'Failed to update storage.');
                },
            });
    }

    deleteLocation(location: StorageLocation): void {
        if (this.saving()) {
            return;
        }
        const dialogRef = this.dialogService.open(ConfirmDialogComponent, {
            title: 'Remove Storage',
            data: {
                message: `Are you sure you want to remove "${location.label}"?`,
                type: 'warning',
            },
            actions: [
                { label: 'Cancel', value: false, style: 'ghost' },
                { label: 'Remove', value: true, style: 'danger' },
            ],
        });

        dialogRef.afterClosed().then((confirmed) => {
            if (!confirmed) {
                return;
            }
            this.saving.set(true);
            this.storageService
                .deleteLocation(location.id)
                .pipe(finalize(() => this.saving.set(false)))
                .subscribe({
                    next: (locations) => this.locations.set(locations),
                    error: (err) => {
                        this.error.set(err.error?.message || 'Failed to remove storage.');
                    },
                });
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

    onCreateDiskMountChange(value: unknown): void {
        this.selectedDiskMount.set(String(value ?? ''));
    }

    onEditDiskMountChange(value: unknown): void {
        this.editSelectedDiskMount.set(String(value ?? ''));
    }

    private buildPath(mountPoint: string, folderName: string): string {
        const trimmedFolder = folderName.trim().replace(/^[/\\]+/, '');
        if (!trimmedFolder) {
            return mountPoint;
        }
        const separator = mountPoint.endsWith('\\') || mountPoint.endsWith('/') ? '' : '\\';
        return `${mountPoint}${separator}${trimmedFolder}`;
    }

    private resolveDiskMount(path: string): string {
        const pathLower = path.toLowerCase();
        const match = this.disks()
            .filter(disk => pathLower.startsWith(disk.mountPoint.toLowerCase()))
            .sort((a, b) => b.mountPoint.length - a.mountPoint.length)[0];
        return match?.mountPoint ?? '';
    }

    private extractFolderName(path: string, mountPoint: string): string {
        if (!mountPoint) {
            return path.replace(/^[/\\]+/, '');
        }
        const remainder = path.slice(mountPoint.length);
        return remainder.replace(/^[/\\]+/, '');
    }
}
