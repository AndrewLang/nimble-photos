
import { Component, HostListener, OnInit, inject, signal } from '@angular/core';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
import { first } from 'rxjs';
import { logger } from '../../models/logger';
import { Photo } from '../../models/photo';
import { PhotoMetadataProcessor } from '../../models/photo.metadata';
import { AuthService } from '../../services/auth.service';
import { LocalSettingService } from '../../services/local.setting.service';
import { PhotoService } from '../../services/photo.service';
import { SvgComponent } from '../svg/svg.component';
import { PhotoCommentComponent } from './photo.comment.component';

@Component({
    selector: 'mtx-photo-detail',
    imports: [RouterModule, SvgComponent, PhotoCommentComponent],
    templateUrl: './photo.detail.component.html',
    host: {
        class: 'fixed inset-0 z-[100] bg-slate-950 flex flex-col',
    }
})
export class PhotoDetailComponent implements OnInit {
    readonly photo = signal<Photo | null>(null);
    readonly loading = signal(false);
    readonly adjacents = signal<{ prevId: string | null; nextId: string | null }>({ prevId: null, nextId: null });
    readonly previewLoading = signal(false);
    readonly previewReady = signal(false);
    readonly previewSrc = signal<string | null>(null);

    readonly metadataExpanded = signal(false);
    readonly sidebarHidden = signal(false);

    private albumId: string | null = null;
    private returnUrl = '/';
    private previewRequestSeq = 0;

    private readonly route = inject(ActivatedRoute);
    private readonly router = inject(Router);
    readonly authService = inject(AuthService);
    private readonly photoService = inject(PhotoService);
    private readonly localSettingService = inject(LocalSettingService);

    private readonly photoMetadata = new PhotoMetadataProcessor();

    async ngOnInit() {
        const initialAlbumId = this.route.snapshot.paramMap.get('albumId');
        this.albumId = initialAlbumId;
        const navigationState = this.router.getCurrentNavigation()?.extras.state as { returnUrl?: string } | undefined;
        this.returnUrl = navigationState?.returnUrl ?? this.buildDefaultReturnUrl(initialAlbumId);

        this.route.paramMap.subscribe(params => {
            const id = params.get('id');
            this.albumId = params.get('albumId');
            if (id) {
                this.fetchPhoto(id);
            }
        });

        this.metadataExpanded.set(this.localSettingService.get('photo.detail.metadata.visible', false));
        this.sidebarHidden.set(this.localSettingService.get('photo.detail.sidebar.hidden', false));
    }

    @HostListener('window:keydown', ['$event'])
    handleKeyDown(event: KeyboardEvent): void {
        if (event.key === 'ArrowRight' && this.adjacents().nextId) {
            this.navigateToPhoto(this.adjacents().nextId!);
        } else if (event.key === 'ArrowLeft' && this.adjacents().prevId) {
            this.navigateToPhoto(this.adjacents().prevId!);
        } else if (event.key === 'Escape') {
            this.close();
        }
    }

    goHome() {
        this.router.navigateByUrl('/');
    }

    toggleMetadata(): void {
        this.metadataExpanded.update(value => !value);
        this.localSettingService.set('photo.detail.metadata.visible', this.metadataExpanded());
    }

    toggleSidebar(): void {
        this.sidebarHidden.update(value => !value);
        this.localSettingService.set('photo.detail.sidebar.hidden', this.sidebarHidden());
    }

    navigateToPhoto(id: string): void {
        const commands = this.albumId
            ? ['/album', this.albumId, 'photo', id]
            : ['/photo', id];
        this.router.navigate(commands, { state: { returnUrl: this.returnUrl } });
    }

    close(): void {
        const target = this.returnUrl ?? this.buildDefaultReturnUrl(this.albumId);
        this.router.navigateByUrl(target);
    }

    getPhotoPath(): string {
        return this.photoService.getThumbnailPath(this.photo()!);
    }

    getPreviewPath(): string {
        return this.previewSrc() ?? this.photoService.getPreviewPath(this.photo()!);
    }

    metadataSections(photo?: Photo | null): { title: string; fields: { label: string; value: string }[] }[] {
        if (photo && !photo?.metadata) {
            this.photoService.getPhotoMetadata(photo.id)
                .pipe(first())
                .subscribe(metadata => {
                    this.photo.update(current =>
                        current ? { ...current, metadata: metadata ?? undefined } : current
                    );

                    logger.debug('Loaded metadata for photo', photo?.id, metadata);
                });
        }
        let metadata = this.photoMetadata.buildMetadataSections(photo);
        logger.debug('Built metadata sections', metadata);
        return metadata;
    }

    private buildDefaultReturnUrl(albumId: string | null): string {
        return albumId ? `/album/${albumId}` : '/';
    }

    private fetchPhoto(id: string): void {
        this.loading.set(true);
        this.previewLoading.set(false);
        this.previewReady.set(false);
        this.previewSrc.set(null);

        this.photoService.getPhotoById(id).pipe(first()).subscribe(result => {
            this.photo.set(result);
            if (result) {
                this.fetchAdjacents(result.id);
                this.loadPhotoMetadata(result.id);
                this.loadPreview(result);
            }
            this.loading.set(false);
        });
    }

    private loadPreview(photo: Photo): void {
        const previewBasePath = this.photoService.getPreviewPath(photo);
        if (!photo.hash || previewBasePath === photo.path) {
            this.previewReady.set(false);
            this.previewLoading.set(false);
            this.previewSrc.set(null);
            return;
        }

        const seq = ++this.previewRequestSeq;
        this.previewLoading.set(true);
        this.previewReady.set(false);

        const previewRequestPath = `${previewBasePath}?v=${Date.now()}`;
        const image = new Image();
        image.onload = () => {
            if (seq !== this.previewRequestSeq) {
                return;
            }
            this.previewSrc.set(previewRequestPath);
            this.previewReady.set(true);
            this.previewLoading.set(false);
        };
        image.onerror = () => {
            if (seq !== this.previewRequestSeq) {
                return;
            }
            this.previewReady.set(false);
            this.previewLoading.set(false);
            this.previewSrc.set(null);
        };
        image.src = previewRequestPath;
    }

    private fetchAdjacents(id: string): void {
        logger.debug('Fetching adjacent', id)
        this.photoService.getAdjacentPhotos(id, this.albumId || undefined)
            .pipe(first())
            .subscribe({
                next: adj => {
                    logger.debug('Received adjacent', adj)
                    this.adjacents.set({
                        prevId: adj?.prevId ?? null,
                        nextId: adj?.nextId ?? null,
                    });
                },
                error: err => {
                    logger.error('Failed to fetch adjacent photos', err);
                    this.adjacents.set({ prevId: null, nextId: null });
                }
            });
    }

    private loadPhotoMetadata(photoId: string): void {
        this.photoService.getPhotoMetadata(photoId)
            .pipe(first())
            .subscribe(metadata => {
                this.photo.update(current =>
                    current ? { ...current, metadata: metadata ?? undefined } : current
                );
            });
    }
}
