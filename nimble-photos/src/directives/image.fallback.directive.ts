import { Directive, ElementRef, HostListener, Input } from '@angular/core';

@Directive({
    selector: 'img[fallback]'
})
export class ImageFallbackDirective {
    @Input() fallback?: string;

    private readonly defaultPlaceholder = `data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64' fill='none' stroke='%23475569' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'%3e%3crect width='64' height='64' fill='%230f172a' stroke='none'/%3e%3cg transform='translate(20,20)'%3e%3crect x='3' y='3' width='18' height='18' rx='2' ry='2' /%3e%3ccircle cx='8.5' cy='8.5' r='1.5' /%3e%3cpolyline points='21 15 16 10 5 21' /%3e%3c/g%3e%3c/svg%3e`;

    constructor(private el: ElementRef<HTMLImageElement>) { }

    @HostListener('error')
    onError() {
        if (this.el.nativeElement.src !== this.defaultPlaceholder && this.el.nativeElement.src !== this.fallback) {
            this.el.nativeElement.src = this.fallback || this.defaultPlaceholder;
            this.el.nativeElement.classList.add('is-placeholder');
        }
    }
}
