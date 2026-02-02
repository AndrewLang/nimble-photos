import { Component, signal, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, Router } from '@angular/router';
import { FormsModule, ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';

@Component({
    selector: 'mtx-login',
    imports: [CommonModule, RouterModule, FormsModule, ReactiveFormsModule],
    templateUrl: './login.component.html',
    host: {
        class: 'flex flex-1 items-center justify-center p-6 bg-slate-950/40 relative overflow-hidden'
    }
})
export class LoginComponent {
    private readonly fb = inject(FormBuilder);
    private readonly router = inject(Router);

    readonly loading = signal(false);
    readonly showPassword = signal(false);

    loginForm = this.fb.group({
        email: ['', [Validators.required, Validators.email]],
        password: ['', [Validators.required, Validators.minLength(6)]],
        rememberMe: [false]
    });

    constructor() { }

    onSubmit(): void {
        if (this.loginForm.valid) {
            this.loading.set(true);
            // Simulate API call
            setTimeout(() => {
                this.loading.set(false);
                this.router.navigate(['/']);
            }, 1500);
        }
    }

    togglePassword(): void {
        this.showPassword.update(v => !v);
    }
}
