import { Routes } from '@angular/router';

export const routes: Routes = [
    { path: '', loadComponent: () => import('../components/grouped-gallery/grouped-gallery').then(m => m.GroupedGallery) },
    { path: 'albums', loadComponent: () => import('../components/albums/albums.component').then(m => m.AlbumsComponent) },
    { path: 'album/:id', loadComponent: () => import('../components/album-detail/album-detail.component').then(m => m.AlbumDetailComponent) },
    { path: 'all', loadComponent: () => import('../components/gallery/gallery.component').then(m => m.GalleryComponent) }
];

