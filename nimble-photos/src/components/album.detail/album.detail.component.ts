import { Component, OnInit, signal, computed, inject } from '@angular/core';
import { ActivatedRoute, RouterModule, Router } from '@angular/router';
import { CommonModule, DatePipe } from '@angular/common';
import { first } from 'rxjs';
import { PhotoService } from '../../services/photo.service';
import { AuthService } from '../../services/auth.service';
import { DialogService } from '../../services/dialog.service';
import { Album, Photo } from '../../models/photo';
import { GalleryComponent } from '../gallery/gallery.component';
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

    readonly album = signal<Album | null>(null);
    readonly loading = signal(false);
    readonly isDeleting = signal(false);

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
    }

    private fetchAlbum(id: string): void {
        this.loading.set(true);
        this.photoService.getAlbumById(id).pipe(first()).subscribe(result => {
            this.album.set(result);
            this.loading.set(false);
        });
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
