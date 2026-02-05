import { Component, signal, inject } from '@angular/core';

import { RouterModule, Router } from '@angular/router';
import { FormsModule, ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { AuthService } from '../../services/auth.service';
import { SvgComponent } from '../svg/svg.component';

@Component({
    selector: 'mtx-register',
    standalone: true,
    imports: [RouterModule, FormsModule, ReactiveFormsModule, SvgComponent],
    templateUrl: './register.component.html',
    host: {
        class: 'flex flex-1 items-center justify-center p-6 bg-slate-950/40 relative overflow-hidden'
    }
})
export class RegisterComponent {
    private readonly fb = inject(FormBuilder);
    private readonly router = inject(Router);
    private readonly authService = inject(AuthService);

    readonly loading = signal(false);
    readonly error = signal<string | null>(null);

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
            this.error.set(null);

            const { displayName, email, password, confirmPassword } = this.registerForm.value;

            this.authService.register({
                displayName: displayName!,
                email: email!,
                password: password!,
                confirmPassword: confirmPassword!
            }).subscribe({
                next: () => {
                    this.loading.set(false);
                    this.router.navigate(['/login'], { queryParams: { registered: true } });
                },
                error: (err) => {
                    this.loading.set(false);
                    this.error.set(err.error?.message || 'Registration failed. Please try again.');
                    console.error('Registration error:', err);
                }
            });
        }
    }
}
