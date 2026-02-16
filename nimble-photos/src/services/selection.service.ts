import { Injectable, signal, computed } from '@angular/core';
import { Photo } from '../models/photo';

@Injectable({
    providedIn: 'root',
})
export class SelectionService {
    readonly selectedPhotos = signal<Photo[]>([]);
    readonly selectedIds = computed(() => new Set(this.selectedPhotos().map(p => p.id)));
    readonly selectedCount = computed(() => this.selectedPhotos().length);
    readonly hasSelection = computed(() => this.selectedCount() > 0);

    updateSelection(photos: Photo[]) {
        this.selectedPhotos.set(photos);
    }

    clearSelection() {
        this.selectedPhotos.set([]);
    }

    isSelected(photoId: string): boolean {
        return this.selectedIds().has(photoId);
    }
}
