import { Component, OnInit, signal } from '@angular/core';
import { ActivatedRoute, RouterModule } from '@angular/router';
import { first } from 'rxjs';
import { PhotoService } from '../../services/photo.service';
import { Album } from '../../models/photo.model';
import { DatePipe } from '@angular/common';

@Component({
    selector: 'mtx-album-detail',
    standalone: true,
    imports: [RouterModule, DatePipe],
    templateUrl: './album-detail.component.html',
    host: {
        class: 'block flex-1 min-h-0',
    },
})
export class AlbumDetailComponent implements OnInit {
    readonly album = signal<Album | null>(null);
    readonly loading = signal(false);

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
}
