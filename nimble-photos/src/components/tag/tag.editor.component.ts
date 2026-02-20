import { Component, OnInit, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { catchError, first, of } from 'rxjs';
import { Photo } from '../../models/photo';
import { PhotoService } from '../../services/photo.service';
import { SelectionService } from '../../services/selection.service';
import { SvgComponent } from '../svg/svg.component';

@Component({
  selector: 'mtx-tag-editor',
  imports: [FormsModule, SvgComponent],
  templateUrl: './tag.editor.component.html'
})
export class TagEditorComponent implements OnInit {
  private readonly selectionService = inject(SelectionService);
  private readonly photoService = inject(PhotoService);

  readonly selectedPhotos = signal<Photo[]>([]);
  readonly availableTags = signal<string[]>([]);
  readonly selectedTags = signal<string[]>([]);
  readonly draftTag = signal('');

  ngOnInit(): void {
    this.selectedPhotos.set(this.selectionService.selectedPhotos());

    const merged = new Set<string>();
    for (const photo of this.selectedPhotos()) {
      for (const tag of photo.tags ?? []) {
        const normalized = tag.trim();
        if (normalized) {
          merged.add(normalized);
        }
      }
    }
    this.selectedTags.set(Array.from(merged).sort((a, b) => a.localeCompare(b)));

    this.photoService.getAllPhotoTags()
      .pipe(
        first(),
        catchError(() => of([]))
      )
      .subscribe(tags => {
        this.availableTags.set(tags);
      });
  }

  addDraftTag(): void {
    this.addTag(this.draftTag());
    this.draftTag.set('');
  }

  addTag(raw: string): void {
    const normalized = raw.trim();
    if (!normalized) {
      return;
    }
    if (this.selectedTags().includes(normalized)) {
      return;
    }
    this.selectedTags.update(tags => [...tags, normalized].sort((a, b) => a.localeCompare(b)));
  }

  removeTag(tag: string): void {
    this.selectedTags.update(tags => tags.filter(item => item !== tag));
  }

  toggleSuggestedTag(tag: string): void {
    if (this.selectedTags().includes(tag)) {
      this.removeTag(tag);
      return;
    }
    this.addTag(tag);
  }

  onTagInputKeydown(event: KeyboardEvent): void {
    if (event.key !== 'Enter' && event.key !== ',') {
      return;
    }
    event.preventDefault();
    this.addDraftTag();
  }

  getFormValue() {
    return {
      photoIds: this.selectedPhotos().map(photo => photo.id),
      tags: this.selectedTags(),
    };
  }

  isValid() {
    return this.selectedPhotos().length > 0;
  }
}
