import { CommonModule } from '@angular/common';
import { Component, EventEmitter, Input, Output, forwardRef, signal } from '@angular/core';
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from '@angular/forms';

import { SvgComponent } from '../svg/svg.component';

@Component({
    selector: 'mtx-dropdown',
    imports: [CommonModule, SvgComponent],
    templateUrl: './dropdown.component.html',
    providers: [
        {
            provide: NG_VALUE_ACCESSOR,
            useExisting: forwardRef(() => DropdownComponent),
            multi: true,
        },
    ],
})
export class DropdownComponent implements ControlValueAccessor {
    @Input() options: readonly unknown[] = [];
    @Input() placeholder = 'Select an option';
    @Input() emptyText = 'No options available';
    @Input() disabled = false;
    @Input() valueKey = 'value';
    @Input() labelKey = 'label';
    @Input() descriptionKey = '';
    @Input() labelFn?: (option: unknown) => string;
    @Input() descriptionFn?: (option: unknown) => string;
    @Input() valueFn?: (option: unknown) => unknown;
    @Output() valueChange = new EventEmitter<unknown>();
    @Output() optionChange = new EventEmitter<unknown>();

    readonly open = signal(false);
    private value: unknown = null;
    private onChange: (value: unknown) => void = () => { };
    private onTouched: () => void = () => { };

    writeValue(value: unknown): void {
        this.value = value;
    }

    registerOnChange(fn: (value: unknown) => void): void {
        this.onChange = fn;
    }

    registerOnTouched(fn: () => void): void {
        this.onTouched = fn;
    }

    setDisabledState(isDisabled: boolean): void {
        this.disabled = isDisabled;
    }

    toggle(): void {
        if (this.disabled) {
            return;
        }
        this.open.set(!this.open());
    }

    close(): void {
        this.open.set(false);
    }

    selectedLabel(): string {
        const selected = this.options.find((option) => this.valuesEqual(this.getOptionValue(option), this.value));
        return selected ? this.getOptionLabel(selected) : this.placeholder;
    }

    select(option: unknown): void {
        if (this.disabled) {
            return;
        }

        const nextValue = this.getOptionValue(option);
        this.value = nextValue;
        this.onChange(nextValue);
        this.onTouched();
        this.valueChange.emit(nextValue);
        this.optionChange.emit(option);
        this.close();
    }

    isSelected(option: unknown): boolean {
        return this.valuesEqual(this.getOptionValue(option), this.value);
    }

    getOptionLabel(option: unknown): string {
        if (this.labelFn) {
            return this.labelFn(option);
        }
        return this.readString(option, this.labelKey);
    }

    getOptionDescription(option: unknown): string {
        if (this.descriptionFn) {
            return this.descriptionFn(option);
        }
        if (!this.descriptionKey) {
            return '';
        }
        return this.readString(option, this.descriptionKey);
    }

    private getOptionValue(option: unknown): unknown {
        if (this.valueFn) {
            return this.valueFn(option);
        }
        return this.readProperty(option, this.valueKey);
    }

    private readString(option: unknown, key: string): string {
        const value = this.readProperty(option, key);
        return typeof value === 'string' ? value : String(value ?? '');
    }

    private readProperty(option: unknown, key: string): unknown {
        if (!option || typeof option !== 'object') {
            return null;
        }
        return (option as Record<string, unknown>)[key];
    }

    private valuesEqual(left: unknown, right: unknown): boolean {
        return String(left ?? '') === String(right ?? '');
    }
}
