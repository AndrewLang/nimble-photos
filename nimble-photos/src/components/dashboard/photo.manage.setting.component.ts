import { Component, computed, inject, signal } from '@angular/core';

import { DashboardSettingsService } from '../../services/dashboard.setting.service';
import { PhotoService } from '../../services/photo.service';

@Component({
    selector: 'mtx-photo-manage-setting',
    standalone: true,
    templateUrl: './photo.manage.setting.component.html',
})
export class PhotoManageSettingComponent {
    private readonly settingsService = inject(DashboardSettingsService);
    private readonly photoService = inject(PhotoService);

    readonly isDragActive = signal(false);
    readonly selectedFiles = signal<File[]>([]);
    readonly uploadError = signal<string | null>(null);
    readonly uploading = signal(false);
    readonly uploadSuccess = signal(false);
    readonly supportedExtensions = [
        'jpg',
        'jpeg',
        'png',
        'heic',
        'heif',
        'webp',
        'gif',
        'tiff',
        'bmp',
        'raw',
        'dng',
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

    constructor() {
        this.settingsService.ensureLoaded();
    }

    onDragOver(event: DragEvent): void {
        event.preventDefault();
        this.isDragActive.set(true);
    }

    onDragLeave(event: DragEvent): void {
        event.preventDefault();
        this.isDragActive.set(false);
    }

    onDrop(event: DragEvent): void {
        event.preventDefault();
        this.isDragActive.set(false);
        if (event.dataTransfer?.files?.length) {
            this.addFiles(event.dataTransfer.files);
        }
    }

    onFileSelected(event: Event): void {
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
    }

    removeFile(target: File): void {
        const remaining = this.selectedFiles().filter(
            file => !(file.name === target.name && file.size === target.size),
        );
        this.selectedFiles.set(remaining);
    }

    uploadFiles(): void {
        const files = this.selectedFiles();
        if (!files.length || this.uploading()) {
            return;
        }

        this.uploading.set(true);
        this.uploadError.set(null);
        this.uploadSuccess.set(false);

        this.photoService.uploadPhotos(files).subscribe({
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

    formatBytes(bytes: number): string {
        if (!bytes || bytes <= 0) return '0 B';
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
        const value = bytes / Math.pow(1024, index);
        return `${value.toFixed(value >= 10 || index === 0 ? 0 : 1)} ${units[index]}`;
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
