import { Component, computed, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { ActivatedRoute, NavigationEnd, Router, RouterOutlet } from '@angular/router';
import { filter } from 'rxjs';

type SetupStep = {
    path: string;
    title: string;
    description?: string;
};

@Component({
    selector: 'mtx-wizard',
    imports: [RouterOutlet],
    templateUrl: './wizard.component.html',
    host: {
        class: 'flex-1 flex flex-col min-h-0',
    },
})
export class WizardComponent {
    private readonly router = inject(Router);
    private readonly route = inject(ActivatedRoute);
    readonly activePath = signal('');

    readonly steps = computed<SetupStep[]>(() => {
        const children = this.route.routeConfig?.children ?? [];
        return children
            .filter(child => !!child.path && child.path !== '' && !child.redirectTo)
            .map(child => ({
                path: child.path ?? '',
                title: (child.data?.['title'] as string) ?? this.formatTitle(child.path ?? ''),
                description: child.data?.['description'] as string | undefined,
            }));
    });
    readonly activeIndex = computed(() => {
        const index = this.steps().findIndex(step => step.path === this.activePath());
        return index === -1 ? 0 : index;
    });
    readonly progress = computed(() => {
        const total = this.steps().length || 1;
        return ((this.activeIndex() + 1) / total) * 100;
    });
    readonly isLastStep = computed(() => this.activeIndex() >= this.steps().length - 1);

    constructor() {
        this.router.events
            .pipe(
                filter((event): event is NavigationEnd => event instanceof NavigationEnd),
                takeUntilDestroyed(),
            )
            .subscribe(() => this.syncActivePath());

        this.syncActivePath();
    }

    goBack(): void {
        const index = this.activeIndex();
        if (index <= 0) {
            return;
        }
        const step = this.steps()[index - 1];
        if (step) {
            void this.router.navigate([step.path], { relativeTo: this.route });
        }
    }

    goNext(): void {
        if (this.isLastStep()) {
            void this.router.navigate(['/']);
            return;
        }

        const index = this.activeIndex();
        const step = this.steps()[index + 1];
        if (step) {
            void this.router.navigate([step.path], { relativeTo: this.route });
        }
    }

    goTo(path: string): void {
        if (path && path !== this.activePath()) {
            void this.router.navigate([path], { relativeTo: this.route });
        }
    }

    private syncActivePath(): void {
        const childPath = this.route.firstChild?.routeConfig?.path ?? '';
        this.activePath.set(childPath);
    }

    private formatTitle(path: string): string {
        return path
            .split('-')
            .filter(Boolean)
            .map(segment => segment[0]?.toUpperCase() + segment.slice(1))
            .join(' ');
    }
}
