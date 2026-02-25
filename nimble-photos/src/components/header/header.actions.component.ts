import { Component, computed, inject, signal } from '@angular/core';
import { Router } from '@angular/router';
import { Command } from '../../models/command';
import { DialogService } from '../../services/dialog.service';
import { PhotoService } from '../../services/photo.service';
import { SelectionService } from '../../services/selection.service';
import { AlbumEditorComponent } from '../album/album.editor.component';
import { AlbumSelectorComponent } from '../album/album.selector.component';
import { ConfirmDialogComponent } from '../shared/confirm-dialog/confirm.dialog.component';
import { SvgComponent } from '../svg/svg.component';
import { TagEditorComponent } from '../tag/tag.editor.component';

@Component({
    selector: 'mtx-header-actions',
    templateUrl: 'header.actions.component.html',
    imports: [SvgComponent],
})
export class HeaderActionsComponent {
    private readonly router = inject(Router);
    private readonly dialogService = inject(DialogService);
    private readonly selectionService = inject(SelectionService);
    private readonly photoService = inject(PhotoService);

    readonly photoCommands = signal<Command[]>([
        {
            id: 'createAlbum',
            name: 'Create Album',
            description: 'Create a new album with the selected photos',
            icon: 'plus',
            action: () => {
                void this.createAlbum();
            }
        },
        {
            id: 'addToAlbum',
            name: 'Add to Album',
            description: 'Add the selected photos to an existing album',
            icon: 'folderPlus',
            action: () => {
                void this.addToAlbum();
            }
        },
        {
            id: 'tagPhotos',
            name: 'Tag Photos',
            description: 'Add tags to the selected photos',
            icon: 'tag',
            action: () => this.tagPhotos()
        },
        {
            id: 'deletePhotos',
            name: 'Delete Photos',
            description: 'Delete the selected photos',
            icon: 'trash',
            action: () => {
                void this.deletePhotos();
            }
        },
        {
            id: 'downloadPhotos',
            name: 'Download Photos',
            description: 'Download the selected photos',
            icon: 'download',
            isHidden: true,
            action: () => this.downloadSelected()
        }
    ]);

    readonly selectionCommands = computed(() => this.photoCommands().filter(command => !command.isHidden));
    readonly hasSelection = computed(() => this.selectionService.hasSelection());
    readonly selectionCount = computed(() => this.selectionService.selectedPhotos().length);

    clearSelection(): void {
        this.selectionService.clearSelection();
    }

    async createAlbum(): Promise<void> {
        const photos = this.selectionService.selectedPhotos();
        const ref = this.dialogService.open(AlbumEditorComponent, {
            title: 'Create New Album',
            width: '600px',
            data: { photos },
            actions: [
                { label: 'Cancel', value: false, style: 'ghost' },
                { label: 'Create Album', value: 'submit', style: 'primary' }
            ]
        });

        const result = await ref.afterClosed();
        if (!result || result === 'submit' || result === false) {
            return;
        }

        const albumData = result;
        const photoIds: string[] = Array.isArray(albumData.photoIds) ? albumData.photoIds : [];
        this.photoService
            .createAlbum({
                name: albumData.name,
                description: albumData.description,
                kind: 'manual',
                sortOrder: 0
            })
            .subscribe({
                next: album => {
                    if (!photoIds.length) {
                        this.selectionService.clearSelection();
                        this.router.navigate(['/albums', album.id]);
                        return;
                    }
                    this.photoService.addPhotosToAlbum(album.id, photoIds).subscribe({
                        next: () => {
                            this.selectionService.clearSelection();
                            this.router.navigate(['/albums', album.id]);
                        },
                        error: err => {
                            console.error('Failed to add photos to created album:', err);
                            alert('Album created, but adding photos failed.');
                        }
                    });
                },
                error: err => {
                    console.error('Failed to create album:', err);
                    alert('Failed to create album. Please try again.');
                }
            });
    }

    async addToAlbum(): Promise<void> {
        const photos = this.selectionService.selectedPhotos();
        if (photos.length === 0) {
            return;
        }

        const ref = this.dialogService.open(AlbumSelectorComponent, {
            title: 'Add to Album',
            width: '500px',
            actions: [
                { label: 'Cancel', value: false, style: 'ghost' },
                { label: 'Add to Album', value: 'submit', style: 'primary' }
            ]
        });

        const result = await ref.afterClosed();
        if (!result || result === 'submit' || result === false) {
            return;
        }

        const targetAlbum = result;
        const photoIds = photos.map(photo => photo.id);
        this.photoService.addPhotosToAlbum(targetAlbum.id!, photoIds).subscribe({
            next: () => {
                this.selectionService.clearSelection();
                this.router.navigate(['/albums', targetAlbum.id]);
            },
            error: err => {
                console.error('Failed to add photos to album', err);
                alert('Failed to add photos to album.');
            }
        });
    }

    downloadSelected(): void {
        const photos = this.selectionService.selectedPhotos();
        photos.forEach(photo => {
            const link = document.createElement('a');
            link.href = this.photoService.getDownloadPath(photo);
            link.download = photo.name;
            link.click();
        });
    }

    tagPhotos(): void {
        const photos = this.selectionService.selectedPhotos();
        if (photos.length === 0) {
            return;
        }

        const ref = this.dialogService.open(TagEditorComponent, {
            title: 'Tag Photos',
            width: '700px',
            actions: [
                { label: 'Cancel', value: false, style: 'ghost' },
                { label: 'Apply', value: 'submit', style: 'primary' }
            ]
        });

        ref.afterClosed().then(result => {
            if (!result || result === 'submit' || result === false) {
                return;
            }

            this.photoService.updatePhotoTags(result.photoIds, result.tags).subscribe({
                next: () => {
                    this.selectionService.clearSelection();
                },
                error: err => {
                    console.error('Failed to update tags', err);
                    alert('Failed to update photo tags.');
                }
            });
        });
    }

    async deletePhotos(): Promise<void> {
        const photos = this.selectionService.selectedPhotos();
        if (photos.length === 0) {
            return;
        }

        const dialogRef = this.dialogService.open(ConfirmDialogComponent, {
            title: 'Delete Photos',
            data: {
                message: `Are you sure you want to delete ${photos.length} selected photo${photos.length === 1 ? '' : 's'}? This action cannot be undone.`,
                type: 'danger'
            },
            actions: [
                { label: 'Cancel', value: false, style: 'ghost' },
                { label: 'Delete', value: true, style: 'danger' }
            ]
        });

        const confirmed = await dialogRef.afterClosed();
        if (!confirmed) {
            return;
        }

        this.photoService.deletePhotos(photos.map(photo => photo.id)).subscribe({
            next: () => {
                this.selectionService.clearSelection();
                this.photoService.requestTimelineRefresh();
            },
            error: err => {
                console.error('Failed to delete photos', err);
                alert('Failed to delete photos.');
            }
        });
    }
}
