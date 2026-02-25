import { Component, computed, effect, inject, input, model, output, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { finalize } from 'rxjs';

import { FormatSizePipe } from '../../directives/format.size.pipe';
import { NamedValue } from '../../models/namedvalue';
import { StorageDiskInfo, StorageLocation } from '../../models/storage.model';
import { StorageService } from '../../services/storage.service';
import { DropdownComponent } from '../dropdown/dropdown.component';

type DiskOption = NamedValue<string> & { disk: StorageDiskInfo };

@Component({
    selector: 'mtx-storage-editor',
    templateUrl: 'storage.editor.component.html',
    imports: [ReactiveFormsModule, DropdownComponent, FormatSizePipe],
})
export class StorageEditorComponent {
    private static readonly DEFAULT_FOLDER = 'Nimble Photos';

    private readonly formBuilder = inject(FormBuilder);
    private readonly storageService = inject(StorageService);

    readonly disks = input<StorageDiskInfo[]>([]);
    readonly showCreateForm = computed(() => this.storage() !== null);
    readonly storage = model<StorageLocation | null>(null);
    readonly isSaving = signal(false);

    readonly cancelled = output<void>();
    readonly storageCreated = output<StorageLocation>();
    readonly storageUpdated = output<StorageLocation[]>();
    readonly saveError = output<string>();

    readonly selectedDiskMount = signal('');

    readonly form = this.formBuilder.nonNullable.group({
        label: ['', [Validators.required, Validators.minLength(2)]],
        diskMount: ['', [Validators.required]],
        folderName: [StorageEditorComponent.DEFAULT_FOLDER, [Validators.required, Validators.minLength(2)]],
        categoryTemplate: ['date', [Validators.required]],
    });

    readonly isEditing = computed(() => this.storage() && this.storage()?.id !== '');
    readonly isVisible = computed(() => this.showCreateForm() || this.isEditing());
    readonly panelTitle = computed(() => this.isEditing() ? 'Edit' : 'Create');
    readonly panelDescription = computed(() =>
        this.isEditing()
            ? 'Update the storage label and location. The folder name will be set under the selected mount.'
            : 'Name the storage and choose a disk. The folder name will be created under the selected mount.',
    );
    readonly submitLabel = computed(() => this.isEditing() ? 'Save' : 'Create');

    readonly selectedDisk = computed(() => {
        const mount = this.selectedDiskMount();
        if (!mount) {
            return null;
        }
        return this.disks().find(disk => disk.mountPoint === mount) || null;
    });
    readonly diskOptions = computed<DiskOption[]>(() =>
        this.disks().map((disk) => ({
            name: disk.name,
            value: disk.mountPoint,
            disk,
        }))
    );
    readonly categoryTemplates: readonly NamedValue<string>[] = [
        { value: 'date', name: 'Date' },
        { value: 'hash', name: 'Hash' },
    ];
    readonly getDiskLabel = (option: NamedValue<unknown>): string => {
        const disk = (option as DiskOption).disk;
        return `${disk.name} (${disk.mountPoint})`;
    };

    constructor() {
        effect(() => {
            const storage = this.storage();
            if (storage) {
                const diskMount = this.resolveDiskMount(storage.path);
                this.form.reset({
                    label: storage.label,
                    diskMount,
                    folderName: this.extractFolderName(storage.path, diskMount),
                    categoryTemplate: storage.categoryTemplate || 'date',
                });
                this.selectedDiskMount.set(diskMount);
                return;
            }
            if (!this.showCreateForm()) {
                return;
            }
            const firstDiskMount = this.disks()[0]?.mountPoint ?? '';
            this.form.reset({
                label: '',
                diskMount: firstDiskMount,
                folderName: StorageEditorComponent.DEFAULT_FOLDER,
                categoryTemplate: 'date',
            });
            this.selectedDiskMount.set(firstDiskMount);
        });
    }

    submit(): void {
        if (this.form.invalid || this.isSaving()) {
            this.form.markAllAsTouched();
            return;
        }

        if (this.isEditing()) {
            this.saveEdit();
            return;
        }
        this.createLocation();
    }

    cancel(): void {
        this.cancelled.emit();
    }

    onDiskMountChange(value: unknown): void {
        this.selectedDiskMount.set(String(value ?? ''));
    }

    private createLocation(): void {
        this.isSaving.set(true);
        const payload = this.form.getRawValue();
        this.storageService
            .createStorage({
                label: payload.label.trim(),
                mountPoint: payload.diskMount,
                path: payload.folderName,
                categoryTemplate: payload.categoryTemplate,
            })
            .pipe(finalize(() => this.isSaving.set(false)))
            .subscribe({
                next: (location) => {
                    this.storageCreated.emit(location);
                    this.cancelled.emit();
                },
                error: (err) => {
                    this.saveError.emit(err.error?.message || 'Failed to add storage location.');
                },
            });
    }

    private saveEdit(): void {
        const location = this.storage();
        if (!location) {
            return;
        }

        this.isSaving.set(true);
        const payload = this.form.getRawValue();
        const path = this.buildPath(payload.diskMount, payload.folderName);

        this.storageService
            .updateStorage(location.id, {
                label: payload.label.trim(),
                path: path.trim(),
                categoryTemplate: payload.categoryTemplate,
            })
            .pipe(finalize(() => this.isSaving.set(false)))
            .subscribe({
                next: (locations) => {
                    this.storageUpdated.emit(locations);
                    this.cancelled.emit();
                },
                error: (err) => {
                    this.saveError.emit(err.error?.message || 'Failed to update storage.');
                },
            });
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
