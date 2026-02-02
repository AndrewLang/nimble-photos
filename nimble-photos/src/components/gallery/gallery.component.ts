import { Component, OnInit, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';
import { first } from 'rxjs';

import { Photo } from '../../models/photo.model';
import { PhotoService } from '../../services/photo.service';

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

  private currentPage = 0;
  private readonly pageSize = 56;
  private readonly preloadThreshold = 28;

  constructor(private readonly photoService: PhotoService) { }

  ngOnInit(): void {
    this.fetchNextPage();
  }

  onScroll(event: Event): void {
    const element = event.target as HTMLElement;

    if (element.scrollHeight - element.scrollTop <= element.clientHeight + 1000) {
      this.fetchNextPage();
    }
  }

  private fetchNextPage(): void {
    if (this.isFetching()) {
      return;
    }

    if (this.totalPhotos() > 0 && this.photos().length >= this.totalPhotos()) {
      return;
    }

    const nextPage = this.currentPage + 1;
    this.isFetching.set(true);

    this.photoService
      .getPhotos(nextPage, this.pageSize)
      .pipe(first())
      .subscribe((page) => {
        this.photos.update((previous) => [...previous, ...page.items]);
        this.totalPhotos.set(page.total);
        this.currentPage = page.page;
        this.isFetching.set(false);
      });
  }
}
