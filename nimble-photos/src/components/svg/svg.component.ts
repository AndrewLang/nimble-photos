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
})
export class SvgComponent {
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
