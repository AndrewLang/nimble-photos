import { Component, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';
import { SelectionService } from '../../services/selection.service';
import { PhotoService } from '../../services/photo.service';

@Component({
  selector: 'mtx-header',
  imports: [CommonModule, RouterModule],
  templateUrl: './header.component.html',
  styles: [`
    :host {
      display: block;
      width: 100%;
    }
  `]
})
export class HeaderComponent {
  readonly isMenuOpen = signal(false);
  readonly isUserMenuOpen = signal(false);

  constructor(
    public readonly selectionService: SelectionService,
    public readonly photoService: PhotoService
  ) { }

  toggleMenu() {
    this.isMenuOpen.update(v => !v);
    if (this.isMenuOpen()) this.isUserMenuOpen.set(false);
  }

  toggleUserMenu() {
    this.isUserMenuOpen.update(v => !v);
  }

  closeMenu() {
    this.isMenuOpen.set(false);
    this.isUserMenuOpen.set(false);
  }

  closeUserMenu() {
    this.isUserMenuOpen.set(false);
  }

  clearSelection() {
    this.selectionService.clearSelection();
  }

  createAlbum() {
    const photos = this.selectionService.selectedPhotos();
    alert(`Coming soon: Create new album with ${photos.length} photos`);
  }

  addToAlbum() {
    const photos = this.selectionService.selectedPhotos();
    alert(`Coming soon: Add ${photos.length} photos to existing album`);
  }

  downloadSelected() {
    const photos = this.selectionService.selectedPhotos();
    photos.forEach(p => {
      const link = document.createElement('a');
      link.href = this.photoService.getDownloadPath(p);
      link.download = p.name;
      link.click();
    });
  }

  tagPhotos() {
    const photos = this.selectionService.selectedPhotos();
    alert(`Coming soon: Tag ${photos.length} photos`);
  }
}
