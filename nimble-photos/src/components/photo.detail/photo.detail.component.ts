import { CommonModule, Location } from '@angular/common';
import { Component, HostListener, OnInit, signal } from '@angular/core';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
import { first } from 'rxjs';
import { Photo } from '../../models/photo';
import { PhotoService } from '../../services/photo.service';

@Component({
    selector: 'mtx-photo-detail',
    imports: [CommonModule, RouterModule],
    templateUrl: './photo.detail.component.html',
    host: {
        class: 'fixed inset-0 z-[100] bg-slate-950 flex flex-col',
    }
})
export class PhotoDetailComponent implements OnInit {
    readonly photo = signal<Photo | null>(null);
    readonly loading = signal(false);
    readonly adjacents = signal<{ prevId: string | null; nextId: string | null }>({ prevId: null, nextId: null });
    readonly reactions = signal<{ emoji: string; count: number; selected: boolean }[]>([
        { emoji: '❤️', count: 12, selected: false },
        { emoji: '🔥', count: 8, selected: false },
        { emoji: '👏', count: 5, selected: false },
        { emoji: '😮', count: 2, selected: false },
        { emoji: '✨', count: 4, selected: false },
    ]);

    private albumId: string | null = null;

    constructor(
        private readonly route: ActivatedRoute,
        private readonly router: Router,
        private readonly location: Location,
        private readonly photoService: PhotoService
    ) { }

    ngOnInit(): void {
        this.route.paramMap.subscribe(params => {
            const id = params.get('id');
            this.albumId = params.get('albumId');
            if (id) {
                this.fetchPhoto(id);
            }
        });
    }

    @HostListener('window:keydown', ['$event'])
    handleKeyDown(event: KeyboardEvent): void {
        if (event.key === 'ArrowRight' && this.adjacents().nextId) {
            this.navigateToPhoto(this.adjacents().nextId!);
        } else if (event.key === 'ArrowLeft' && this.adjacents().prevId) {
            this.navigateToPhoto(this.adjacents().prevId!);
        } else if (event.key === 'Escape') {
            this.close();
        }
    }

    private fetchPhoto(id: string): void {
        this.loading.set(true);
        this.photoService.getPhotoById(id).pipe(first()).subscribe(result => {
            this.photo.set(result);
            if (result) {
                this.fetchAdjacents(result.id);
                this.loadPhotoMetadata(result.id);
            }
            this.loading.set(false);
        });
    }

    private fetchAdjacents(id: string): void {
        this.photoService.getAdjacentPhotos(id, this.albumId || undefined)
            .pipe(first())
            .subscribe(adj => this.adjacents.set(adj));
    }

    private loadPhotoMetadata(photoId: string): void {
        this.photoService.getPhotoMetadata(photoId)
            .pipe(first())
            .subscribe(metadata => {
                this.photo.update(current =>
                    current ? { ...current, metadata: metadata ?? undefined } : current
                );
            });
    }

    addReaction(emoji: string): void {
        this.reactions.update(prev => prev.map(r => {
            if (r.emoji === emoji) {
                return { ...r, count: r.selected ? r.count - 1 : r.count + 1, selected: !r.selected };
            }
            return r;
        }));
    }

    navigateToPhoto(id: string): void {
        const commands = this.albumId
            ? ['/album', this.albumId, 'photo', id]
            : ['/photo', id];
        this.router.navigate(commands);
    }

    close(): void {
        const canGoBack = typeof window !== 'undefined' && window.history.length > 1;
        if (canGoBack) {
            this.location.back();
            return;
        }

        if (this.albumId) {
            this.router.navigate(['/album', this.albumId]);
        } else {
            this.router.navigate(['/']);
        }
    }

    formatBytes(size?: number): string {
        if (!size || size <= 0) {
            return 'n/a';
        }
        const units = ['B', 'KB', 'MB', 'GB'];
        let value = size;
        let index = 0;
        while (value >= 1024 && index < units.length - 1) {
            value /= 1024;
            index += 1;
        }
        return `${value.toFixed(1)} ${units[index]}`;
    }

    getPhotoPath(): string {
        return this.photoService.getThumbnailPath(this.photo()!);
    }

    metadataSections(metadata?: Photo['metadata']): { title: string; fields: { label: string; value: string }[] }[] {
        if (!metadata) {
            return [];
        }

        const record = metadata as Record<string, unknown>;
        const usedKeys = new Set<string>();

        const buildSection = (title: string, keys: string[]) => {
            const fields = keys
                .map(key => {
                    const field = this.buildMetadataField(record, key);
                    if (field) {
                        usedKeys.add(key);
                        return field;
                    }
                    return null;
                })
                .filter((field): field is { label: string; value: string } => Boolean(field));
            return { title, fields };
        };

        const sections = [
            buildSection('Summary', [
                'make',
                'model',
                'lensModel',
                'apertureValue',
                'shutterSpeedValue',
                'iso',
                'focalLength',
                'focalLengthIn35mmFilm',
                'datetimeOriginal',
            ]),
            buildSection('Exposure & Capture', [
                'exposureProgram',
                'exposureMode',
                'exposureBiasValue',
                'meteringMode',
                'lightSource',
                'flash',
                'gainControl',
                'exposureIndex',
                'brightnessValue',
                'recommendedExposureIndex',
            ]),
            buildSection('Lens & Body', [
                'lensMake',
                'lensModel',
                'lensSpecification',
                'lensSerialNumber',
                'bodySerialNumber',
            ]),
            buildSection('Image Info', [
                'imageWidth',
                'imageLength',
                'pixelXDimension',
                'pixelYDimension',
                'bitsPerSample',
                'orientation',
                'compression',
                'digitalZoomRatio',
                'resolutionUnit',
                'colorSpace',
                'width',
                'height',
            ]),
            buildSection('Timing', [
                'datetime',
                'datetimeOriginal',
                'datetimeDigitized',
                'subsecTime',
                'subsecTimeOriginal',
                'subsecTimeDigitized',
            ]),
            buildSection('GPS', [
                'gpsLatitude',
                'gpsLongitude',
                'gpsAltitude',
                'gpsLatitudeRef',
                'gpsLongitudeRef',
                'gpsAltitudeRef',
                'gpsSpeed',
                'gpsSpeedRef',
                'gpsImgDirection',
                'gpsImgDirectionRef',
            ]),
        ].filter(section => section.fields.length > 0);

        const additionalFields = Object.keys(record)
            .filter(key => !usedKeys.has(key))
            .sort()
            .map(key => {
                const field = this.buildMetadataField(record, key);
                if (field) {
                    usedKeys.add(key);
                }
                return field;
            })
            .filter((field): field is { label: string; value: string } => Boolean(field));

        if (additionalFields.length > 0) {
            sections.push({ title: 'Additional Metadata', fields: additionalFields });
        }

        return sections;
    }

    private buildMetadataField(record: Record<string, unknown>, key: string): { label: string; value: string } | null {
        const raw = record[key];
        if (raw === undefined || raw === null || raw === '') {
            return null;
        }
        const value = this.formatMetadataValue(key, raw);
        if (!value) {
            return null;
        }
        return {
            label: this.friendlyLabel(key),
            value,
        };
    }

    private friendlyLabel(key: string): string {
        const labelMap: Record<string, string> = {
            iso: 'ISO',
            fNumber: 'F Number',
            apertureValue: 'Aperture',
            shutterSpeedValue: 'Shutter Speed',
            focalLength: 'Focal Length',
            focalLengthIn35mmFilm: 'Focal Length (35mm)',
            exposureBiasValue: 'Exposure Bias',
            gpsLatitude: 'GPS Latitude',
            gpsLongitude: 'GPS Longitude',
            gpsAltitude: 'GPS Altitude',
            datetimeOriginal: 'Date Taken',
            datetimeDigitized: 'Date Digitized',
            datetime: 'Date',
            digitalZoomRatio: 'Digital Zoom',
        };
        return labelMap[key] ?? this.humanizeKey(key);
    }

    private formatMetadataValue(key: string, value: unknown): string {
        if (value === null || value === undefined) {
            return '';
        }
        if (typeof value === 'boolean') {
            return value ? 'Yes' : 'No';
        }
        if (typeof value === 'string' && value.trim() === '') {
            return '';
        }
        if (key.toLowerCase().includes('datetime') && typeof value === 'string') {
            return this.formatDateString(value);
        }
        if (key === 'apertureValue' || key === 'fNumber') {
            const num = this.formatNumber(value, 1);
            return num ? `f/${num}` : '';
        }
        if (key === 'shutterSpeedValue') {
            const num = this.formatNumber(value, 3);
            return num ? `${num}s` : '';
        }
        if (key === 'exposureBiasValue') {
            const num = this.formatNumber(value, 2);
            return num ? `${num} eV` : '';
        }
        if (key === 'focalLength' || key === 'focalLengthIn35mmFilm') {
            const num = this.formatNumber(value, 1);
            return num ? `${num} mm` : '';
        }
        if (key === 'digitalZoomRatio') {
            const num = this.formatNumber(value, 2);
            return num ? `${num}x` : '';
        }
        if (typeof value === 'number') {
            return value.toString();
        }
        return `${value}`;
    }

    private humanizeKey(key: string): string {
        return key
            .replace(/([A-Z])/g, ' $1')
            .replace(/_/g, ' ')
            .replace(/\s+/g, ' ')
            .trim()
            .replace(/\b\w/g, (char) => char.toUpperCase());
    }

    private formatNumber(value: unknown, decimals = 0): string {
        const num =
            typeof value === 'number' ? value :
            typeof value === 'string' ? Number(value) : NaN;
        if (Number.isNaN(num)) {
            return '';
        }
        return num.toFixed(decimals);
    }

    formatCoordinates(metadata?: Photo['metadata']): string | null {
        if (!metadata) {
            return null;
        }

        const lat = metadata.gpsLatitude;
        const lng = metadata.gpsLongitude;
        if (lat === undefined && lng === undefined) {
            return null;
        }

        const formatComponent = (value?: number | null, ref?: string | null, positiveLabel?: string, negativeLabel?: string) => {
            if (value === undefined || value === null) {
                return 'Unknown';
            }
            const direction = ref ?? (value >= 0 ? positiveLabel : negativeLabel);
            return `${Math.abs(value).toFixed(4)}° ${direction ?? ''}`.trim();
        };

        const latString = formatComponent(lat, metadata.gpsLatitudeRef ?? null, 'N', 'S');
        const lngString = formatComponent(lng, metadata.gpsLongitudeRef ?? null, 'E', 'W');
        return `${latString}, ${lngString}`;
    }

    formatCamera(metadata?: Photo['metadata']): string | null {
        if (!metadata) {
            return null;
        }

        const components = [metadata.make, metadata.model].filter((value): value is string => Boolean(value));
        if (components.length === 0) {
            return null;
        }

        return components.join(' ');
    }

    formatAperture(metadata?: Photo['metadata']): string | null {
        const value = metadata?.apertureValue ?? metadata?.fNumber;
        if (value === undefined || value === null) {
            return null;
        }
        return `ƒ/${value.toFixed(1)}`;
    }

    formatShutterSpeed(metadata?: Photo['metadata']): string | null {
        const value = metadata?.shutterSpeedValue;
        if (value === undefined || value === null) {
            return null;
        }
        return `${value.toFixed(2)}s`;
    }

    private formatDateString(value: string): string {
        const parsed = new Date(value);
        if (Number.isNaN(parsed.getTime())) {
            return value;
        }
        return parsed.toLocaleDateString('en-US', {
            month: 'short',
            day: 'numeric',
            year: 'numeric',
        });
    }
}
