import { DOCUMENT } from '@angular/common';
import { Component, computed, ElementRef, HostListener, inject, OnInit, signal, ViewChild } from '@angular/core';

import { FormBuilder } from '@angular/forms';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
import { catchError, first, of } from 'rxjs';
import { AuthService } from '../../services/auth.service';
import { DialogService } from '../../services/dialog.service';
import { PhotoService } from '../../services/photo.service';
import { SelectionService } from '../../services/selection.service';
import { SettingsService } from '../../services/settings.service';
import { AlbumEditorComponent } from '../album/album.editor.component';
import { AlbumSelectorComponent } from '../album/album.selector.component';
import { SvgComponent } from '../svg/svg.component';
import { TagEditorComponent } from '../tag/tag.editor.component';

@Component({
  selector: 'mtx-header',
  imports: [RouterModule, SvgComponent],
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
  private readonly settingsService = inject(SettingsService);
  private readonly document = inject(DOCUMENT);
  readonly selectionService = inject(SelectionService);
  readonly photoService = inject(PhotoService);
  readonly authService = inject(AuthService);

  readonly isMenuOpen = signal(false);
  readonly isUserMenuOpen = signal(false);
  readonly siteTitle = signal('Nimble Photos');
  readonly siteTagline = signal('My Photo Stories');
  readonly siteLogo = signal<string | null>(null);
  readonly allowRegistration = signal(true);
  readonly photoCommands = signal([
    {
      id: 'createAlbum',
      name: 'Create Album',
      description: 'Create a new album with the selected photos',
      icon: 'plus',
      action: () => this.createAlbum()
    },
    {
      id: 'addToAlbum',
      name: 'Add to Album',
      description: 'Add the selected photos to an existing album',
      icon: 'folderPlus',
      action: () => this.addToAlbum()
    },
    {
      id: 'tagPhotos',
      name: 'Tag Photos',
      description: 'Add tags to the selected photos',
      icon: 'tag',
      action: () => this.tagPhotos()
    },
    {
      id: 'downloadPhotos',
      name: 'Download Photos',
      description: 'Download the selected photos',
      icon: 'download',
      isHidden: true,
      action: () => { }
    }
  ]);
  readonly selectionCommands = computed(() => this.photoCommands().filter(cmd => !cmd.isHidden));
  readonly hasSelection = computed(() => this.selectionService.hasSelection());

  @ViewChild('userMenuRoot') userMenuRoot?: ElementRef<HTMLElement>;

  ngOnInit(): void {
    this.setBrowserTitle(this.siteTitle());
    this.loadSiteSettings();
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

  @HostListener('document:click', ['$event'])
  handleDocumentClick(event: MouseEvent) {
    if (!this.isUserMenuOpen()) {
      return;
    }

    const target = event.target as Node | null;
    if (!target) {
      return;
    }

    if (this.userMenuRoot?.nativeElement.contains(target)) {
      return;
    }

    this.isUserMenuOpen.set(false);
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
    if (result && result !== 'submit' && result !== false) {
      const albumData = result;

      this.photoService.createAlbum({
        name: albumData.name,
        description: albumData.description,
        kind: 'manual',
        rulesJson: JSON.stringify({ photoIds: albumData.photoIds }),
        sortOrder: 0
      }).subscribe({
        next: (album) => {
          this.selectionService.clearSelection();
          this.router.navigate(['/album', album.id]);
        },
        error: (err) => {
          console.error('Failed to create album:', err);
          alert('Failed to create album. Please try again.');
        }
      });
    }
  }

  async addToAlbum() {
    const photos = this.selectionService.selectedPhotos();
    if (photos.length === 0) return;

    const ref = this.dialogService.open(AlbumSelectorComponent, {
      title: 'Add to Album',
      width: '500px',
      actions: [
        { label: 'Cancel', value: false, style: 'ghost' },
        { label: 'Add to Album', value: 'submit', style: 'primary' }
      ]
    });

    const result = await ref.afterClosed();
    if (result && result !== 'submit' && result !== false) {
      const targetAlbum = result;

      // Fetch full album to get current rules/photoIds
      this.photoService.getAlbumById(targetAlbum.id!).subscribe(fullAlbum => {
        if (!fullAlbum) {
          alert('Album not found.');
          return;
        }

        let currentIds: string[] = [];
        if (fullAlbum.rulesJson) {
          try {
            const rules = JSON.parse(fullAlbum.rulesJson);
            currentIds = rules.photoIds || [];
          } catch (e) {
            console.error('Error parsing album rules', e);
          }
        }

        // Merge IDs (Set to avoid duplicates)
        const currentIdsSet = new Set(currentIds.map(id => id.toLowerCase()));
        const newIds = photos.map(p => p.id.toLowerCase());
        const idsToAdd = newIds.filter(id => !currentIdsSet.has(id));

        if (idsToAdd.length === 0) {
          alert('Selected photos are already in this album.');
          this.selectionService.clearSelection();
          return;
        }

        const mergedIds = [...currentIds, ...idsToAdd];

        // Update album
        this.photoService.updateAlbum({
          id: fullAlbum.id,
          name: fullAlbum.name,
          description: fullAlbum.description,
          kind: fullAlbum.kind,
          sortOrder: fullAlbum.sortOrder,
          rulesJson: JSON.stringify({ photoIds: mergedIds })
        }).subscribe({
          next: () => {
            this.selectionService.clearSelection();
            this.router.navigate(['/album', fullAlbum.id]);
          },
          error: (err) => {
            console.error('Failed to update album', err);
            alert('Failed to add photos to album.');
          }
        });
      });
    }
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
    if (photos.length === 0) {
      return;
    }

    const ref = this.dialogService.open(TagEditorComponent, {
      title: 'Tag Photos',
      width: '700px',
      actions: [
        { label: 'Cancel', value: false, style: 'ghost' },
        { label: 'Apply', value: 'submit', style: 'primary' }
      ]
    });

    ref.afterClosed().then(result => {
      if (!result || result === 'submit' || result === false) {
        return;
      }

      this.photoService.updatePhotoTags(result.photoIds, result.tags).subscribe({
        next: () => {
          this.selectionService.clearSelection();
        },
        error: (err) => {
          console.error('Failed to update tags', err);
          alert('Failed to update photo tags.');
        }
      });
    });
  }

  private loadSiteSettings(): void {
    this.settingsService
      .getSettingByName('site.title')
      .pipe(
        first(),
        catchError(() => of(null))
      )
      .subscribe(setting => {
        if (typeof setting?.value === 'string' && setting.value.trim().length) {
          this.siteTitle.set(setting.value);
          this.setBrowserTitle(setting.value);
        }
      });

    this.settingsService
      .getSettingByName('site.tagline')
      .pipe(
        first(),
        catchError(() => of(null))
      )
      .subscribe(setting => {
        if (typeof setting?.value === 'string' && setting.value.trim().length) {
          this.siteTagline.set(setting.value);
        }
      });

    this.settingsService
      .getSettingByName('site.allowRegistration')
      .pipe(
        first(),
        catchError(() => of(null))
      )
      .subscribe(setting => {
        if (typeof setting?.value === 'boolean') {
          this.allowRegistration.set(setting.value);
        }
      });

    this.settingsService
      .getLogoUrl()
      .subscribe(logoUrl => {
        if (typeof logoUrl === 'string' && logoUrl.trim().length) {
          this.siteLogo.set(logoUrl);
          this.setFavicon(logoUrl);
        }
      });
  }

  private setFavicon(url: string): void {
    const head = this.document.head;
    if (!head) {
      return;
    }

    const existing = head.querySelector('link[rel="icon"]') as HTMLLinkElement | null;
    if (existing) {
      existing.href = url;
      return;
    }

    const link = this.document.createElement('link');
    link.rel = 'icon';
    link.type = 'image/png';
    link.href = url;
    head.appendChild(link);
  }

  private setBrowserTitle(title: string): void {
    this.document.title = title;
  }
}
