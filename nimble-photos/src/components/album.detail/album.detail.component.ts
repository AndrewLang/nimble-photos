import { CommonModule, DatePipe } from '@angular/common';
import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
import { catchError, first, of } from 'rxjs';
import { PagedModel } from '../../models/paged.response.model';
import { Album, AlbumComment, Photo } from '../../models/photo';
import { AuthService } from '../../services/auth.service';
import { DialogService } from '../../services/dialog.service';
import { PhotoService } from '../../services/photo.service';
import { SelectionService } from '../../services/selection.service';
import { SettingsService } from '../../services/settings.service';
import { GalleryComponent } from '../gallery/gallery.component';
import { ConfirmDialogComponent } from '../shared/confirm-dialog/confirm.dialog.component';

const MAX_COMMENT_LENGTH = 1024;

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
    private readonly settingsService = inject(SettingsService);

    readonly album = signal<Album | null>(null);
    readonly loading = signal(false);
    readonly isDeleting = signal(false);
    readonly isEditMode = signal(false);
    readonly selectedCount = computed(() => this.selectionService.selectedCount());
    readonly isAdmin = computed(() => this.authService.isAdmin());
    readonly albumPhotos = computed<Photo[]>(() => {
        return this.album()?.photos?.items ?? [];
    });

    readonly albumComments = signal<PagedModel<AlbumComment> | null>(null);
    readonly commentsLoading = signal(false);
    readonly commentsError = signal<string | null>(null);
    readonly commentDraft = signal('');
    readonly commentSaving = signal(false);
    readonly commentError = signal<string | null>(null);
    readonly commentEditorVisible = signal(false);
    readonly maxCommentLength = MAX_COMMENT_LENGTH;
    readonly isAuthenticated = computed(() => this.authService.isAuthenticated());
    readonly allowComments = signal(false);
    readonly sidebarOpen = signal(false);

    constructor() { }

    get albumCommentsList(): AlbumComment[] {
        return this.albumComments()?.items ?? [];
    }

    ngOnInit(): void {
        this.route.paramMap.subscribe(params => {
            const id = params.get('id');
            if (id) {
                this.fetchAlbum(id);
            }
        });

        this.settingsService.getSettingByName('site.allowComments')
            .pipe(
                first(),
                catchError(() => of(null))
            )
            .subscribe(setting => {
                const raw = setting?.value;
                const enabled = typeof raw === 'boolean'
                    ? raw
                    : typeof raw === 'string'
                        ? raw.toLowerCase() === 'true'
                        : typeof raw === 'number'
                            ? raw !== 0
                            : false;
                this.allowComments.set(enabled);
                if (!enabled) {
                    this.commentEditorVisible.set(false);
                }
            });

        this.selectionService.clearSelection();
    }

    private fetchAlbum(id: string): void {
        this.loading.set(true);
        this.photoService.getAlbumById(id).pipe(first()).subscribe(result => {
            this.album.set(result);
            if (result?.id) {
                this.loadAlbumComments(result.id);
            } else {
                this.albumComments.set(null);
            }
            this.commentEditorVisible.set(false);
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

    handleCommentInput(event: Event): void {
        const target = event.target as HTMLTextAreaElement;
        this.commentDraft.set(target.value.slice(0, MAX_COMMENT_LENGTH));
    }

    saveAlbumComment(): void {
        const album = this.album();
        if (!album || !this.authService.isAuthenticated() || !this.allowComments()) {
            return;
        }

        const trimmed = this.commentDraft().trim();
        if (trimmed.length === 0 || trimmed.length > MAX_COMMENT_LENGTH) {
            this.commentError.set(`Comment must be between 1 and ${MAX_COMMENT_LENGTH} characters.`);
            return;
        }

        this.commentSaving.set(true);
        this.commentError.set(null);

        this.photoService.createAlbumComment(album.id, trimmed)
            .pipe(first())
            .subscribe({
                next: () => {
                    this.commentSaving.set(false);
                    this.commentDraft.set('');
                    this.commentEditorVisible.set(false);
                    this.loadAlbumComments(album.id);
                },
                error: () => {
                    this.commentSaving.set(false);
                    this.commentError.set('Unable to save your comment.');
                }
            });
    }

    toggleCommentEditor(): void {
        if (!this.allowComments() || !this.authService.isAuthenticated()) {
            return;
        }
        this.commentEditorVisible.update(value => !value);
    }

    toggleSidebar(): void {
        this.sidebarOpen.update(value => !value);
    }

    private loadAlbumComments(albumId: string): void {
        this.commentsLoading.set(true);
        this.commentsError.set(null);
        this.photoService.getAlbumComments(albumId)
            .pipe(first())
            .subscribe({
                next: comments => {
                    this.albumComments.set(comments);
                    this.commentsLoading.set(false);
                    const hasComments = (comments?.items.length ?? 0) > 0;
                    this.sidebarOpen.set(hasComments);
                },
                error: () => {
                    this.albumComments.set(null);
                    this.commentsLoading.set(false);
                    this.commentsError.set('Failed to load comments.');
                    this.sidebarOpen.set(false);
                }
            });
    }

    hideAlbumComment(comment: AlbumComment): void {
        const album = this.album();
        if (!album) {
            return;
        }

        this.photoService.updateAlbumCommentVisibility(album.id, comment.id, !comment.hidden)
            .pipe(first())
            .subscribe({
                next: () => this.loadAlbumComments(album.id),
                error: () => console.error('Failed to update comment visibility'),
            });
    }

    formatCommentDate(value?: string): string {
        if (!value) {
            return '';
        }
        const parsed = new Date(value);
        if (Number.isNaN(parsed.getTime())) {
            return value;
        }
        return parsed.toLocaleDateString('en-US', {
            month: 'short',
            day: 'numeric',
            year: 'numeric',
        });
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

