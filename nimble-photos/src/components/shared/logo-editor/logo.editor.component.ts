import { Component, effect, input, output, signal, inject } from '@angular/core';
import { SettingsService } from '../../../services/settings.service';

@Component({
    selector: 'mtx-logo-editor',
    templateUrl: './logo.editor.component.html',
})
export class LogoEditorComponent {
    private readonly settingsService = inject(SettingsService);

    readonly logoPreview = signal<string | null>(null);
    readonly logoUploading = signal(false);
    readonly isDragOver = signal(false);
    readonly error = signal<string | null>(null);

    private currentPath = '';

    readonly disabled = input(false);
    readonly showHelpText = input(true);
    readonly logoPath = input<string | null | undefined>(undefined);

    private readonly logoPathEffect = effect(() => {
        const value = this.logoPath();
        this.currentPath = value ?? '';
        const url = this.settingsService.buildLogoUrl(this.currentPath);
        this.logoPreview.set(url ? url : null);
    });

    readonly logoChanged = output<string>();

    onLogoSelected(event: Event): void {
        const input = event.target as HTMLInputElement;
        const file = input.files?.[0];
        this.handleLogoFile(file);
        input.value = '';
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
        this.error.set(null);
        this.logoPreview.set(null);
        this.currentPath = '';
        this.logoChanged.emit('');
    }

    private handleLogoFile(file?: File): void {
        if (!file || this.logoUploading() || this.disabled()) {
            return;
        }

        if (!file.type.startsWith('image/')) {
            this.error.set('Please select an image file.');
            return;
        }

        const reader = new FileReader();
        reader.onload = () => {
            const result = typeof reader.result === 'string' ? reader.result : null;
            if (!result) {
                return;
            }

            this.error.set(null);
            this.logoPreview.set(result);
            this.logoUploading.set(true);
            this.settingsService.uploadLogo(result).subscribe({
                next: (setting) => {
                    this.logoUploading.set(false);
                    if (typeof setting.value === 'string') {
                        this.currentPath = setting.value;
                        this.logoPreview.set(this.settingsService.buildLogoUrl(setting.value));
                        this.logoChanged.emit(setting.value);
                    }
                },
                error: (err) => {
                    this.logoUploading.set(false);
                    this.error.set(err.error?.message || 'Failed to upload logo.');
                },
            });
        };
        reader.readAsDataURL(file);
    }
}
