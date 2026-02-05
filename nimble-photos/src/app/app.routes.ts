import { Routes } from '@angular/router';

export const routes: Routes = [
    { path: '', loadComponent: () => import('../components/grouped.gallery/grouped.gallery').then(m => m.GroupedGallery) },
    { path: 'photo/:id', loadComponent: () => import('../components/photo.detail/photo.detail.component').then(m => m.PhotoDetailComponent) },
    { path: 'album/:id', loadComponent: () => import('../components/album.detail/album.detail.component').then(m => m.AlbumDetailComponent) },
    { path: 'albums', loadComponent: () => import('../components/albums/albums.component').then(m => m.AlbumsComponent) },
    { path: 'album/:albumId/photo/:id', loadComponent: () => import('../components/photo.detail/photo.detail.component').then(m => m.PhotoDetailComponent) },
    { path: 'login', loadComponent: () => import('../components/auth/login.component').then(m => m.LoginComponent) },
    { path: 'register', loadComponent: () => import('../components/auth/register.component').then(m => m.RegisterComponent) },
    { path: 'forgot-password', loadComponent: () => import('../components/auth/forgot.password.component').then(m => m.ForgotPasswordComponent) },
    { path: 'map', loadComponent: () => import('../components/map/map.component').then(m => m.MapComponent) },
    {
        path: 'setup',
        loadComponent: () => import('../components/wizard/wizard.component').then(m => m.WizardComponent),
        children: [
            { path: '', pathMatch: 'full', redirectTo: 'welcome' },
            {
                path: 'welcome',
                loadComponent: () => import('../components/wizard/steps/welcome.step.component').then(m => m.WelcomeStepComponent),
                data: {
                    title: 'Welcome',
                    description: 'Meet Nimble Photos and prep for setup.',
                },
            },
            {
                path: 'admin-user',
                loadComponent: () => import('../components/wizard/steps/user.step.component').then(m => m.UserStepComponent),
                data: {
                    title: 'Admin User',
                    description: 'Create the primary admin account.',
                },
            },
        ],
    },
    {
        path: 'dashboard',
        loadComponent: () => import('../components/dashboard/dashboard.component').then(m => m.DashboardComponent),
        children: [
            { path: '', pathMatch: 'full', redirectTo: 'general' },
            { path: 'general', loadComponent: () => import('../components/dashboard/general.setting.component').then(m => m.GeneralSettingComponent) },
            { path: 'photo-manage', loadComponent: () => import('../components/dashboard/photo.manage.setting.component').then(m => m.PhotoManageSettingComponent) },
        ],
    },
    { path: 'all', loadComponent: () => import('../components/gallery/gallery.component').then(m => m.GalleryComponent) },
];

