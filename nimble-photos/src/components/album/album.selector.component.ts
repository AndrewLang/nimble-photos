import { Component, OnInit, signal, computed, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { PhotoService } from '../../services/photo.service';
import { Album } from '../../models/photo';

@Component({
    selector: 'mtx-album-selector',
    imports: [CommonModule, FormsModule],
    templateUrl: './album.selector.component.html'
})
export class AlbumSelectorComponent implements OnInit {
    private readonly photoService = inject(PhotoService);

    readonly albums = signal<Album[]>([]);
    readonly searchQuery = signal('');
    readonly selectedAlbum = signal<Album | null>(null);
    readonly loading = signal(false);

    readonly filteredAlbums = computed(() => {
        const query = this.searchQuery().toLowerCase();
        return this.albums().filter(a => a.name.toLowerCase().includes(query));
    });

    ngOnInit() {
        this.fetchAlbums();
    }

    private fetchAlbums() {
        this.loading.set(true);
        this.photoService.getAlbums(1, 100).subscribe(result => {
            this.albums.set(result.items);
            this.loading.set(false);
        });
    }

    selectAlbum(album: Album) {
        this.selectedAlbum.set(album);
    }

    getThumbnailUrl(album: Album): string | null {
        if (!album.thumbnailHash) return null;
        return (this.photoService as any).getThumbnailPath({ hash: album.thumbnailHash } as any);
    }

    isValid() {
        return this.selectedAlbum() !== null;
    }

    getFormValue() {
        return this.selectedAlbum();
    }
}
