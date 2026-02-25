import { computed, signal } from '@angular/core';
import { EMPTY, Observable, Subscription } from 'rxjs';
import { catchError, finalize, first } from 'rxjs/operators';

export class AsyncLoader<T> {

    readonly value = signal<T | null>(null);
    readonly loading = signal(false);
    readonly error = signal<string | null>(null);

    readonly hasValue = computed(() => this.value() !== null);
    readonly hasError = computed(() => !!this.error());

    private currentSub?: Subscription;

    constructor(initial?: T) {
        if (initial !== undefined) {
            this.value.set(initial);
        }
    }

    load(factory: () => Observable<T>,
        afterLoad?: (result: T) => void,
        errorMsg = 'Failed to load'): void {
        this.currentSub?.unsubscribe();

        this.loading.set(true);
        this.error.set(null);

        this.currentSub = factory().pipe(
            first(),
            catchError(err => {
                console.error(err);
                this.error.set(errorMsg);
                return EMPTY;
            }),
            finalize(() => this.loading.set(false))
        ).subscribe(result => {
            this.value.set(result);
            afterLoad?.(result);
        });
    }

    set(value: T): void {
        this.value.set(value);
    }

    clear(): void {
        this.value.set(null);
        this.error.set(null);
    }

    reset(initial?: T): void {
        this.currentSub?.unsubscribe();
        this.loading.set(false);
        this.error.set(null);
        this.value.set(initial ?? null);
    }
}