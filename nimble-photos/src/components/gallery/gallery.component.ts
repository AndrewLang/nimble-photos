import { CommonModule } from '@angular/common';
import { Component, OnInit, signal } from '@angular/core';
import { first } from 'rxjs';

import { PhotoService } from '../../services/photo.service';
import { Photo } from '../../models/photo.model';

@Component({
  standalone: true,
  selector: 'app-gallery',
  imports: [CommonModule],
  templateUrl: './gallery.component.html',
  styleUrls: ['./gallery.component.css'],
  host: {
    class: 'block',
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

  trackByPhoto(_: number, photo: Photo): string {
    return photo.id;
  }

  onScroll(event: Event): void {
    const element = event.target as HTMLElement;
    // console.log('Scroll:', element.scrollTop, element.clientHeight, element.scrollHeight);

    // Check if we are close to the bottom (1000px threshold)
    if (element.scrollHeight - element.scrollTop <= element.clientHeight + 1000) {
      this.fetchNextPage();
    }
  }

  private fetchNextPage(): void {
    if (this.isFetching()) {
      // console.log('Already fetching');
      return;
    }

    if (this.totalPhotos() > 0 && this.photos().length >= this.totalPhotos()) {
      // console.log('All photos loaded');
      return;
    }

    // console.log('Fetching next page...');
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
