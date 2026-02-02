import { Component, OnInit, signal } from '@angular/core';
import { CommonModule, DatePipe } from '@angular/common';
import { RouterModule } from '@angular/router';
import { first } from 'rxjs';
import { GroupedPhotos, Photo } from '../../models/photo.model';
import { PhotoService } from '../../services/photo.service';

@Component({
  selector: 'mtx-grouped-gallery',
  imports: [CommonModule, RouterModule, DatePipe],
  templateUrl: './grouped-gallery.html',
  styleUrls: [],
  standalone: true,
  host: {
    class: 'block flex-1 min-h-0',
  },
})
export class GroupedGallery implements OnInit {
  readonly groups = signal<GroupedPhotos[]>([]);
  readonly loading = signal(false);
  readonly activeGroupTitle = signal('');

  private currentGroupIndex = 0;
  private currentPageInGroup = 1;
  private readonly pageSize = 100;

  constructor(private readonly photoService: PhotoService) { }

  ngOnInit(): void {
    this.loadNextBatch();
  }

  getMonthName(monthStr: string): string {
    const date = new Date(2000, parseInt(monthStr) - 1, 1);
    return date.toLocaleString('default', { month: 'short' });
  }

  scrollToGroup(title: string): void {
    const element = document.getElementById(`group-${title}`);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' });
      this.activeGroupTitle.set(title);
    }
  }

  onScroll(event: Event): void {
    const element = event.target as HTMLElement;

    const headers = document.querySelectorAll('.group-header');
    let currentActive = '';

    const visibleGroups = Array.from(headers).filter(h => {
      const rect = h.getBoundingClientRect();
      return rect.top < 300;
    });

    if (visibleGroups.length > 0) {
      const activeEl = visibleGroups[visibleGroups.length - 1];
      const id = activeEl.getAttribute('id')?.replace('group-header-', '');
      if (id && id !== this.activeGroupTitle()) {
        this.activeGroupTitle.set(id);
      }
    }

    if (element.scrollHeight - element.scrollTop <= element.clientHeight + 1000) {
      this.loadNextBatch();
    }
  }

  private loadNextBatch(): void {
    if (this.loading()) return;
    this.loading.set(true);

    this.photoService.getGroupedPhotos(this.currentGroupIndex, this.currentPageInGroup, this.pageSize)
      .pipe(first())
      .subscribe(result => {
        if (!result) {
          this.loading.set(false);
          return;
        }

        let shouldLoadMore = false;

        this.groups.update(current => {
          const existingGroupIndex = current.findIndex(g => g.title === result.title);
          let updatedGroups = [...current];

          if (existingGroupIndex >= 0) {
            const existing = current[existingGroupIndex];
            const newPhotos = result.photos.items;

            if (newPhotos.length > 0) {
              updatedGroups[existingGroupIndex] = {
                ...existing,
                photos: {
                  ...existing.photos,
                  items: [...existing.photos.items, ...newPhotos]
                }
              };
            }
          } else {
            if (result.photos.items.length > 0) {
              updatedGroups = [...current, result];
            }
          }

          if (result.photos.items.length < this.pageSize) {
            this.currentGroupIndex++;
            this.currentPageInGroup = 1;
          } else {
            this.currentPageInGroup++;
          }

          const totalPhotos = updatedGroups.reduce((acc, g) => acc + g.photos.items.length, 0);
          if (totalPhotos < 60 && result.photos.items.length > 0) {
            shouldLoadMore = true;
          } else if (result.photos.items.length === 0) {
            shouldLoadMore = true;
          }

          return updatedGroups;
        });

        this.loading.set(false);

        if (shouldLoadMore) {
          setTimeout(() => this.loadNextBatch(), 0);
        }
      });
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
}
