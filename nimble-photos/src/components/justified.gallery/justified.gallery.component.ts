import { CdkVirtualScrollViewport, ScrollingModule } from '@angular/cdk/scrolling';
import { CommonModule, DatePipe } from '@angular/common';
import { AfterViewInit, Component, ElementRef, HostListener, OnInit, ViewChild, computed, effect, inject, input, output, signal } from '@angular/core';
import { Router, RouterModule } from '@angular/router';
import { first } from 'rxjs';

import { ImageFallbackDirective } from '../../directives/image.fallback.directive';
import { GroupedPhotos, Photo } from '../../models/photo';
import { PhotoService } from '../../services/photo.service';
import { SelectionService } from '../../services/selection.service';
import { SvgComponent } from '../svg/svg.component';

interface PhotoRow {
    photos: Photo[];
    height: number;
}

type GalleryItem =
    | { type: 'header'; title: string; count: number }
    | { type: 'row'; row: PhotoRow };

@Component({
    selector: 'mtx-justified-gallery',
    imports: [CommonModule, RouterModule, ScrollingModule, DatePipe, ImageFallbackDirective, SvgComponent],
    templateUrl: './justified.gallery.component.html',
    host: {
        class: 'block h-full w-full overflow-hidden'
    }
})
export class JustifiedGalleryComponent implements OnInit, AfterViewInit {
    @ViewChild('container') container?: ElementRef<HTMLElement>;
    @ViewChild(CdkVirtualScrollViewport) viewport?: CdkVirtualScrollViewport;

    readonly activeTitleChange = output<string>();
    readonly timelineLoaded = output<GroupedPhotos[]>();

    readonly showHeader = input(true);
    readonly selectionEnabled = input(true);
    readonly autoFetch = input(true);
    readonly albumId = input<string | undefined>(undefined);

    private readonly photoService = inject(PhotoService);
    private readonly selectionService = inject(SelectionService);
    readonly router = inject(Router);

    readonly selectedIds = computed(() => this.selectionService.selectedIds());
    readonly selectedPhotos = computed(() => this.selectionService.selectedPhotos());
    readonly isSelectionMode = computed(() => this.selectionService.hasSelection());

    readonly selectionChange = output<Photo[]>();

    private readonly _timeline = signal<GroupedPhotos[]>([]);
    readonly timeline = input<GroupedPhotos[]>([]);
    private readonly syncTimelineEffect = effect(() => {
        const value = this.timeline();
        this._timeline.set(value ?? []);
    });

    readonly containerWidth = signal<number>(0);
    readonly targetHeight = signal<number>(180);
    readonly gap = signal<number>(6);
    readonly isFetching = signal(false);
    readonly totalPhotos = signal(0);
    readonly loadedPhotosCount = computed(() =>
        this._timeline().reduce((acc, g) => acc + g.photos.items.length, 0)
    );

    readonly items = computed(() => {
        const groups = this._timeline();
        const containerW = this.containerWidth();
        const targetH = this.targetHeight();
        const g = this.gap();

        const availableW = Math.max(containerW - 32, 200);
        if (groups.length === 0) return [];

        const flattenedItems: GalleryItem[] = [];

        for (const group of groups) {
            // Add Header
            flattenedItems.push({
                type: 'header',
                title: group.title,
                count: group.photos.items.length
            });

            // Calculate rows for this group
            const photos = group.photos.items;
            let i = 0;
            while (i < photos.length) {
                let j = i;
                let currentBestJ = i;
                let minHeightDiff = Infinity;
                let runningWidthAtTargetH = 0;

                while (j < photos.length && j < i + 12) {
                    const photo = photos[j];
                    const isValid = !!(photo.width && photo.height && photo.width > 0 && photo.height > 0);
                    const ratio = isValid ? (photo.width! / photo.height!) : (4 / 3);
                    runningWidthAtTargetH += (targetH * ratio);

                    const totalGaps = (j - i) * g;
                    const rowHeight = ((availableW - totalGaps) / runningWidthAtTargetH) * targetH;
                    const diff = Math.abs(rowHeight - targetH);

                    if (diff < minHeightDiff) {
                        minHeightDiff = diff;
                        currentBestJ = j;
                    } else if (runningWidthAtTargetH > availableW * 1.5) {
                        break;
                    }
                    j++;
                }

                const rowPhotos = photos.slice(i, currentBestJ + 1);
                let rowWidthAtTargetH = 0;
                for (const p of rowPhotos) {
                    const isValid = !!(p.width && p.height && p.width > 0 && p.height > 0);
                    rowWidthAtTargetH += targetH * (isValid ? (p.width! / p.height!) : (4 / 3));
                }

                const totalGaps = (rowPhotos.length - 1) * g;
                let height = ((availableW - totalGaps) / rowWidthAtTargetH) * targetH;

                // Handle last row of the group
                const isLastRowInGroup = currentBestJ === photos.length - 1;
                if (isLastRowInGroup && height > targetH * 1.25) {
                    height = targetH;
                }

                flattenedItems.push({
                    type: 'row',
                    row: { photos: rowPhotos, height: Math.floor(height) }
                });

                i = currentBestJ + 1;
            }
        }

        return flattenedItems;
    });

    readonly rows = computed(() => {
        return this.items()
            .filter(item => item.type === 'row')
            .map(item => (item as any).row as PhotoRow);
    });

    private currentPage = 1;
    private readonly pageSize = 1000;
    private hasMore = true;
    private isRestoring = false;
    private lastSelectedIndex: number | null = null;

    ngOnInit() {
        if (!this.autoFetch()) {
            if (this._timeline().length > 0) {
                this.timelineLoaded.emit(this._timeline());
            }
            this.hasMore = false;
            return;
        }

        const cached = this.photoService.timelineCache;
        if (cached && cached.length > 0) {
            this._timeline.set([...cached]);
            this.totalPhotos.set(cached.reduce((acc, g) => acc + g.photos.total, 0));
            this.timelineLoaded.emit(cached);
            this.currentPage = Math.ceil(cached.length / this.pageSize) + 1;

            if (this.photoService.lastGalleryScrollIndex > 0) {
                this.isRestoring = true;
                requestAnimationFrame(() => {
                    this.viewport?.scrollToIndex(this.photoService.lastGalleryScrollIndex);
                    setTimeout(() => {
                        this.isRestoring = false;
                    }, 100);
                });
            }
        } else {
            this.fetchNextPage();
        }
    }

    ngAfterViewInit() {
        this.updateContainerWidth();
        if (this.container) {
            const observer = new ResizeObserver(() => {
                this.updateContainerWidth();
            });
            observer.observe(this.container.nativeElement);
        }
    }

    @HostListener('window:resize')
    onResize() {
        this.updateContainerWidth();
    }

    updateContainerWidth() {
        if (this.container) {
            this.containerWidth.set(this.container.nativeElement.clientWidth);
        }
    }

    fetchNextPage() {
        if (this.isFetching() || !this.hasMore) {
            return;
        }

        this.isFetching.set(true);
        this.photoService.getTimeline(this.currentPage, this.pageSize)
            .pipe(first())
            .subscribe(groups => {
                console.log(`Timeline ${this.currentPage} loaded with ${groups.length} groups`, groups);
                if (groups.length < this.pageSize) {
                    this.hasMore = false;
                }

                if (this.currentPage === 1) {
                    this._timeline.set(groups);
                } else {
                    this._timeline.update(current => [...current, ...groups]);
                }

                this.currentPage++;
                this.totalPhotos.set(this._timeline().reduce((acc, g) => acc + g.photos.total, 0));
                this.timelineLoaded.emit(this._timeline());
                this.isFetching.set(false);

                if (this.photoService.lastGalleryScrollIndex > 0 && this.currentPage === 2) {
                    this.isRestoring = true;
                    requestAnimationFrame(() => {
                        this.viewport?.scrollToIndex(this.photoService.lastGalleryScrollIndex);
                        setTimeout(() => {
                            this.isRestoring = false;
                        }, 100);
                    });
                }
            });
    }

    jumpToGroupOffset(offset: number, yearLabel?: string) {
        if (this.isFetching() || !this.hasMore) return;

        const currentGroupsCount = this._timeline().length;
        if (offset < currentGroupsCount) {
            this.scrollByYearLabel(yearLabel);
            return;
        }

        this.isFetching.set(true);
        this.performJumpRecursive(offset, yearLabel);
    }

    private performJumpRecursive(offset: number, yearLabel?: string) {
        this.photoService.getTimeline(this.currentPage, this.pageSize)
            .pipe(first())
            .subscribe(groups => {
                if (!groups || groups.length === 0) {
                    this.hasMore = false;
                    this.isFetching.set(false);
                    return;
                }

                if (groups.length < this.pageSize) {
                    this.hasMore = false;
                }

                this._timeline.update(current => [...current, ...groups]);
                this.currentPage++;
                this.totalPhotos.set(this._timeline().reduce((acc, g) => acc + g.photos.total, 0));
                this.timelineLoaded.emit(this._timeline());

                const currentGroupsCount = this._timeline().length;
                if (offset < currentGroupsCount) {
                    this.isFetching.set(false);
                    this.scrollByYearLabel(yearLabel);
                } else if (this.hasMore) {
                    this.performJumpRecursive(offset, yearLabel);
                } else {
                    this.isFetching.set(false);
                    this.scrollByYearLabel(yearLabel);
                }
            });
    }

    private scrollByYearLabel(yearLabel?: string) {
        const title = yearLabel ? yearLabel : '';
        const target = this._timeline().find(g => g.title.startsWith(title));
        if (target) {
            this.scrollToTitle(target.title);
        }
    }

    onScroll(index: number) {
        if (!this.isRestoring) {
            this.photoService.lastGalleryScrollIndex = index;
            this.photoService.isScrolled.set(index > 0);
        }

        const currentItems = this.items();
        if (index >= 0 && index < currentItems.length) {
            for (let i = index; i >= 0; i--) {
                const item = currentItems[i];
                if (item.type === 'header') {
                    this.activeTitleChange.emit(item.title);
                    break;
                }
            }
        }

        if (index > 0 && index > currentItems.length - 15 && !this.isFetching() && this.hasMore && this.autoFetch()) {
            this.fetchNextPage();
        }
    }

    scrollToTitle(title: string) {
        const index = this.items().findIndex(item => item.type === 'header' && item.title === title);
        if (index >= 0 && this.viewport) {
            this.viewport.scrollToIndex(index, 'instant');
        }
    }

    getImageUrl(photo: Photo): string {
        return this.photoService.getThumbnailPath(photo);
    }

    getPhotoWidth(photo: Photo, rowHeight: number): number {
        const isValid = (photo.width > 0 && photo.height > 0);
        const aspectRatio = isValid ? (photo.width / photo.height) : (4 / 3);
        return rowHeight * aspectRatio;
    }

    togglePhotoSelection(photo: Photo, event?: MouseEvent) {
        if (event) {
            event.preventDefault();
            event.stopPropagation();
        }

        const current = this.selectedPhotos();
        const flatPhotos = this.flattenedPhotos();
        const photoIndex = flatPhotos.findIndex(p => p.id === photo.id);
        let next: Photo[];

        if (event?.shiftKey && this.lastSelectedIndex !== null && photoIndex >= 0) {
            const start = Math.min(this.lastSelectedIndex, photoIndex);
            const end = Math.max(this.lastSelectedIndex, photoIndex);
            const range = flatPhotos.slice(start, end + 1);
            const selectedIds = new Set<string>(current.map(p => p.id));
            range.forEach(p => selectedIds.add(p.id));
            next = flatPhotos.filter(p => selectedIds.has(p.id));
        } else {
            const index = current.findIndex(p => p.id === photo.id);
            if (index >= 0) {
                next = [...current];
                next.splice(index, 1);
            } else {
                next = [...current, photo];
            }
        }

        if (photoIndex >= 0) {
            this.lastSelectedIndex = photoIndex;
        }

        this.selectionService.updateSelection(next);
        this.selectionChange.emit(next);
    }

    toggleGroupSelection(groupTitle: string) {
        const group = this._timeline().find(g => g.title === groupTitle);
        if (!group) return;

        const groupPhotos = group.photos.items;
        const current = new Set(this.selectedIds());
        let next = [...this.selectedPhotos()];

        const allSelected = groupPhotos.every(p => current.has(p.id));

        if (allSelected) {
            const groupIds = new Set(groupPhotos.map(p => p.id));
            next = next.filter(p => !groupIds.has(p.id));
        } else {
            groupPhotos.forEach(p => {
                if (!current.has(p.id)) {
                    next.push(p);
                }
            });
        }

        this.selectionService.updateSelection(next);
        this.selectionChange.emit(next);
        this.lastSelectedIndex = null;
    }

    clearSelection() {
        this.selectionService.clearSelection();
        this.selectionChange.emit([]);
        this.lastSelectedIndex = null;
    }

    isSelected(photoId: string): boolean {
        return this.selectedIds().has(photoId);
    }

    isGroupSelected(groupTitle: string): boolean {
        const group = this._timeline().find(g => g.title === groupTitle);
        if (!group || group.photos.items.length === 0) return false;
        return group.photos.items.every(p => this.selectedIds().has(p.id));
    }

    private flattenedPhotos(): Photo[] {
        const groups = this._timeline();
        if (groups.length === 0) return [];
        const result: Photo[] = [];
        for (const group of groups) {
            for (const photo of group.photos.items) {
                result.push(photo);
            }
        }
        return result;
    }
}

