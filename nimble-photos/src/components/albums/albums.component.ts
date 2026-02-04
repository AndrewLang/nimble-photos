import { Component, OnInit, signal, computed, inject } from '@angular/core';
import { RouterModule } from '@angular/router';
import { first } from 'rxjs';
import { PhotoService } from '../../services/photo.service';
import { AuthService } from '../../services/auth.service';
import { DialogService } from '../../services/dialog.service';
import { Album } from '../../models/photo';
import { DatePipe } from '@angular/common';

import { ConfirmDialogComponent } from '../shared/confirm-dialog/confirm-dialog.component';

@Component({
    selector: 'mtx-albums',
    imports: [RouterModule, DatePipe],
    templateUrl: './albums.component.html',
    host: {
        class: 'block flex-1 min-h-0',
    },
})
export class AlbumsComponent implements OnInit {
    private readonly photoService = inject(PhotoService);
    private readonly authService = inject(AuthService);
    private readonly dialogService = inject(DialogService);

    readonly albums = signal<Album[]>([]);
    readonly loading = signal(false);

    readonly isAdmin = computed(() => this.authService.isAdmin());

    constructor() { }

    ngOnInit(): void {
        this.fetchAlbums();
    }

    private fetchAlbums(): void {
        this.loading.set(true);
        this.photoService.getAlbums().pipe(first()).subscribe(result => {
            this.albums.set(result.items);
            this.loading.set(false);
        });
    }

    getThumbnailUrl(album: Album): string | null {
        if (!album.thumbnailHash) return null;
        return `${(this.photoService as any).apiBase}/photos/thumbnail/${album.thumbnailHash}`;
    }

    deleteAlbum(event: MouseEvent, album: Album): void {
        event.preventDefault();
        event.stopPropagation();

        if (!album.id)
            return;

        const dialogRef = this.dialogService.open(ConfirmDialogComponent, {
            title: 'Delete Album',
            data: {
                message: `Are you sure you want to delete the album "${album.name}"?`,
                type: 'danger'
            },
            actions: [
                { label: 'Cancel', value: false, style: 'ghost' },
                { label: 'Delete', value: true, style: 'danger' }
            ]
        });

        dialogRef.afterClosed().then(confirmed => {
            if (confirmed) {
                this.photoService.deleteAlbum(album.id!).pipe(first()).subscribe({
                    next: () => {
                        this.albums.update(items => items.filter(a => a.id !== album.id));
                    },
                    error: (err) => {
                        console.error('Failed to delete album', err);
                        alert('Failed to delete album.');
                    }
                });
            }
        });
    }
}
