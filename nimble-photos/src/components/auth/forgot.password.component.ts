import { Component, signal, inject } from '@angular/core';

import { RouterModule, Router } from '@angular/router';
import { FormsModule, ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { SvgComponent } from '../svg/svg.component';

@Component({
    selector: 'mtx-forgot-password',
    imports: [RouterModule, FormsModule, ReactiveFormsModule, SvgComponent],
    templateUrl: './forgot.password.component.html',
    host: {
        class: 'flex flex-1 items-center justify-center p-6 bg-slate-950/40 relative overflow-hidden'
    }
})
export class ForgotPasswordComponent {
    private readonly fb = inject(FormBuilder);
    private readonly router = inject(Router);

    readonly loading = signal(false);
    readonly submitted = signal(false);

    forgotForm = this.fb.group({
        email: ['', [Validators.required, Validators.email]]
    });

    constructor() { }

    onSubmit(): void {
        if (this.forgotForm.valid) {
            this.loading.set(true);
            // Simulate API call
            setTimeout(() => {
                this.loading.set(false);
                this.submitted.set(true);
            }, 1500);
        }
    }
}
