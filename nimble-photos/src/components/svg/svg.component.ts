import { CommonModule } from '@angular/common';
import { Component, Input, signal } from '@angular/core';

import { SvgIcon } from './svg.icons';

@Component({
    selector: 'mtx-svg',
    standalone: true,
    imports: [CommonModule],
    template: `
    <svg
        [attr.viewBox]="iconDef.viewBox" [attr.width]="size" [attr.height]="size" [attr.aria-hidden]="ariaHidden"
        [attr.role]="ariaLabel ? 'img' : 'presentation'" [attr.aria-label]="ariaLabel || null" [ngClass]="svgClass"
        [attr.fill]="fillColor" [attr.stroke]="strokeColor" stroke-linecap="round" stroke-linejoin="round"
        [attr.stroke-width]="strokeThickness">

        @for(d of iconDef.paths; track d){
            @if (isMarkupPath(d)) {
                <g [innerHTML]="toPathMarkup(d)"></g>
            } @else {
            <path [attr.d]="d"></path>
            }
        }
    </svg>
    `
    // templateUrl: './svg.component.html',
    // styles: [`
    //     .mtx-svg svg {
    //         width: 100% !important;
    //         height: 100% !important;
    //         display: block;
    //     }
    // `],
})
export class SvgComponent {
    // private readonly sanitizer = inject(DomSanitizer);
    // private readonly iconName = signal<string>('');

    // @Input() set name(value: string) {
    //     this.iconName.set(value ?? '');
    // }

    // @Input() size: string = '1.25rem';
    // @Input() className = '';
    // @Input() ariaLabel?: string;

    // readonly svgContent = computed<SafeHtml>(() => {
    //     const svg = SvgIcon.get(this.iconName());
    //     return this.sanitizer.bypassSecurityTrustHtml(this.normalizeSvg(svg));
    // });

    // private normalizeSvg(raw: string): string {
    //     if (!raw) {
    //         return '';
    //     }

    //     if (typeof DOMParser === 'undefined') {
    //         return raw;
    //     }

    //     try {
    //         const parser = new DOMParser();
    //         const doc = parser.parseFromString(raw, 'image/svg+xml');
    //         const svg = doc.querySelector('svg');
    //         if (!svg) {
    //             return raw;
    //         }

    //         svg.removeAttribute('width');
    //         svg.removeAttribute('height');
    //         svg.removeAttribute('class');
    //         svg.setAttribute('width', '100%');
    //         svg.setAttribute('height', '100%');
    //         svg.setAttribute('preserveAspectRatio', 'xMidYMid meet');

    //         return svg.outerHTML;
    //     } catch {
    //         return raw;
    //     }
    // }

    private readonly iconName = signal<string>('');
    @Input() set name(value: string) {
        this.iconName.set(value ?? '');
    }
    @Input() size = 20;
    @Input() strokeThickness = 1.5;
    @Input() strokeColor = 'currentColor';
    @Input() fillColor = 'none';
    @Input() svgClass = '';
    @Input() ariaLabel?: string;

    constructor() { }

    ngOnInit() { }

    get ariaHidden(): 'true' | 'false' {
        return this.ariaLabel ? 'false' : 'true';
    }

    get iconDef(): SvgIcon {
        let icon = SvgIcon.getIcon(this.iconName());

        return icon || { name: 'default', viewBox: '0 0 24 24', paths: [] };
    }

    isMarkupPath(value: string): boolean {
        return value.trim().startsWith('<');
    }

    toPathMarkup(value: string): string {
        const trimmed = value.trim();
        if (trimmed.endsWith('/>') || trimmed.includes('</')) {
            return trimmed;
        }
        return `${trimmed} />`;
    }
}
