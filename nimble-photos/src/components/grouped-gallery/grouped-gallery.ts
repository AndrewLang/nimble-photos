import { Component, OnInit, signal, ViewChild } from '@angular/core';
import { CommonModule, DatePipe } from '@angular/common';
import { RouterModule } from '@angular/router';
import { GroupedPhotos, Photo } from '../../models/photo.model';
import { PhotoService } from '../../services/photo.service';
import { JustifiedGalleryComponent } from '../justified-gallery/justified-gallery.component';

@Component({
  selector: 'mtx-grouped-gallery',
  imports: [CommonModule, RouterModule, DatePipe, JustifiedGalleryComponent],
  templateUrl: './grouped-gallery.html',
  host: {
    class: 'block flex-1 min-h-0',
  },
})
export class GroupedGallery implements OnInit {
  @ViewChild(JustifiedGalleryComponent) gallery?: JustifiedGalleryComponent;

  readonly groups = signal<GroupedPhotos[]>([]);
  readonly years = signal<string[]>([]);
  readonly activeGroupTitle = signal('');
  readonly activeYear = signal('');
  readonly selectedPhotos = signal<Photo[]>([]);

  constructor(private readonly photoService: PhotoService) { }

  ngOnInit(): void {
    this.photoService.getTimelineYears().subscribe(years => {
      this.years.set(years);
      if (years.length > 0 && !this.activeYear()) {
        this.activeYear.set(years[0]);
      }
    });
  }

  getMonthName(monthStr: string): string {
    try {
      if (!monthStr)
        return '';
      const date = new Date(2000, parseInt(monthStr) - 1, 1);
      return date.toLocaleString('default', { month: 'short' });
    } catch (error) {
      console.error('Error getting month name:', error);
      return '';
    }
  }

  scrollToGroup(title: string): void {
    if (this.gallery) {
      this.gallery?.scrollToTitle(title);
      this.activeGroupTitle.set(title);
      const year = title.split('-')[0];
      if (year) this.activeYear.set(year);
    }
  }

  scrollToYear(year: string): void {
    const targetGroup = this.groups().find(g => g.title.startsWith(year));
    if (targetGroup) {
      this.scrollToGroup(targetGroup.title);
    } else {
      // Fetch the offset and jump
      this.photoService.getTimelineYearOffset(year).subscribe(offset => {
        if (this.gallery) {
          // Tell the gallery to load specifically the page containing this offset
          this.gallery.jumpToGroupOffset(offset, year);
        }
      });
    }
    this.activeYear.set(year);
  }

  onTimelineLoaded(groups: GroupedPhotos[]): void {
    this.groups.set(groups);
    if (groups.length > 0 && !this.activeGroupTitle()) {
      this.activeGroupTitle.set(groups[0].title);
    }
  }

  onActiveTitleChange(title: string): void {
    this.activeGroupTitle.set(title);
    const year = title.split('-')[0];
    if (year) this.activeYear.set(year);
  }

  onSelectionChange(photos: Photo[]): void {
    this.selectedPhotos.set(photos);
  }

  clearSelection(): void {
    this.gallery?.clearSelection();
  }
}
