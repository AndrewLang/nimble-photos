import { Component, ElementRef, HostListener, inject, input, output, signal } from '@angular/core';
import { SvgComponent } from '../svg/svg.component';

export interface ActionSelectorItem {
    key: string;
    label: string;
}

@Component({
    selector: 'mtx-action-selector',
    imports: [SvgComponent],
    templateUrl: './action.selector.component.html',
})
export class ActionSelectorComponent {
    private readonly host = inject(ElementRef<HTMLElement>);

    readonly actions = input<readonly ActionSelectorItem[]>([]);
    readonly selectedKeys = input<readonly string[]>([]);
    readonly disabled = input(false);
    readonly readOnly = input(false);
    readonly selectedKeysChange = output<string[]>();

    readonly isOpen = signal(false);

    toggleOpen(): void {
        if (this.disabled() || this.readOnly()) {
            return;
        }
        this.isOpen.update(current => !current);
    }

    isSelected(key: string): boolean {
        return this.selectedKeys().includes(key);
    }

    toggleAction(key: string, checked: boolean): void {
        if (this.disabled() || this.readOnly()) {
            return;
        }

        const selected = new Set(this.selectedKeys());
        if (checked) {
            selected.add(key);
        } else {
            selected.delete(key);
        }
        this.selectedKeysChange.emit(Array.from(selected));
    }

    summary(): string {
        const actions = this.actions();
        if (!actions.length) {
            return 'No actions';
        }

        const selectedSet = new Set(this.selectedKeys());
        const selectedCount = actions.filter(action => selectedSet.has(action.key)).length;
        if (selectedCount === actions.length) {
            return 'All actions';
        }
        if (selectedCount === 0) {
            return 'No actions';
        }
        return `${selectedCount} of ${actions.length} actions`;
    }

    @HostListener('document:click', ['$event'])
    onDocumentClick(event: MouseEvent): void {
        if (!this.isOpen()) {
            return;
        }

        if (!this.host.nativeElement.contains(event.target as Node)) {
            this.isOpen.set(false);
        }
    }
}
