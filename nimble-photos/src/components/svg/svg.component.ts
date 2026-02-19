import { CommonModule } from '@angular/common';
import { Component, input } from '@angular/core';

import { SvgIcon } from './svg.icons';

@Component({
    selector: 'mtx-svg',
    standalone: true,
    imports: [CommonModule],
    template: `
    <svg
        [attr.viewBox]="iconDef.viewBox" [attr.width]="size()" [attr.height]="size()" [attr.aria-hidden]="ariaHidden"
        [attr.role]="ariaLabel() ? 'img' : 'presentation'" [attr.aria-label]="ariaLabel() || null" [ngClass]="svgClass()"
        [attr.fill]="fillColor()" [attr.stroke]="strokeColor()" stroke-linecap="round" stroke-linejoin="round"
        [attr.stroke-width]="strokeThickness()">

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
    readonly name = input('');
    readonly size = input(20);
    readonly strokeThickness = input(1.5);
    readonly strokeColor = input('currentColor');
    readonly fillColor = input('none');
    readonly svgClass = input('');
    readonly ariaLabel = input<string | undefined>(undefined);

    constructor() { }

    ngOnInit() { }

    get ariaHidden(): 'true' | 'false' {
        return this.ariaLabel() ? 'false' : 'true';
    }

    get iconDef(): SvgIcon {
        const icon = SvgIcon.getIcon(this.name() || '');

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
