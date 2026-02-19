import { CommonModule } from '@angular/common';
import { Component, computed, forwardRef, input, output, signal } from '@angular/core';
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from '@angular/forms';

import { SvgComponent } from '../svg/svg.component';
import { NamedValue } from '../../models/namedvalue';

type DropdownOption = NamedValue<unknown>;

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
    readonly options = input<readonly DropdownOption[]>([]);
    readonly placeholder = input('Select an option');
    readonly emptyText = input('No options available');
    readonly disabled = input(false);
    readonly valueKey = input('value');
    readonly labelKey = input('name');
    readonly descriptionKey = input('');
    readonly labelFn = input<((option: DropdownOption) => string) | undefined>(undefined);
    readonly descriptionFn = input<((option: DropdownOption) => string) | undefined>(undefined);
    readonly valueFn = input<((option: DropdownOption) => unknown) | undefined>(undefined);
    readonly valueChange = output<unknown>();
    readonly optionChange = output<DropdownOption>();

    readonly open = signal(false);
    private readonly disabledState = signal(false);
    private value: unknown = null;
    private onChange: (value: unknown) => void = () => { };
    private onTouched: () => void = () => { };
    readonly isDisabled = computed(() => this.disabled() || this.disabledState());

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
        this.disabledState.set(isDisabled);
    }

    toggle(): void {
        if (this.isDisabled()) {
            return;
        }
        this.open.set(!this.open());
    }

    close(): void {
        this.open.set(false);
    }

    selectedLabel(): string {
        const options = this.options();
        const selected = options.find((option) => this.valuesEqual(this.getOptionValue(option), this.value));
        return selected ? this.getOptionLabel(selected) : this.placeholder();
    }

    select(option: DropdownOption): void {
        if (this.isDisabled()) {
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

    isSelected(option: DropdownOption): boolean {
        return this.valuesEqual(this.getOptionValue(option), this.value);
    }

    getOptionLabel(option: DropdownOption): string {
        const labelFn = this.labelFn();
        if (labelFn) {
            return labelFn(option);
        }
        return this.readString(option, this.labelKey());
    }

    getOptionDescription(option: DropdownOption): string {
        const descriptionFn = this.descriptionFn();
        if (descriptionFn) {
            return descriptionFn(option);
        }
        if (!this.descriptionKey()) {
            return '';
        }
        return this.readString(option, this.descriptionKey());
    }

    private getOptionValue(option: DropdownOption): unknown {
        const valueFn = this.valueFn();
        if (valueFn) {
            return valueFn(option);
        }
        return this.readProperty(option, this.valueKey());
    }

    private readString(option: DropdownOption, key: string): string {
        const value = this.readProperty(option, key);
        return typeof value === 'string' ? value : String(value ?? '');
    }

    private readProperty(option: DropdownOption, key: string): unknown {
        return (option as Record<string, unknown>)[key] ?? null;
    }

    private valuesEqual(left: unknown, right: unknown): boolean {
        return String(left ?? '') === String(right ?? '');
    }
}
