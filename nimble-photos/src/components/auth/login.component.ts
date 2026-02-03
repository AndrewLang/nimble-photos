import { Component, signal, inject, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, Router, ActivatedRoute } from '@angular/router';
import { FormsModule, ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { AuthService } from '../../services/auth.service';

@Component({
    selector: 'mtx-login',
    standalone: true,
    imports: [CommonModule, RouterModule, FormsModule, ReactiveFormsModule],
    templateUrl: './login.component.html',
    host: {
        class: 'flex flex-1 items-center justify-center p-6 bg-slate-950/40 relative overflow-hidden'
    }
})
export class LoginComponent implements OnInit {
    private readonly fb = inject(FormBuilder);
    private readonly router = inject(Router);
    private readonly route = inject(ActivatedRoute);
    private readonly authService = inject(AuthService);

    readonly loading = signal(false);
    readonly error = signal<string | null>(null);
    readonly success = signal<string | null>(null);
    readonly showPassword = signal(false);

    loginForm = this.fb.group({
        email: ['', [Validators.required, Validators.email]],
        password: ['', [Validators.required, Validators.minLength(6)]],
        rememberMe: [false]
    });

    ngOnInit(): void {
        this.route.queryParams.subscribe(params => {
            if (params['registered']) {
                this.success.set('Registration successful! Please sign in with your new account.');
            }
        });
    }

    constructor() { }

    onSubmit(): void {
        if (this.loginForm.valid) {
            this.loading.set(true);
            this.error.set(null);

            const { email, password } = this.loginForm.value;

            this.authService.login({ email: email!, password: password! }).subscribe({
                next: () => {
                    this.loading.set(false);
                    this.router.navigate(['/']);
                },
                error: (err) => {
                    this.loading.set(false);
                    this.error.set(err.error?.message || 'Login failed. Please check your credentials.');
                    console.error('Login error:', err);
                }
            });
        }
    }

    togglePassword(): void {
        this.showPassword.update(v => !v);
    }
}
