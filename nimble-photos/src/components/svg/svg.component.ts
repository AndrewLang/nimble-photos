import { CommonModule } from '@angular/common';
import { Component, Input, computed, inject, signal } from '@angular/core';
import { DomSanitizer, SafeHtml } from '@angular/platform-browser';

import { SvgIcons } from './svg.icons';

@Component({
    selector: 'mtx-svg',
    standalone: true,
    imports: [CommonModule],
    template: `<span
        class="inline-flex items-center justify-center"
        [ngClass]="className"
        [style.width]="size"
        [style.height]="size"
        [attr.aria-label]="ariaLabel"
        [attr.role]="ariaLabel ? 'img' : null"
        [innerHTML]="sanitizedSvg()"
    ></span>`,
})
export class SvgComponent {
    private readonly sanitizer = inject(DomSanitizer);

    private readonly iconName = signal('');

    @Input() set name(value: string) {
        this.iconName.set(value ?? '');
    }

    @Input() size: string = '1.25rem';
    @Input() className = '';
    @Input() ariaLabel?: string;

    readonly sanitizedSvg = computed<SafeHtml>(() => {
        const svg = SvgIcons[this.iconName()] ?? '';
        return this.sanitizer.bypassSecurityTrustHtml(svg);
    });
}

