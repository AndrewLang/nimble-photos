import { Component, OnInit, signal, computed, Input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';
import { first } from 'rxjs';

import { Photo } from '../../models/photo.model';
import { PhotoService } from '../../services/photo.service';
import { SelectionService } from '../../services/selection.service';

@Component({
  selector: 'mtx-gallery',
  imports: [CommonModule, RouterModule],
  templateUrl: './gallery.component.html',
  host: {
    class: 'block flex-1 min-h-0',
  },
})
export class GalleryComponent implements OnInit {
  readonly photos = signal<Photo[]>([]);
  readonly totalPhotos = signal(0);
  readonly isFetching = signal(false);
  readonly selectedIds = computed(() => this.selectionService.selectedIds());
  readonly selectedPhotos = computed(() => this.selectionService.selectedPhotos());
  readonly isSelectionMode = computed(() => this.selectionService.hasSelection());

  @Input() set initialPhotos(value: Photo[]) {
    this.photos.set(value);
    if (!this.autoFetch) {
      this.totalPhotos.set(value.length);
    }
  }

  @Input() set initialTotal(value: number) {
    this.totalPhotos.set(value);
  }

  @Input() autoFetch = true;
  @Input() albumId?: string | null = null;
  @Input() showHeader = true;
  @Input() paddingTop = '56px';

  private currentPage = 1;
  private readonly pageSize = 50;
  private hasMore = true;

  constructor(
    private readonly photoService: PhotoService,
    private readonly selectionService: SelectionService
  ) { }

  ngOnInit(): void {
    if (this.autoFetch) {
      this.fetchNextPage();
    }
  }

  fetchNextPage(): void {
    if (this.isFetching() || !this.hasMore) {
      return;
    }

    this.isFetching.set(true);
    this.photoService.getPhotos(this.currentPage, this.pageSize)
      .pipe(first())
      .subscribe(pagedPhotos => {
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
    const index = current.findIndex(p => p.id === photo.id);
    let next: Photo[];

    if (index >= 0) {
      next = [...current];
      next.splice(index, 1);
    } else {
      next = [...current, photo];
    }

    this.selectionService.updateSelection(next);
  }

  isSelected(photoId: string): boolean {
    return this.selectedIds().has(photoId);
  }

  onClearSelection(): void {
    this.selectionService.clearSelection();
  }
}
