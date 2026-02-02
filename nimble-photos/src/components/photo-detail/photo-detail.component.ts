import { Component, OnInit, signal, HostListener } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ActivatedRoute, RouterModule, Router } from '@angular/router';
import { first } from 'rxjs';
import { PhotoService } from '../../services/photo.service';
import { Photo } from '../../models/photo.model';

@Component({
    selector: 'mtx-photo-detail',
    imports: [CommonModule, RouterModule],
    templateUrl: './photo-detail.component.html',
    host: {
        class: 'fixed inset-0 z-[100] bg-slate-950 flex flex-col',
    }
})
export class PhotoDetailComponent implements OnInit {
    readonly photo = signal<Photo | null>(null);
    readonly loading = signal(false);
    readonly adjacents = signal<{ prevId: string | null; nextId: string | null }>({ prevId: null, nextId: null });
    readonly reactions = signal<{ emoji: string; count: number; selected: boolean }[]>([
        { emoji: '❤️', count: 12, selected: false },
        { emoji: '🔥', count: 8, selected: false },
        { emoji: '👏', count: 5, selected: false },
        { emoji: '😮', count: 2, selected: false },
        { emoji: '✨', count: 4, selected: false },
    ]);

    private albumId: string | null = null;

    constructor(
        private readonly route: ActivatedRoute,
        private readonly router: Router,
        private readonly photoService: PhotoService
    ) { }

    ngOnInit(): void {
        this.route.paramMap.subscribe(params => {
            const id = params.get('id');
            this.albumId = params.get('albumId');
            if (id) {
                this.fetchPhoto(id);
            }
        });
    }

    @HostListener('window:keydown', ['$event'])
    handleKeyDown(event: KeyboardEvent): void {
        if (event.key === 'ArrowRight' && this.adjacents().nextId) {
            this.navigateToPhoto(this.adjacents().nextId!);
        } else if (event.key === 'ArrowLeft' && this.adjacents().prevId) {
            this.navigateToPhoto(this.adjacents().prevId!);
        } else if (event.key === 'Escape') {
            this.close();
        }
    }

    private fetchPhoto(id: string): void {
        this.loading.set(true);
        this.photoService.getPhotoById(id).pipe(first()).subscribe(result => {
            this.photo.set(result);
            if (result) {
                this.fetchAdjacents(result.id);
            }
            this.loading.set(false);
        });
    }

    private fetchAdjacents(id: string): void {
        this.photoService.getAdjacentPhotos(id, this.albumId || undefined)
            .pipe(first())
            .subscribe(adj => this.adjacents.set(adj));
    }

    addReaction(emoji: string): void {
        this.reactions.update(prev => prev.map(r => {
            if (r.emoji === emoji) {
                return { ...r, count: r.selected ? r.count - 1 : r.count + 1, selected: !r.selected };
            }
            return r;
        }));
    }

    navigateToPhoto(id: string): void {
        const commands = this.albumId
            ? ['/album', this.albumId, 'photo', id]
            : ['/photo', id];
        this.router.navigate(commands);
    }

    close(): void {
        if (this.albumId) {
            this.router.navigate(['/album', this.albumId]);
        } else {
            // Logic for where they came from might be needed, 
            // but for now go to home (timeline)
            this.router.navigate(['/']);
        }
    }

    formatBytes(size?: number): string {
        if (!size || size <= 0) {
            return 'n/a';
        }
        const units = ['B', 'KB', 'MB', 'GB'];
        let value = size;
        let index = 0;
        while (value >= 1024 && index < units.length - 1) {
            value /= 1024;
            index += 1;
        }
        return `${value.toFixed(1)} ${units[index]}`;
    }
}
