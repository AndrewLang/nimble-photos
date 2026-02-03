import { Component, OnInit, signal, computed } from '@angular/core';
import { ActivatedRoute, RouterModule } from '@angular/router';
import { CommonModule, DatePipe } from '@angular/common';
import { first } from 'rxjs';
import { PhotoService } from '../../services/photo.service';
import { Album, Photo } from '../../models/photo';
import { GalleryComponent } from '../gallery/gallery.component';

@Component({
    selector: 'mtx-album-detail',
    imports: [CommonModule, RouterModule, DatePipe, GalleryComponent],
    templateUrl: './album.detail.component.html',
    host: {
        class: 'block flex-1 min-h-0',
    },
})
export class AlbumDetailComponent implements OnInit {
    readonly album = signal<Album | null>(null);
    readonly loading = signal(false);

    readonly albumPhotos = computed<Photo[]>(() => {
        return this.album()?.photos?.items ?? [];
    });

    constructor(
        private readonly route: ActivatedRoute,
        private readonly photoService: PhotoService
    ) { }

    ngOnInit(): void {
        this.route.paramMap.subscribe(params => {
            const id = params.get('id');
            if (id) {
                this.fetchAlbum(id);
            }
        });
    }

    private fetchAlbum(id: string): void {
        this.loading.set(true);
        this.photoService.getAlbumById(id).pipe(first()).subscribe(result => {
            this.album.set(result);
            this.loading.set(false);
        });
    }

    getImageUrl(photo: Photo): string {
        return this.photoService.getThumbnailPath(photo);
    }

    getAspectRatio(photo: Photo): string {
        if (photo.width && photo.height) {
            return `${photo.width} / ${photo.height}`;
        }
        return '4 / 3';
    }
}
