import { Component, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { forkJoin } from 'rxjs';
import { SettingsService } from '../../../services/settings.service';

@Component({
    selector: 'mtx-general-settings-step',
    imports: [ReactiveFormsModule],
    templateUrl: './general.step.component.html',
})
export class GeneralStepComponent {
    private readonly fb = inject(FormBuilder);
    private readonly settingsService = inject(SettingsService);

    readonly loading = signal(false);
    readonly saving = signal(false);
    readonly error = signal<string | null>(null);
    readonly success = signal<string | null>(null);
    readonly logoUploading = signal(false);
    readonly isDragOver = signal(false);

    readonly settingsForm = this.fb.nonNullable.group({
        title: ['', [Validators.required, Validators.minLength(2)]],
        tagline: ['', [Validators.required, Validators.minLength(2)]],
        logo: [''],
        isPublic: [true],
        allowRegistration: [true],
    });

    readonly logoPreview = signal<string | null>(null);

    constructor() {
        this.loadSettings();
    }

    private loadSettings(): void {
        this.loading.set(true);
        this.error.set(null);

        forkJoin({
            title: this.settingsService.getSettingByName('site.title'),
            tagline: this.settingsService.getSettingByName('site.tagline'),
            logo: this.settingsService.getSettingByName('site.logo'),
            isPublic: this.settingsService.getSettingByName('site.public'),
            allowRegistration: this.settingsService.getSettingByName('site.allowRegistration'),
        }).subscribe({
            next: (settings) => {
                this.settingsForm.patchValue({
                    title: settings.title.value as string,
                    tagline: settings.tagline.value as string,
                    logo: (settings.logo.value as string) ?? '',
                    isPublic: Boolean(settings.isPublic.value),
                    allowRegistration: Boolean(settings.allowRegistration.value),
                });
                console.log('Loaded Settings:', settings);
                const logo = this.settingsService.buildLogoUrl(this.settingsForm.get('logo')?.value ?? '');
                this.logoPreview.set(logo ? logo : null);
                this.loading.set(false);
            },
            error: (err) => {
                this.loading.set(false);
                this.error.set(err.error?.message || 'Failed to load settings.');
            },
        });
    }

    onSubmit(): void {
        if (this.settingsForm.invalid || this.saving()) {
            this.settingsForm.markAllAsTouched();
            return;
        }

        this.saving.set(true);
        this.error.set(null);
        this.success.set(null);

        const { title, tagline, logo, isPublic, allowRegistration } = this.settingsForm.getRawValue();

        forkJoin([
            this.settingsService.updateSetting('site.title', title.trim()),
            this.settingsService.updateSetting('site.tagline', tagline.trim()),
            this.settingsService.updateSetting('site.logo', logo.trim()),
            this.settingsService.updateSetting('site.public', isPublic),
            this.settingsService.updateSetting('site.allowRegistration', allowRegistration),
        ]).subscribe({
            next: () => {
                this.saving.set(false);
                this.success.set('General settings saved.');
            },
            error: (err) => {
                this.saving.set(false);
                this.error.set(err.error?.message || 'Failed to save settings.');
            },
        });
    }

    onLogoSelected(event: Event): void {
        const input = event.target as HTMLInputElement;
        const file = input.files?.[0];
        this.handleLogoFile(file);
    }

    onLogoDropped(event: DragEvent): void {
        event.preventDefault();
        this.isDragOver.set(false);
        const file = event.dataTransfer?.files?.[0];
        this.handleLogoFile(file);
    }

    onLogoDragOver(event: DragEvent): void {
        event.preventDefault();
        if (!this.logoUploading()) {
            this.isDragOver.set(true);
        }
    }

    onLogoDragLeave(): void {
        this.isDragOver.set(false);
    }

    clearLogo(): void {
        this.logoPreview.set(null);
        this.settingsForm.get('logo')?.setValue('');
    }

    private handleLogoFile(file?: File): void {
        if (!file || this.logoUploading()) {
            return;
        }

        if (!file.type.startsWith('image/')) {
            this.error.set('Please select an image file.');
            return;
        }

        const reader = new FileReader();
        reader.onload = () => {
            const result = typeof reader.result === 'string' ? reader.result : null;
            if (result) {
                this.logoPreview.set(result);
                this.logoUploading.set(true);
                this.settingsService.uploadLogo(result).subscribe({
                    next: (setting) => {
                        this.logoUploading.set(false);
                        if (typeof setting.value === 'string') {
                            this.settingsForm.get('logo')?.setValue(setting.value);
                            this.logoPreview.set(setting.value);
                        }
                    },
                    error: (err) => {
                        this.logoUploading.set(false);
                        this.error.set(err.error?.message || 'Failed to upload logo.');
                    },
                });
            }
        };
        reader.readAsDataURL(file);
    }
}
