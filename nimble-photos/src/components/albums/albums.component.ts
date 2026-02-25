import { DatePipe } from '@angular/common';
import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { RouterModule } from '@angular/router';
import { first } from 'rxjs';
import { Album } from '../../models/photo';
import { AuthService } from '../../services/auth.service';
import { DialogService } from '../../services/dialog.service';
import { PhotoService } from '../../services/photo.service';

import { ImageFallbackDirective } from '../../directives/image.fallback.directive';
import { ConfirmDialogComponent } from '../shared/confirm-dialog/confirm.dialog.component';
import { SvgComponent } from '../svg/svg.component';

@Component({
    selector: 'mtx-albums',
    imports: [RouterModule, DatePipe, ImageFallbackDirective, SvgComponent],
    templateUrl: './albums.component.html',
    host: {
        class: 'block flex-1 min-h-0',
    },
})
export class AlbumsComponent implements OnInit {
    private readonly photoService = inject(PhotoService);
    private readonly authService = inject(AuthService);
    private readonly dialogService = inject(DialogService);
    private readonly pageSize = 12;
    private page = 1;

    readonly albums = signal<Album[]>([]);
    readonly searchQuery = signal('');
    readonly isLoading = signal(false);
    readonly isLoadingMore = signal(false);
    readonly hasMore = signal(true);

    readonly isAdmin = computed(() => this.authService.isAdmin());
    readonly albumsByDateDesc = computed(() =>
        [...this.albums()].sort((a, b) => {
            const left = a.createDate?.getTime() ?? 0;
            const right = b.createDate?.getTime() ?? 0;
            return right - left;
        }),
    );
    readonly filteredAlbums = computed(() => {
        const query = this.searchQuery().trim().toLowerCase();
        if (!query) {
            return this.albumsByDateDesc();
        }

        return this.albumsByDateDesc().filter((album) => {
            const name = album.name?.toLowerCase() ?? '';
            const description = album.description?.toLowerCase() ?? '';
            const category = album.category?.toLowerCase() ?? '';
            return name.includes(query) || description.includes(query) || category.includes(query);
        });
    });

    constructor() { }

    ngOnInit(): void {
        this.fetchAlbums(true);
    }

    private fetchAlbums(reset: boolean): void {
        if (reset) {
            this.page = 1;
            this.hasMore.set(true);
            this.isLoading.set(true);
        } else {
            this.isLoadingMore.set(true);
        }

        this.photoService.getAlbums(this.page, this.pageSize).pipe(first()).subscribe(result => {
            if (reset) {
                this.albums.set(result.items);
            } else {
                this.albums.update(items => [...items, ...result.items]);
            }
            const loaded = (this.page - 1) * this.pageSize + result.items.length;
            this.hasMore.set(loaded < result.total);
            this.page += 1;
            this.isLoading.set(false);
            this.isLoadingMore.set(false);
        }, () => {
            this.isLoading.set(false);
            this.isLoadingMore.set(false);
        });
    }

    loadMore(): void {
        if (this.isLoadingMore() || !this.hasMore()) {
            return;
        }
        this.fetchAlbums(false);
    }

    onContainerScroll(event: Event): void {
        if (this.searchQuery().trim().length || this.isLoading() || this.isLoadingMore() || !this.hasMore()) {
            return;
        }

        const element = event.target as HTMLElement | null;
        if (!element) {
            return;
        }
        const threshold = 350;
        const viewportBottom = element.scrollTop + element.clientHeight;
        const docHeight = element.scrollHeight;
        if (docHeight - viewportBottom <= threshold) {
            this.loadMore();
        }
    }

    getThumbnailUrl(album: Album): string | null {
        if (!album.thumbnailHash)
            return null;
        return `${this.photoService.apiBase}/photos/thumbnail/${album.thumbnailHash}`;
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
                        if (!this.albums().length && this.hasMore()) {
                            this.fetchAlbums(true);
                        }
                    },
                    error: (err) => {
                        console.error('Failed to delete album', err);
                        alert('Failed to delete album.');
                    }
                });
            }
        });
    }

    onSearchInput(event: Event): void {
        const input = event.target as HTMLInputElement | null;
        this.searchQuery.set(input?.value ?? '');
    }
}
