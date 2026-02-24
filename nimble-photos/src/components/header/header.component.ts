import { DOCUMENT } from '@angular/common';
import { Component, computed, ElementRef, HostListener, inject, OnInit, signal, ViewChild } from '@angular/core';

import { ActivatedRoute, RouterModule } from '@angular/router';
import { Nav } from '../../models/command';
import { SettingNames } from '../../models/setting.names';
import { AuthService } from '../../services/auth.service';
import { SettingsService } from '../../services/settings.service';
import { SvgComponent } from '../svg/svg.component';
import { HeaderActionsComponent } from './header.actions.component';

@Component({
  selector: 'mtx-header',
  imports: [RouterModule, SvgComponent, HeaderActionsComponent],
  templateUrl: './header.component.html',
  styles: [`
    :host {
      display: block;
      width: 100%;
    }
  `]
})
export class HeaderComponent implements OnInit {
  private readonly route = inject(ActivatedRoute);
  private readonly settingsService = inject(SettingsService);
  private readonly document = inject(DOCUMENT);
  readonly authService = inject(AuthService);

  readonly isMenuOpen = signal(false);
  readonly isUserMenuOpen = signal(false);
  readonly siteTitle = signal('Nimble Photos');
  readonly siteTagline = signal('My Photo Stories');
  readonly siteLogo = signal<string | null>(null);
  readonly allowRegistration = signal(true);
  readonly isAuthenticated = this.authService.isAuthenticated;
  readonly currentUser = this.authService.currentUser;
  readonly canAccessDashboard = this.authService.canAccessDashboard;

  readonly mainNavs = computed(() => {
    let navItems: Nav[] = []
    if (this.isAuthenticated()) {
      navItems = [
        { id: 'nav-timeline', label: 'Timeline', route: '/' },
        { id: 'nav-albums', label: 'Albums', route: '/albums' },
        { id: 'nav-map', label: 'Map', route: '/map' },
      ]
    }
    return navItems;
  }
  );

  readonly userMenuNavs = computed(() => {
    const navItems: Nav[] = [];
    if (this.isAuthenticated()) {
      if (this.authService.canAccessDashboard()) {
        navItems.push({ id: 'nav-dashboard', label: 'Dashboard', route: '/dashboard' });
      }

      navItems.push({
        id: 'nav-logout', label: 'Sign Out', action: () => {
          this.logout();
        }
      });
    } else {
      navItems.push({ id: 'nav-login', label: 'Sign In', route: '/login' });
      if (this.allowRegistration()) {
        navItems.push({ id: 'nav-register', label: 'Register', route: '/register' });
      }
    }
    return navItems;
  });

  @ViewChild('userMenuRoot') userMenuRoot?: ElementRef<HTMLElement>;

  ngOnInit(): void {
    this.setBrowserTitle(this.siteTitle());
    this.loadSiteSettings();
    this.route.queryParams.subscribe(params => {
      if (params['registered']) {
      }
    });
  }

  logout() {
    this.authService.logout();
    this.closeMenu();
  }

  toggleMenu() {
    this.isMenuOpen.update(v => !v);
    if (this.isMenuOpen()) this.isUserMenuOpen.set(false);
  }

  toggleUserMenu() {
    this.isUserMenuOpen.update(v => !v);
  }

  @HostListener('document:click', ['$event'])
  handleDocumentClick(event: MouseEvent) {
    if (!this.isUserMenuOpen()) {
      return;
    }

    const target = event.target as Node | null;
    if (!target) {
      return;
    }

    if (this.userMenuRoot?.nativeElement.contains(target)) {
      return;
    }

    this.isUserMenuOpen.set(false);
  }

  closeMenu() {
    this.isMenuOpen.set(false);
    this.isUserMenuOpen.set(false);
  }

  closeUserMenu() {
    this.isUserMenuOpen.set(false);
  }

  private loadSiteSettings(): void {
    this.settingsService.getSetting<string>(SettingNames.SiteTitle, value => {
      this.siteTitle.set(value);
      this.setBrowserTitle(value);
    });
    this.settingsService.getSetting<string>(SettingNames.SiteTagline, value => this.siteTagline.set(value));
    this.settingsService.getSetting<boolean>(SettingNames.SiteAllowRegistration, value => this.allowRegistration.set(value));
    this.settingsService.getSetting<string>(SettingNames.SiteLogo, value => {
      console.log('Site logo URL:', value);
      this.siteLogo.set(value);
      this.setFavicon(value);
    });
  }

  private setFavicon(url: string): void {
    const head = this.document.head;
    if (!head) {
      return;
    }

    const existing = head.querySelector('link[rel="icon"]') as HTMLLinkElement | null;
    if (existing) {
      existing.href = url;
      return;
    }

    const link = this.document.createElement('link');
    link.rel = 'icon';
    link.type = 'image/png';
    link.href = url;
    head.appendChild(link);
  }

  private setBrowserTitle(title: string): void {
    this.document.title = title;
  }
}
