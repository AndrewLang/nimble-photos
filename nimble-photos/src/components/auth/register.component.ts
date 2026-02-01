import { Component, signal, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, Router } from '@angular/router';
import { FormsModule, ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';

@Component({
    selector: 'mtx-register',
    standalone: true,
    imports: [CommonModule, RouterModule, FormsModule, ReactiveFormsModule],
    templateUrl: './register.component.html',
    host: {
        class: 'flex flex-1 items-center justify-center p-6 bg-slate-950/40 relative overflow-hidden'
    }
})
export class RegisterComponent {
    private readonly fb = inject(FormBuilder);
    private readonly router = inject(Router);

    readonly loading = signal(false);

    registerForm = this.fb.group({
        displayName: ['', [Validators.required, Validators.minLength(2)]],
        email: ['', [Validators.required, Validators.email]],
        password: ['', [Validators.required, Validators.minLength(6)]],
        confirmPassword: ['', [Validators.required]],
        terms: [false, [Validators.requiredTrue]]
    }, {
        validators: this.passwordMatchValidator
    });

    constructor() { }

    passwordMatchValidator(g: any) {
        return g.get('password').value === g.get('confirmPassword').value
            ? null : { mismatch: true };
    }

    onSubmit(): void {
        if (this.registerForm.valid) {
            this.loading.set(true);
            // Simulate API call
            setTimeout(() => {
                this.loading.set(false);
                this.router.navigate(['/']);
            }, 1500);
        }
    }
}
