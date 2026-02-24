import { Component, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { forkJoin } from 'rxjs';
import { SettingNames } from '../../../models/setting.names';
import { SettingsService } from '../../../services/settings.service';
import { LogoEditorComponent } from '../../shared/logo-editor/logo.editor.component';

@Component({
    selector: 'mtx-general-settings-step',
    imports: [ReactiveFormsModule, LogoEditorComponent],
    templateUrl: './general.step.component.html',
})
export class GeneralStepComponent {
    private readonly formBuilder = inject(FormBuilder);
    private readonly settingsService = inject(SettingsService);

    readonly loading = signal(false);
    readonly saving = signal(false);
    readonly error = signal<string | null>(null);
    readonly success = signal<string | null>(null);

    readonly settingsForm = this.formBuilder.nonNullable.group({
        title: ['', [Validators.required, Validators.minLength(2)]],
        tagline: ['', [Validators.required, Validators.minLength(2)]],
        logo: [''],
        isPublic: [true],
        allowRegistration: [true],
    });


    constructor() {
        this.loadSettings();
    }

    private loadSettings(): void {
        this.loading.set(true);
        this.error.set(null);

        forkJoin({
            title: this.settingsService.getSettingByName(SettingNames.SiteTitle),
            tagline: this.settingsService.getSettingByName(SettingNames.SiteTagline),
            logo: this.settingsService.getSettingByName(SettingNames.SiteLogo),
            isPublic: this.settingsService.getSettingByName(SettingNames.SitePublic),
            allowRegistration: this.settingsService.getSettingByName(SettingNames.SiteAllowRegistration),
        }).subscribe({
            next: (settings) => {
                this.settingsForm.patchValue({
                    title: settings.title?.value as string,
                    tagline: settings.tagline?.value as string,
                    logo: (settings.logo?.value as string) ?? '',
                    isPublic: Boolean(settings.isPublic?.value),
                    allowRegistration: Boolean(settings.allowRegistration?.value),
                });
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
            this.settingsService.updateSetting(SettingNames.SiteTitle, title.trim()),
            this.settingsService.updateSetting(SettingNames.SiteTagline, tagline.trim()),
            this.settingsService.updateSetting(SettingNames.SiteLogo, logo.trim()),
            this.settingsService.updateSetting(SettingNames.SitePublic, isPublic),
            this.settingsService.updateSetting(SettingNames.SiteAllowRegistration, allowRegistration),
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

    onLogoChanged(path: string): void {
        this.settingsForm.get('logo')?.setValue(path);
    }
}
