import { Component, inject, OnInit, signal } from '@angular/core';

import { FormBuilder, FormsModule, ReactiveFormsModule, Validators } from '@angular/forms';
import { Router, RouterModule } from '@angular/router';
import { catchError, of } from 'rxjs';
import { AuthService } from '../../services/auth.service';
import { SettingsService } from '../../services/settings.service';
import { SvgComponent } from '../svg/svg.component';

@Component({
    selector: 'mtx-register',
    imports: [RouterModule, FormsModule, ReactiveFormsModule, SvgComponent],
    templateUrl: './register.component.html',
    host: {
        class: 'flex flex-1 items-center justify-center p-6 bg-slate-950/40 relative overflow-hidden'
    }
})
export class RegisterComponent implements OnInit {
    private readonly fb = inject(FormBuilder);
    private readonly router = inject(Router);
    private readonly authService = inject(AuthService);
    private readonly settingsService = inject(SettingsService);

    readonly loading = signal(false);
    readonly error = signal<string | null>(null);
    readonly logoUrl = signal<string | null>(null);

    registerForm = this.fb.group({
        displayName: ['', [Validators.required, Validators.minLength(2)]],
        email: ['', [Validators.required, Validators.email]],
        password: ['', [Validators.required, Validators.minLength(6)]],
        confirmPassword: ['', [Validators.required]],
    }, {
        validators: this.passwordMatchValidator
    });

    constructor() { }

    ngOnInit(): void {
        this.settingsService.getLogoUrl()
            .pipe(catchError(() => of(null)))
            .subscribe(url => this.logoUrl.set(url));
    }

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
