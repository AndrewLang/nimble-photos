import { Component, OnInit, signal, effect } from '@angular/core';
import { RouterModule } from '@angular/router';
import { first } from 'rxjs';
import { PhotoService } from '../../../services/photo.service';
import { GroupedPhotos, Photo } from '../../../models/photo.model';

@Component({
  selector: 'app-grouped-gallery',
  imports: [RouterModule],
  templateUrl: './grouped-gallery.html',
  styleUrl: './grouped-gallery.css',
  standalone: true
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

    // Detect active group
    const headers = document.querySelectorAll('.group-header');
    let currentActive = '';

    // Simple intersection check: find the first header that is above a certain threshold
    // or arguably the one closest to top.
    // Iterating backwards to find the one that has "passed" the top.

    // Convert to array and reverse to find the last one that is above the "view line"
    const visibleGroups = Array.from(headers).filter(h => {
      const rect = h.getBoundingClientRect();
      // Check if header is roughly near the top (e.g. within top 1/3 of screen or passed it)
      return rect.top < 300; // 300px buffer from top
    });

    if (visibleGroups.length > 0) {
      // The last one in this list is effectively the "current" section active
      const activeEl = visibleGroups[visibleGroups.length - 1];
      // We need to store the ID in the template first
      const id = activeEl.getAttribute('id')?.replace('group-header-', '');
      if (id && id !== this.activeGroupTitle()) {
        this.activeGroupTitle.set(id);
      }
    }

    // Check if we are close to the bottom
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
          return; // End of all groups
        }

        let shouldLoadMore = false;

        this.groups.update(current => {
          // 1. Find if we already have this group
          const existingGroupIndex = current.findIndex(g => g.title === result.title);
          let updatedGroups = [...current];

          if (existingGroupIndex >= 0) {
            // Append to existing
            const existing = current[existingGroupIndex];
            const newPhotos = result.photos.items;

            // Note: Service handles returning empty if page is out of range, 
            // but we might optimize by checking length below.

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
            // New group
            if (result.photos.items.length > 0) {
              updatedGroups = [...current, result];
            }
          }

          // 2. Logic to advance state
          // If we received fewer items than requested, we are at the end of this group.
          if (result.photos.items.length < this.pageSize) {
            this.currentGroupIndex++;
            this.currentPageInGroup = 1;
          } else {
            this.currentPageInGroup++;
          }

          // 3. Check if we have enough content to fill screen
          // Count total photos
          const totalPhotos = updatedGroups.reduce((acc, g) => acc + g.photos.items.length, 0);
          if (totalPhotos < 60 && result.photos.items.length > 0) {
            shouldLoadMore = true;
          } else if (result.photos.items.length === 0) {
            // If we got an empty page, definitely try next group/page immediately 
            // (unless we are truly at end of data, which 'result' check at top handles roughly, 
            // but service returns empty object for OOB page, null for OOB group)
            shouldLoadMore = true;
          }

          return updatedGroups;
        });

        this.loading.set(false);

        if (shouldLoadMore) {
          // Queue next load
          setTimeout(() => this.loadNextBatch(), 0);
        }
      });
  }
}
