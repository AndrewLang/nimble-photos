import { Routes } from '@angular/router';

export const routes: Routes = [
    { path: '', loadComponent: () => import('../components/grouped-gallery/grouped-gallery').then(m => m.GroupedGallery) },
    { path: 'albums', loadComponent: () => import('../components/albums/albums.component').then(m => m.AlbumsComponent) },
    { path: 'album/:id', loadComponent: () => import('../components/album-detail/album-detail.component').then(m => m.AlbumDetailComponent) },
    { path: 'photo/:id', loadComponent: () => import('../components/photo-detail/photo-detail.component').then(m => m.PhotoDetailComponent) },
    { path: 'album/:albumId/photo/:id', loadComponent: () => import('../components/photo-detail/photo-detail.component').then(m => m.PhotoDetailComponent) },
    { path: 'login', loadComponent: () => import('../components/auth/login.component').then(m => m.LoginComponent) },
    { path: 'register', loadComponent: () => import('../components/auth/register.component').then(m => m.RegisterComponent) },
    { path: 'forgot-password', loadComponent: () => import('../components/auth/forgot-password.component').then(m => m.ForgotPasswordComponent) },
    { path: 'map', loadComponent: () => import('../components/map/map.component').then(m => m.MapComponent) },
    { path: 'all', loadComponent: () => import('../components/gallery/gallery.component').then(m => m.GalleryComponent) },
    { path: 'justified', loadComponent: () => import('../components/justified-gallery/justified-gallery.component').then(m => m.JustifiedGalleryComponent) }
];

