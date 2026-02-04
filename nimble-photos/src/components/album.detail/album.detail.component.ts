import { Component, OnInit, signal, computed, inject } from '@angular/core';
import { ActivatedRoute, RouterModule, Router } from '@angular/router';
import { CommonModule, DatePipe } from '@angular/common';
import { first } from 'rxjs';
import { PhotoService } from '../../services/photo.service';
import { AuthService } from '../../services/auth.service';
import { DialogService } from '../../services/dialog.service';
import { Album, Photo } from '../../models/photo';
import { GalleryComponent } from '../gallery/gallery.component';
import { SelectionService } from '../../services/selection.service';
import { ConfirmDialogComponent } from '../shared/confirm-dialog/confirm-dialog.component';

@Component({
    selector: 'mtx-album-detail',
    imports: [CommonModule, RouterModule, DatePipe, GalleryComponent],
    templateUrl: './album.detail.component.html',
    host: {
        class: 'flex flex-col flex-1 min-h-0',
    },
})
export class AlbumDetailComponent implements OnInit {
    private readonly route = inject(ActivatedRoute);
    private readonly router = inject(Router);
    private readonly photoService = inject(PhotoService);
    private readonly authService = inject(AuthService);
    private readonly dialogService = inject(DialogService);
    private readonly selectionService = inject(SelectionService);

    readonly album = signal<Album | null>(null);
    readonly loading = signal(false);
    readonly isDeleting = signal(false);
    readonly isEditMode = signal(false);

    readonly selectedCount = computed(() => this.selectionService.selectedCount());

    readonly isAdmin = computed(() => this.authService.isAdmin());

    readonly albumPhotos = computed<Photo[]>(() => {
        return this.album()?.photos?.items ?? [];
    });

    constructor() { }

    ngOnInit(): void {
        this.route.paramMap.subscribe(params => {
            const id = params.get('id');
            if (id) {
                this.fetchAlbum(id);
            }
        });

        // Clear selection when entering album detail
        this.selectionService.clearSelection();
    }

    private fetchAlbum(id: string): void {
        this.loading.set(true);
        this.photoService.getAlbumById(id).pipe(first()).subscribe(result => {
            this.album.set(result);
            this.loading.set(false);
        });
    }

    toggleEditMode(): void {
        this.isEditMode.update(v => !v);
        this.selectionService.clearSelection();
    }

    getImageUrl(photo: Photo): string {
        return this.photoService.getThumbnailPath(photo);
    }

    getAspectRatio(photo: Photo): string {
        if (photo.width && photo.height) {
            return `${photo.width} / ${photo.height}`;
        }
        return '4 / 3';
    }

    removeSelectedPhotos(): void {
        const currentAlbum = this.album();
        const selectedPhotos = this.selectionService.selectedPhotos();

        if (!currentAlbum || selectedPhotos.length === 0) return;

        const dialogRef = this.dialogService.open(ConfirmDialogComponent, {
            title: 'Remove Photos',
            data: {
                message: `Are you sure you want to remove ${selectedPhotos.length} photos from this album?`,
                type: 'danger'
            },
            actions: [
                { label: 'Cancel', value: false, style: 'ghost' },
                { label: 'Remove', value: true, style: 'danger' }
            ]
        });

        dialogRef.afterClosed().then(confirmed => {
            if (confirmed) {
                this.performRemoval(currentAlbum, selectedPhotos);
            }
        });
    }

    private performRemoval(album: Album, photosToRemove: Photo[]) {
        let currentIds: string[] = [];
        if (album.rulesJson) {
            try {
                const rules = JSON.parse(album.rulesJson);
                currentIds = rules.photoIds || [];
            } catch (e) {
                console.error('Error parsing album rules', e);
            }
        }

        const idsToRemove = new Set(photosToRemove.map(p => p.id));
        const newIds = currentIds.filter(id => !idsToRemove.has(id));

        this.photoService.updateAlbum({
            id: album.id,
            name: album.name,
            description: album.description,
            kind: album.kind,
            sortOrder: album.sortOrder,
            rulesJson: JSON.stringify({ photoIds: newIds })
        }).subscribe({
            next: (updatedAlbum) => {
                // We need to refresh the album because the backend might return the updated album object
                // but we specifically need to update the photos list locally or re-fetch.
                // Re-fetching is safer to get the correct paged view.
                this.fetchAlbum(album.id!);
                this.selectionService.clearSelection();
            },
            error: (err) => {
                console.error('Failed to remove photos', err);
                alert('Failed to remove photos from album.');
            }
        });
    }

    deleteAlbum(): void {
        const id = this.album()?.id;
        if (!id) return;

        const dialogRef = this.dialogService.open(ConfirmDialogComponent, {
            title: 'Delete Album',
            data: {
                message: 'Are you sure you want to delete this album? This action cannot be undone.',
                type: 'danger'
            },
            actions: [
                { label: 'Cancel', value: false, style: 'ghost' },
                { label: 'Delete Album', value: true, style: 'danger' }
            ]
        });

        dialogRef.afterClosed().then(confirmed => {
            if (confirmed) {
                this.isDeleting.set(true);
                this.photoService.deleteAlbum(id).pipe(first()).subscribe({
                    next: () => {
                        this.router.navigate(['/']);
                    },
                    error: (err) => {
                        console.error('Failed to delete album', err);
                        this.isDeleting.set(false);
                        alert('Failed to delete album.');
                    }
                });
            }
        });
    }
}
