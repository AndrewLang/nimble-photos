import { Component, computed, effect, inject, OnInit, signal } from '@angular/core';
import { RouterModule } from '@angular/router';

import { FormatSizePipe } from '../../directives/format.size.pipe';
import { AsyncLoader } from '../../models/resource.loader';
import { StorageLocation } from '../../models/storage.model';
import { DialogService } from '../../services/dialog.service';
import { PhotoService } from '../../services/photo.service';
import { StorageService } from '../../services/storage.service';
import { SpinnerComponent } from '../spinner/spinner.component';
import { StorageSelectorComponent } from '../storage/storage.selector.component';
import { SvgComponent } from '../svg/svg.component';


@Component({
    selector: 'mtx-photo-upload',
    templateUrl: 'photo.upload.component.html',
    imports: [RouterModule, SvgComponent, FormatSizePipe, SpinnerComponent],
})
export class PhotoUploadComponent implements OnInit {
    readonly supportedExtensions = [
        'jpg', 'jpeg', 'png', 'heic', 'heif', 'webp', 'gif', 'tiff', 'bmp',
        'cr2', 'cr3', 'nef', 'arw', 'dng', 'orf', 'raf', 'rw2', 'pef', 'srw',
    ];

    private readonly photoService = inject(PhotoService);
    private readonly storageService = inject(StorageService);
    private readonly dialogService = inject(DialogService);

    readonly isDragActive = signal(false);
    readonly selectedFiles = signal<File[]>([]);
    readonly uploadError = signal<string | null>(null);
    readonly uploading = signal(false);
    readonly uploadSuccess = signal(false);

    readonly canUpload = computed(() => !!this.selectedStorage());

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

    readonly storages = new AsyncLoader<StorageLocation[]>();
    readonly storageLoading = computed(() => this.storages.loading());
    readonly selectedStorage = signal<StorageLocation | null>(null);
    readonly hasStorages = computed(() => this.storages.value()?.length || 0 > 0);

    private readonly storageEffect = effect(() => {
        this.loadStorages();
    });


    ngOnInit(): void {
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
    }

    removeFile(target: File): void {
        const remaining = this.selectedFiles()
            .filter(file => !(file.name === target.name && file.size === target.size));
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

        this.photoService.uploadPhotos(files, this.selectedStorage()?.id)
            .subscribe({
                next: () => {
                    this.uploadSuccess.set(true);
                    this.selectedFiles.set([]);
                },
                error: (err) => {
                    this.uploadError.set(err?.message || 'Upload file failed.');
                },
                complete: () => {
                    this.uploading.set(false);
                },
            });
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

    private loadStorages(): void {
        this.storages.load(() =>
            this.storageService.getStorages(),
            (storages) => {
                const defaultStorage = storages.find(x => x.isDefault) ?? null;
                if (defaultStorage && !this.selectedStorage()) {
                    this.selectedStorage.set(defaultStorage);
                }
            },
            'Failed to load storage locations.',
        );
    }

    private addFiles(list: FileList): void {
        const incoming = Array.from(list);
        if (!incoming.length)
            return;

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
        if (parts.length < 2)
            return false;
        const ext = parts[parts.length - 1];
        return this.supportedExtensions.includes(ext);
    }
}
