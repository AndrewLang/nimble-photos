import { Component, OnInit, computed, effect, inject, signal } from '@angular/core';
import { RouterModule } from '@angular/router';

import { Formatter } from '../../models/formatters';
import { StorageLocation } from '../../models/storage.model';
import { DashboardSettingsService } from '../../services/dashboard.setting.service';
import { DialogService } from '../../services/dialog.service';
import { PhotoService } from '../../services/photo.service';
import { StorageService } from '../../services/storage.service';
import { StorageSelectorComponent } from '../storage/storage.selector.component';
import { ActionSelectorComponent } from './action.selector.component';

@Component({
    selector: 'mtx-photo-manage-setting',
    imports: [RouterModule, ActionSelectorComponent],
    templateUrl: './photo.manage.setting.component.html',
})
export class PhotoManageSettingComponent implements OnInit {
    private readonly settingsService = inject(DashboardSettingsService);
    private readonly photoService = inject(PhotoService);
    private readonly storageService = inject(StorageService);
    private readonly dialogService = inject(DialogService);

    readonly isDragActive = signal(false);
    readonly selectedFiles = signal<File[]>([]);
    readonly uploadError = signal<string | null>(null);
    readonly uploading = signal(false);
    readonly uploadSuccess = signal(false);
    readonly storageLoading = signal(false);
    readonly storageLocations = signal<StorageLocation[]>([]);
    readonly selectedStorage = signal<StorageLocation | null>(null);
    readonly availableTags = signal<string[]>([]);
    readonly viewerHiddenTags = signal<string[]>([]);
    readonly tagsLoading = signal(false);
    readonly tagVisibilityError = signal<string | null>(null);
    readonly savingTagVisibility = signal(false);
    readonly tagVisibilitySaved = signal(false);
    readonly activeTab = signal<'upload' | 'visibility'>('upload');
    readonly supportedExtensions = [
        'jpg', 'jpeg', 'png', 'heic', 'heif', 'webp', 'gif', 'tiff', 'bmp',
        "cr2", "cr3", "nef", "arw", "dng", "orf", "raf", "rw2", "pef", "srw",
    ];

    readonly title = computed(() => {
        const value = this.settingsService.getSettingValue('dashboard.photo-manage.title');
        if (typeof value === 'string' && value.trim().length) {
            return value;
        }
        return 'Photo Management';
    });

    readonly subtitle = computed(() => {
        const value = this.settingsService.getSettingValue('dashboard.photo-manage.subtitle');
        if (typeof value === 'string' && value.trim().length) {
            return value;
        }
        return 'Upload new photos or drag folders to add them to your library.';
    });

    readonly canUpload = computed(() => !!this.selectedStorage());
    readonly hasStorages = computed(() => this.storageLocations().length > 0);
    readonly totalSelectedBytes = computed(() => {
        return this.selectedFiles().reduce((sum, file) => sum + file.size, 0);
    });
    readonly hasEnoughSpace = computed(() => {
        const storage = this.selectedStorage();
        if (!storage?.disk) {
            return true;
        }
        return this.totalSelectedBytes() <= storage.disk.availableBytes;
    });
    readonly canSubmitUpload = computed(() => {
        return this.canUpload() && this.hasEnoughSpace();
    });
    readonly tagOptions = computed(() => {
        const fromApi = this.availableTags();
        const selected = this.viewerHiddenTags();
        return Array.from(new Set([...fromApi, ...selected]))
            .sort((a, b) => a.localeCompare(b))
            .map(tag => ({ key: tag, label: tag }));
    });
    readonly formatBytes = Formatter.formatBytes;

    ngOnInit(): void {
        this.loadStorages();
        this.loadTags();
    }

    onDragOver(event: DragEvent): void {
        event.preventDefault();
        if (!this.canUpload()) {
            return;
        }
        this.isDragActive.set(true);
    }

    onDragLeave(event: DragEvent): void {
        event.preventDefault();
        this.isDragActive.set(false);
    }

    onDrop(event: DragEvent): void {
        event.preventDefault();
        this.isDragActive.set(false);
        if (!this.canUpload()) {
            return;
        }
        if (event.dataTransfer?.files?.length) {
            this.addFiles(event.dataTransfer.files);
        }
    }

    onFileSelected(event: Event): void {
        if (!this.canUpload()) {
            return;
        }

        const input = event.target as HTMLInputElement | null;
        if (input?.files?.length) {
            this.addFiles(input.files);
            input.value = '';
        }
    }

    clearFiles(): void {
        this.selectedFiles.set([]);
        this.uploadError.set(null);
        this.uploadSuccess.set(false);

        this.uploadError.set(null);
    }

    removeFile(target: File): void {
        const remaining = this.selectedFiles().filter(
            file => !(file.name === target.name && file.size === target.size),
        );
        this.selectedFiles.set(remaining);
    }

    uploadFiles(): void {
        const files = this.selectedFiles();
        if (!files.length || this.uploading() || !this.canSubmitUpload()) {
            return;
        }

        this.uploading.set(true);
        this.uploadError.set(null);
        this.uploadSuccess.set(false);

        this.photoService.uploadPhotos(files, this.selectedStorage()?.id).subscribe({
            next: () => {
                this.uploadSuccess.set(true);
                this.selectedFiles.set([]);
            },
            error: (err) => {
                this.uploadError.set(err?.message || 'Upload failed.');
            },
            complete: () => {
                this.uploading.set(false);
            },
        });
    }

    setActiveTab(tab: 'upload' | 'visibility'): void {
        this.activeTab.set(tab);
    }

    async openStorageSelector(): Promise<void> {
        if (!this.hasStorages()) {
            return;
        }

        const ref = this.dialogService.open(StorageSelectorComponent, {
            title: 'Select Storage',
            width: '560px',
            actions: [
                { label: 'Cancel', value: false, style: 'ghost' },
                { label: 'Use Storage', value: 'submit', style: 'primary' },
            ],
        });

        const result = await ref.afterClosed();
        if (result && result !== 'submit' && result !== false) {
            this.selectedStorage.set(result as StorageLocation);
        }
    }

    onViewerHiddenTagsChange(tags: string[]): void {
        const normalized = Array.from(new Set(tags.map(tag => tag.trim()).filter(tag => tag.length)))
            .sort((a, b) => a.localeCompare(b));
        this.viewerHiddenTags.set(normalized);
        this.tagVisibilityError.set(null);
        this.tagVisibilitySaved.set(false);
    }

    saveViewerHiddenTags(): void {
        const setting = this.settingsService.getSettingByName('photo.manage.viewerHiddenTags');
        if (!setting) {
            this.tagVisibilityError.set('Viewer hidden tags setting was not found.');
            return;
        }

        const payload = this.viewerHiddenTags();
        this.savingTagVisibility.set(true);
        this.tagVisibilityError.set(null);
        this.tagVisibilitySaved.set(false);

        this.settingsService.setLocalValue(setting.key, JSON.stringify(payload, null, 2));
        this.settingsService.saveSetting(setting);

        window.setTimeout(() => {
            this.savingTagVisibility.set(false);
            const fieldError = this.settingsService.fieldErrors()[setting.key];
            if (fieldError) {
                this.tagVisibilityError.set(fieldError);
                return;
            }
            this.tagVisibilitySaved.set(true);
            window.setTimeout(() => this.tagVisibilitySaved.set(false), 1800);
        }, 250);
    }

    private loadStorages(): void {
        this.storageLoading.set(true);
        this.storageService.getLocations().subscribe({
            next: (locations) => {
                this.storageLocations.set(locations);
                const defaultStorage = locations.find(location => location.isDefault) ?? null;
                if (defaultStorage && !this.selectedStorage()) {
                    this.selectedStorage.set(defaultStorage);
                }
                this.storageLoading.set(false);
            },
            error: () => {
                this.storageLocations.set([]);
                this.selectedStorage.set(null);
                this.storageLoading.set(false);
            },
        });
    }

    private loadTags(): void {
        this.tagsLoading.set(true);
        this.photoService.getAllPhotoTags().subscribe({
            next: tags => {
                const normalized = Array.from(new Set(tags.map(tag => tag.trim()).filter(Boolean)))
                    .sort((a, b) => a.localeCompare(b));
                this.availableTags.set(normalized);
                this.tagsLoading.set(false);
            },
            error: () => {
                this.availableTags.set([]);
                this.tagsLoading.set(false);
            },
        });
    }

    constructor() {
        this.settingsService.ensureLoaded();

        effect(() => {
            const setting = this.settingsService.getSettingByName('photo.manage.viewerHiddenTags');
            const raw = setting?.value;
            const parsed = Array.isArray(raw) ? raw : [];
            const normalized = Array.from(
                new Set(parsed.filter(item => typeof item === 'string').map(item => item.trim()).filter(Boolean)),
            ).sort((a, b) => a.localeCompare(b));
            this.viewerHiddenTags.set(normalized);
        });
    }

    private addFiles(list: FileList): void {
        const incoming = Array.from(list);
        if (!incoming.length) return;

        const existing = this.selectedFiles();
        const merged = [...existing];
        const rejected: string[] = [];
        for (const file of incoming) {
            if (!this.isSupported(file.name)) {
                rejected.push(file.name);
                continue;
            }
            if (!merged.some(current => current.name === file.name && current.size === file.size)) {
                merged.push(file);
            }
        }
        this.selectedFiles.set(merged);
        if (rejected.length) {
            this.uploadError.set(`Unsupported files skipped: ${rejected.join(', ')}`);
        }
    }

    private isSupported(filename: string): boolean {
        const parts = filename.toLowerCase().split('.');
        if (parts.length < 2) return false;
        const ext = parts[parts.length - 1];
        return this.supportedExtensions.includes(ext);
    }
}
