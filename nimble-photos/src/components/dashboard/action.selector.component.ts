import { Component, ElementRef, EventEmitter, HostListener, Input, Output, inject, signal } from '@angular/core';
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

    @Input() actions: readonly ActionSelectorItem[] = [];
    @Input() selectedKeys: readonly string[] = [];
    @Input() disabled = false;
    @Input() readOnly = false;
    @Output() selectedKeysChange = new EventEmitter<string[]>();

    readonly isOpen = signal(false);

    toggleOpen(): void {
        if (this.disabled || this.readOnly) {
            return;
        }
        this.isOpen.update(current => !current);
    }

    isSelected(key: string): boolean {
        return this.selectedKeys.includes(key);
    }

    toggleAction(key: string, checked: boolean): void {
        if (this.disabled || this.readOnly) {
            return;
        }

        const selected = new Set(this.selectedKeys);
        if (checked) {
            selected.add(key);
        } else {
            selected.delete(key);
        }
        this.selectedKeysChange.emit(Array.from(selected));
    }

    summary(): string {
        if (!this.actions.length) {
            return 'No actions';
        }

        const selectedCount = this.actions.filter(action => this.selectedKeys.includes(action.key)).length;
        if (selectedCount === this.actions.length) {
            return 'All actions';
        }
        if (selectedCount === 0) {
            return 'No actions';
        }
        return `${selectedCount} of ${this.actions.length} actions`;
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

