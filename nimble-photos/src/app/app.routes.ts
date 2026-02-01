import { Routes } from '@angular/router';

export const routes: Routes = [
    { path: '', loadComponent: () => import('../components/gallery/gallery.component').then(m => m.GalleryComponent) },
    { path: 'timeline', loadComponent: () => import('../components/grouped-gallery/grouped-gallery').then(m => m.GroupedGallery) }
];
