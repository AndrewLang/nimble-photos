import { Component, signal, inject, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, Router, ActivatedRoute } from '@angular/router';
import { FormsModule, ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { SelectionService } from '../../services/selection.service';
import { PhotoService } from '../../services/photo.service';
import { AuthService } from '../../services/auth.service';
import { DialogService } from '../../services/dialog.service';
import { InfoDialog } from '../dialog/info-dialog.component';
import { AlbumEditorComponent } from '../album/album-editor.component';

@Component({
  selector: 'mtx-header',
  standalone: true,
  imports: [CommonModule, RouterModule],
  templateUrl: './header.component.html',
  styles: [`
    :host {
      display: block;
      width: 100%;
    }
  `]
})
export class HeaderComponent implements OnInit {
  private readonly fb = inject(FormBuilder);
  private readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  private readonly dialogService = inject(DialogService);

  readonly isMenuOpen = signal(false);
  readonly isUserMenuOpen = signal(false);

  constructor(
    public readonly selectionService: SelectionService,
    public readonly photoService: PhotoService,
    public readonly authService: AuthService
  ) { }

  ngOnInit(): void {
    this.route.queryParams.subscribe(params => {
      if (params['registered']) {
      }
    });
  }

  logout() {
    this.authService.logout();
    this.closeMenu();
  }

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

  async createAlbum() {
    const photos = this.selectionService.selectedPhotos();
    const ref = this.dialogService.open(AlbumEditorComponent, {
      title: 'Create New Album',
      width: '600px',
      data: { photos },
      actions: [
        { label: 'Cancel', value: false, style: 'ghost' },
        { label: 'Create Album', value: 'submit', style: 'primary' }
      ]
    });

    const result = await ref.afterClosed();
    if (result === 'submit') {
      const editor = ref.componentInstance as AlbumEditorComponent;
      const albumData = editor.getFormValue();

      this.photoService.createAlbum({
        name: albumData.name,
        description: albumData.description,
        kind: 'manual',
        rulesJson: JSON.stringify({ photoIds: albumData.photoIds }),
        sortOrder: 0
      }).subscribe({
        next: (album) => {
          this.selectionService.clearSelection();
          this.router.navigate(['/albums', album.id]);
        },
        error: (err) => {
          console.error('Failed to create album:', err);
          alert('Failed to create album. Please try again.');
        }
      });
    }
  }

  addToAlbum() {
    this.dialogService.open(InfoDialog, {
      title: 'Add to Album',
      actions: [{ label: 'Understood', value: true, style: 'secondary' }]
    });
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
    this.dialogService.open(InfoDialog, {
      title: 'Tag Photos',
      actions: [
        { label: 'Cancel', value: false, style: 'ghost' },
        { label: 'OK', value: true, style: 'primary' }
      ]
    });
  }
}
