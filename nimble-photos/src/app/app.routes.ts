import { Routes } from '@angular/router';

export const routes: Routes = [
    { path: 'all', loadComponent: () => import('../components/gallery/gallery.component').then(m => m.GalleryComponent) },
    { path: '', loadComponent: () => import('../components/grouped-gallery/grouped-gallery').then(m => m.GroupedGallery) }
];
