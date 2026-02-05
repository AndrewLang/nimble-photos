import { Component, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { AuthService } from '../../../services/auth.service';

@Component({
    selector: 'mtx-setup-user-step',
    imports: [ReactiveFormsModule],
    templateUrl: './user.step.component.html',
})
export class UserStepComponent {
    private readonly fb = inject(FormBuilder);
    private readonly authService = inject(AuthService);

    readonly userForm = this.fb.nonNullable.group({
        name: ['', [Validators.required, Validators.minLength(2)]],
        email: ['', [Validators.required, Validators.email]],
        password: ['', [Validators.required, Validators.minLength(8)]],
    });

    readonly loading = signal(false);
    readonly error = signal<string | null>(null);
    readonly success = signal<string | null>(null);

    onSubmit(): void {
        if (this.userForm.invalid || this.loading()) {
            this.userForm.markAllAsTouched();
            return;
        }

        this.loading.set(true);
        this.error.set(null);
        this.success.set(null);

        const { name, email, password } = this.userForm.getRawValue();

        this.authService
            .register({
                displayName: name,
                email,
                password,
                confirmPassword: password,
            })
            .subscribe({
                next: () => {
                    this.loading.set(false);
                    this.success.set('Admin user created. Continue to the next step.');
                },
                error: (err) => {
                    this.loading.set(false);
                    this.error.set(err.error?.message || 'Failed to create admin user. Please try again.');
                },
            });
    }
}
