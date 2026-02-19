import { Component, OnInit, computed, effect, inject, input, signal } from '@angular/core';

import { Router, RouterModule } from '@angular/router';
import { first } from 'rxjs';

import { Photo } from '../../models/photo';
import { PhotoService } from '../../services/photo.service';
import { SelectionService } from '../../services/selection.service';

import { ImageFallbackDirective } from '../../directives/image.fallback.directive';
import { SvgComponent } from '../svg/svg.component';

@Component({
  selector: 'mtx-gallery',
  imports: [RouterModule, ImageFallbackDirective, SvgComponent],
  templateUrl: './gallery.component.html'
})
export class GalleryComponent implements OnInit {
  readonly photos = signal<Photo[]>([]);
  readonly totalPhotos = signal(0);
  readonly isFetching = signal(false);
  private readonly photoService = inject(PhotoService);
  private readonly selectionService = inject(SelectionService);
  readonly router = inject(Router);
  readonly selectedIds = computed(() => this.selectionService.selectedIds());
  readonly selectedPhotos = computed(() => this.selectionService.selectedPhotos());
  readonly isSelectionMode = computed(() => this.selectionEnabled() || this.selectionService.hasSelection());

  readonly initialPhotos = input<Photo[]>([]);
  private readonly syncInitialPhotosEffect = effect(() => {
    const value = this.initialPhotos() ?? [];
    this.photos.set(value);
    if (!this.autoFetch()) {
      this.totalPhotos.set(value.length);
    }
  });

  readonly initialTotal = input(0);
  private readonly syncInitialTotalEffect = effect(() => {
    const value = this.initialTotal();
    this.totalPhotos.set(value ?? 0);
  });

  readonly autoFetch = input(true);
  readonly albumId = input<string | null>(null);
  readonly showHeader = input(true);
  readonly paddingTop = input('56px');
  readonly selectionEnabled = input(false);

  private currentPage = 1;
  private readonly pageSize = 56;
  private hasMore = true;
  private lastSelectedIndex: number | null = null;

  ngOnInit(): void {
    if (this.autoFetch() && this.photos().length === 0) {
      this.fetchNextPage();
    }
  }

  fetchNextPage(): void {
    if (this.isFetching() || !this.hasMore) {
      return;
    }

    this.isFetching.set(true);
    const albumId = this.albumId();
    const fetch$ = albumId
      ? this.photoService.getAlbumPhotos(albumId, this.currentPage, this.pageSize)
      : this.photoService.getPhotos(this.currentPage, this.pageSize);

    fetch$
      .pipe(first())
      .subscribe(pagedPhotos => {
        if (!pagedPhotos) {
          this.hasMore = false;
          this.isFetching.set(false);
          return;
        }

        if (pagedPhotos.items.length < this.pageSize) {
          this.hasMore = false;
        }

        if (this.currentPage === 1) {
          this.photos.set(pagedPhotos.items);
        } else {
          this.photos.update(current => [...current, ...pagedPhotos.items]);
        }

        this.currentPage++;
        this.totalPhotos.set(pagedPhotos.total);
        this.isFetching.set(false);
      });
  }

  onScroll(event: any): void {
    const element = event.target;
    const scrollPosition = element.scrollTop + element.clientHeight;
    const threshold = element.scrollHeight - 1000;

    this.photoService.isScrolled.set(element.scrollTop > 20);

    if (scrollPosition >= threshold && !this.isFetching() && this.hasMore) {
      this.fetchNextPage();
    }
  }

  getImageUrl(photo: Photo): string {
    return this.photoService.getThumbnailPath(photo);
  }

  togglePhotoSelection(photo: Photo, event?: MouseEvent): void {
    if (event) {
      event.preventDefault();
      event.stopPropagation();
    }

    const current = this.selectedPhotos();
    const photoIndex = this.photos().findIndex(p => p.id === photo.id);
    let next: Photo[];

    if (event?.shiftKey && this.lastSelectedIndex !== null && photoIndex >= 0) {
      const start = Math.min(this.lastSelectedIndex, photoIndex);
      const end = Math.max(this.lastSelectedIndex, photoIndex);
      const range = this.photos().slice(start, end + 1);
      const selectedIds = new Set<string>(current.map(p => p.id));
      range.forEach(p => selectedIds.add(p.id));
      next = this.photos().filter(p => selectedIds.has(p.id));
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
  }

  isSelected(photoId: string): boolean {
    return this.selectedIds().has(photoId);
  }

  onClearSelection(): void {
    this.selectionService.clearSelection();
    this.lastSelectedIndex = null;
  }
}

