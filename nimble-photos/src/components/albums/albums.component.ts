import { Component, OnInit, signal } from '@angular/core';
import { RouterModule } from '@angular/router';
import { first } from 'rxjs';
import { PhotoService } from '../../services/photo.service';
import { Album } from '../../models/photo.model';
import { DatePipe } from '@angular/common';

@Component({
    selector: 'mtx-albums',
    imports: [RouterModule, DatePipe],
    templateUrl: './albums.component.html',
    host: {
        class: 'block flex-1 min-h-0',
    },
})
export class AlbumsComponent implements OnInit {
    readonly albums = signal<Album[]>([]);
    readonly loading = signal(false);

    constructor(private readonly photoService: PhotoService) { }

    ngOnInit(): void {
        this.fetchAlbums();
    }

    private fetchAlbums(): void {
        this.loading.set(true);
        this.photoService.getAlbums().pipe(first()).subscribe(result => {
            this.albums.set(result.items);
            this.loading.set(false);
        });
    }
}
