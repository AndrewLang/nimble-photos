
import { Component, effect, ElementRef, inject, OnInit, signal, ViewChild } from '@angular/core';
import { RouterModule } from '@angular/router';
import { GroupedPhotos, Photo } from '../../models/photo';
import { PhotoService } from '../../services/photo.service';
import { JustifiedGalleryComponent } from '../justified.gallery/justified.gallery.component';

@Component({
  selector: 'mtx-grouped.gallery',
  imports: [RouterModule, JustifiedGalleryComponent],
  templateUrl: './grouped.gallery.html',
  host: {
    class: 'block flex-1 min-h-0',
  },
})
export class GroupedGalleryComponent implements OnInit {
  @ViewChild(JustifiedGalleryComponent) gallery?: JustifiedGalleryComponent;
  @ViewChild('monthsRuler') monthsRuler?: ElementRef<HTMLElement>;

  readonly groups = signal<GroupedPhotos[]>([]);
  readonly years = signal<string[]>([]);
  readonly activeGroupTitle = signal('');
  readonly activeYear = signal('');
  readonly selectedPhotos = signal<Photo[]>([]);

  private readonly photoService = inject(PhotoService);
  private readonly scrollEffect = effect(() => {
    const year = this.activeYear();
    if (year && this.monthsRuler) {
      this.scrollMonthRulerToYear(year);
    }
  });

  ngOnInit(): void {
    this.photoService.getTimelineYears().subscribe(years => {
      this.years.set(years);
      if (years.length > 0 && !this.activeYear()) {
        this.activeYear.set(years[0]);
      }
    });
  }

  getYear(title: string): string {
    return title ? title.split('-')[0] : '';
  }

  isNewYear(index: number): boolean {
    const groups = this.groups();
    if (!groups || groups.length === 0 || index === 0) return true;
    const currentYear = this.getYear(groups[index].title);
    const prevYear = this.getYear(groups[index - 1].title);
    return currentYear !== prevYear;
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
    console.log(`Scrolling to year: ${year}`);
    const targetGroup = this.groups().find(g => g.title.startsWith(year));

    console.log('Target group:', targetGroup);
    if (targetGroup) {
      this.scrollToGroup(targetGroup.title);
    } else {
      this.photoService.getTimelineYearOffset(year)
        .subscribe(offset => {
          console.log(`Year ${year} offset:`, offset, this.gallery);
          if (this.gallery) {
            this.gallery.jumpToGroupOffset(offset, year);
          }
        });
    }
    this.activeYear.set(year);

    if (this.monthsRuler) {
      this.scrollMonthRulerToYear(year);
    }
  }

  private scrollMonthRulerToYear(year: string): void {
    if (!this.monthsRuler) return;
    const container = this.monthsRuler.nativeElement;
    const targetItem = container.querySelector(`[data-year="${year}"]`) as HTMLElement;

    if (targetItem) {
      container.scrollTo({
        top: targetItem.offsetTop - container.clientHeight / 2 + targetItem.clientHeight / 2,
        behavior: 'smooth'
      });
    }
  }

  onTimelineLoaded(groups: GroupedPhotos[]): void {
    queueMicrotask(() => {
      this.groups.set(groups);
      if (groups.length > 0 && !this.activeGroupTitle()) {
        this.activeGroupTitle.set(groups[0].title);
      }
    });
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

