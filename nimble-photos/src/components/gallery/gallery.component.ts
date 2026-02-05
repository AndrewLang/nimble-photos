import { Component, OnInit, signal, computed, Input } from '@angular/core';

import { Router, RouterModule } from '@angular/router';
import { first } from 'rxjs';

import { Photo, PagedPhotos } from '../../models/photo';
import { PhotoService } from '../../services/photo.service';
import { SelectionService } from '../../services/selection.service';

import { ImageFallbackDirective } from '../../directives/image.fallback.directive';
import { SvgComponent } from '../svg/svg.component';

@Component({
  selector: 'mtx-gallery',
  imports: [RouterModule, ImageFallbackDirective, SvgComponent],
  templateUrl: './gallery.component.html',
  host: {
    class: 'flex flex-col flex-1 min-h-0',
  },
})
export class GalleryComponent implements OnInit {
  readonly photos = signal<Photo[]>([]);
  readonly totalPhotos = signal(0);
  readonly isFetching = signal(false);
  readonly selectedIds = computed(() => this.selectionService.selectedIds());
  readonly selectedPhotos = computed(() => this.selectionService.selectedPhotos());
  readonly isSelectionMode = computed(() => this.selectionEnabled || this.selectionService.hasSelection());

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
  @Input() selectionEnabled = false;

  private currentPage = 1;
  private readonly pageSize = 56;
  private hasMore = true;

  constructor(
    private readonly photoService: PhotoService,
    private readonly selectionService: SelectionService,
    public readonly router: Router
  ) { }

  ngOnInit(): void {
    if (this.autoFetch && this.photos().length === 0) {
      this.fetchNextPage();
    }
  }

  fetchNextPage(): void {
    if (this.isFetching() || !this.hasMore) {
      return;
    }

    this.isFetching.set(true);
    const fetch$ = this.albumId
      ? this.photoService.getAlbumPhotos(this.albumId, this.currentPage, this.pageSize)
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

