import { Component, ElementRef, HostListener, OnInit, ViewChild, signal, computed, AfterViewInit } from '@angular/core';
import { CommonModule, DatePipe } from '@angular/common';
import { RouterModule } from '@angular/router';
import { ScrollingModule } from '@angular/cdk/scrolling';
import { first } from 'rxjs';

import { GroupedPhotos, Photo } from '../../models/photo.model';
import { PhotoService } from '../../services/photo.service';

interface PhotoRow {
    photos: Photo[];
    height: number;
}

type GalleryItem =
    | { type: 'header'; title: string; count: number }
    | { type: 'row'; row: PhotoRow };

@Component({
    selector: 'mtx-justified-gallery',
    standalone: true,
    imports: [CommonModule, RouterModule, ScrollingModule, DatePipe],
    templateUrl: './justified-gallery.component.html',
    host: {
        class: 'block h-full w-full overflow-hidden'
    }
})
export class JustifiedGalleryComponent implements OnInit, AfterViewInit {
    @ViewChild('container') container?: ElementRef<HTMLElement>;

    readonly timeline = signal<GroupedPhotos[]>([]);
    readonly containerWidth = signal<number>(0);
    readonly targetHeight = signal<number>(240);
    readonly gap = signal<number>(6);
    readonly isFetching = signal(false);
    readonly totalPhotos = signal(0);
    readonly loadedPhotosCount = computed(() =>
        this.timeline().reduce((acc, g) => acc + g.photos.items.length, 0)
    );

    readonly items = computed(() => {
        const groups = this.timeline();
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
        // Redundant with items but kept for compatibility if needed elsewhere
        return this.items()
            .filter(item => item.type === 'row')
            .map(item => (item as any).row as PhotoRow);
    });

    private currentPage = 0;
    private readonly pageSize = 100;

    constructor(private readonly photoService: PhotoService) { }

    ngOnInit() {
        this.fetchNextPage();
    }

    ngAfterViewInit() {
        this.updateContainerWidth();
        // Use ResizeObserver for more robust width tracking
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
        if (this.isFetching()) {
            return;
        }

        this.isFetching.set(true);
        this.photoService.getTimeline()
            .pipe(first())
            .subscribe(groups => {
                console.log('Timeline groups    ', groups);
                this.timeline.set(groups);
                this.totalPhotos.set(groups.reduce((acc, g) => acc + g.photos.total, 0));
                this.isFetching.set(false);
            });
    }

    onScroll(index: number) {
        // Timeline as implemented in service might already be complete or support paging
        // For now we just load once as getTimeline() returns all (limited by backend)
    }

    getImageUrl(photo: Photo): string {
        return this.photoService.getThumbnailPath(photo);
    }

    getPhotoWidth(photo: Photo, rowHeight: number): number {
        const isValid = !!(photo.width && photo.height && photo.width > 0 && photo.height > 0);
        const aspectRatio = isValid ? (photo.width! / photo.height!) : (4 / 3);
        return rowHeight * aspectRatio;
    }
}
