import { Component, computed, effect, inject, input, signal } from '@angular/core';

import { FormatAvailablePercentPipe } from '../../directives/format.available.percent.pipe';
import { FormatSizePipe } from '../../directives/format.size.pipe';
import { AsyncLoader } from '../../models/resource.loader';
import { StorageDiskInfo, StorageLocation } from '../../models/storage.model';
import { DialogService } from '../../services/dialog.service';
import { StorageService } from '../../services/storage.service';
import { ConfirmDialogComponent } from '../shared/confirm-dialog/confirm.dialog.component';
import { StorageEditorComponent } from './storage.editor.component';

@Component({
    selector: 'mtx-storage-manage',
    imports: [StorageEditorComponent, FormatSizePipe, FormatAvailablePercentPipe],
    templateUrl: './storage.manage.component.html',
})
export class StorageManageComponent {
    readonly showHeading = input(true);

    private readonly storageService = inject(StorageService);
    private readonly dialogService = inject(DialogService);

    readonly disksLoader = new AsyncLoader<StorageDiskInfo[]>([]);
    readonly storageLoader = new AsyncLoader<StorageLocation[]>([]);

    readonly disks = computed(() => this.disksLoader.value() ?? []);
    readonly storages = computed(() => this.storageLoader.value() ?? []);
    readonly isLoading = computed(() => this.disksLoader.loading() || this.storageLoader.loading());

    readonly actionError = signal<string | null>(null);
    readonly error = computed(() =>
        this.actionError() ?? this.disksLoader.error() ?? this.storageLoader.error() ?? null,
    );

    readonly editingStorage = signal<StorageLocation | null>(null);

    private readonly loadEffect = effect(() => {
        this.disksLoader.load(
            () => this.storageService.getDisks(),
            (disks) => { },
            'Failed to load disks.',
        );
        this.storageLoader.load(
            () => this.storageService.getStorages(),
            (storages) => { },
            'Failed to load storage locations.',
        );
    });

    createStorage(): void {
        const storage: StorageLocation = {
            id: '',
            label: '',
            path: 'Nimble',
            isDefault: false,
            createdAt: new Date().toISOString(),
            categoryTemplate: 'date',
        }
        this.actionError.set(null);
        this.editingStorage.set(storage);
    }

    onEditorCancelled(): void {
        this.editingStorage.set(null);
    }

    onLocationCreated(location: StorageLocation): void {
        this.actionError.set(null);
        const current = this.storages();
        this.storageLoader.set([...current, location]);
        if (location.isDefault) {
            this.storageLoader.set(
                this.storages().map((entry) =>
                    entry.id === location.id ? entry : { ...entry, isDefault: false },
                ),
            );
        }
    }

    onLocationsUpdated(locations: StorageLocation[]): void {
        this.actionError.set(null);
        this.storageLoader.set(locations);
    }

    onEditorError(message: string): void {
        this.actionError.set(message);
    }

    startEdit(location: StorageLocation): void {
        this.actionError.set(null);
        this.editingStorage.set(location);
    }

    deleteLocation(location: StorageLocation): void {
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
            this.storageService
                .deleteStorage(location.id)
                .subscribe({
                    next: (locations) => this.storageLoader.set(locations),
                    error: (err) => {
                        this.actionError.set(err.error?.message || 'Failed to remove storage.');
                    },
                });
        });
    }

    setDefault(location: StorageLocation): void {
        this.storageService
            .setDefault(location.id)
            .subscribe({
                next: (locations) => this.storageLoader.set(locations),
                error: (err) => {
                    this.actionError.set(err.error?.message || 'Failed to update default storage.');
                },
            });
    }

    refreshLocation(location: StorageLocation): void {
        this.storageService
            .refreshLocation(location.id)
            .subscribe({
                next: (locations) => { },
                error: (err) => {
                    this.actionError.set(err.error?.message || 'Failed to refresh storage location.');
                },
            });
    }
}