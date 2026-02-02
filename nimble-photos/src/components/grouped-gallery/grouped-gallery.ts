import { Component, OnInit, signal, ViewChild } from '@angular/core';
import { CommonModule, DatePipe } from '@angular/common';
import { RouterModule } from '@angular/router';
import { GroupedPhotos } from '../../models/photo.model';
import { PhotoService } from '../../services/photo.service';
import { JustifiedGalleryComponent } from '../justified-gallery/justified-gallery.component';

@Component({
  selector: 'mtx-grouped-gallery',
  imports: [CommonModule, RouterModule, DatePipe, JustifiedGalleryComponent],
  templateUrl: './grouped-gallery.html',
  styleUrls: [],
  host: {
    class: 'block flex-1 min-h-0',
  },
})
export class GroupedGallery implements OnInit {
  @ViewChild(JustifiedGalleryComponent) gallery?: JustifiedGalleryComponent;

  readonly groups = signal<GroupedPhotos[]>([]);
  readonly activeGroupTitle = signal('');

  constructor(private readonly photoService: PhotoService) { }

  ngOnInit(): void {

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
      this.gallery.scrollToTitle(title);
      this.activeGroupTitle.set(title);
    }
  }

  onTimelineLoaded(groups: GroupedPhotos[]): void {
    this.groups.set(groups);
    if (groups.length > 0 && !this.activeGroupTitle()) {
      this.activeGroupTitle.set(groups[0].title);
    }
  }

  onActiveTitleChange(title: string): void {
    this.activeGroupTitle.set(title);
  }
}
