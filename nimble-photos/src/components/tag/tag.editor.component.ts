import { Component, OnInit, input, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Photo } from '../../models/photo';
import { SvgComponent } from '../svg/svg.component';

@Component({
  selector: 'mtx-tag-editor',
  imports: [FormsModule, SvgComponent],
  templateUrl: './tag.editor.component.html'
})
export class TagEditorComponent implements OnInit {
  readonly photos = input<Photo[]>([]);
  readonly existingTags = input<string[]>([]);
  readonly selectedTags = signal<string[]>([]);
  readonly draftTag = signal('');

  ngOnInit(): void {
    const merged = new Set<string>();
    for (const photo of this.photos()) {
      for (const tag of photo.tags ?? []) {
        const normalized = tag.trim();
        if (normalized) {
          merged.add(normalized);
        }
      }
    }
    this.selectedTags.set(Array.from(merged).sort((a, b) => a.localeCompare(b)));
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
      photoIds: this.photos().map(photo => photo.id),
      tags: this.selectedTags(),
    };
  }

  isValid() {
    return this.photos().length > 0;
  }
}
