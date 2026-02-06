
import { Component, HostListener, OnInit, signal } from '@angular/core';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
import { first } from 'rxjs';
import { Photo, PhotoComment } from '../../models/photo';
import { AuthService } from '../../services/auth.service';
import { PhotoService } from '../../services/photo.service';
import { SvgComponent } from '../svg/svg.component';
import { Formatter } from '../../models/formatters';

const MAX_COMMENT_LENGTH = 1024;

@Component({
    selector: 'mtx-photo-detail',
    imports: [RouterModule, SvgComponent],
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

    readonly commentDraft = signal('');
    readonly commentSaving = signal(false);
    readonly commentError = signal<string | null>(null);
    readonly maxCommentLength = MAX_COMMENT_LENGTH;
    readonly comments = signal<PhotoComment[]>([]);
    readonly commentsLoading = signal(false);
    readonly commentsError = signal<string | null>(null);
    readonly metadataExpanded = signal(false);
    readonly commentEditorVisible = signal(false);
    readonly formatBytes = (size?: number) => Formatter.formatBytes(size, { zeroLabel: 'n/a' });

    private albumId: string | null = null;
    private returnUrl = '/';

    constructor(
        private readonly route: ActivatedRoute,
        private readonly router: Router,
        public readonly authService: AuthService,
        private readonly photoService: PhotoService
    ) { }

    ngOnInit(): void {
        const initialAlbumId = this.route.snapshot.paramMap.get('albumId');
        this.albumId = initialAlbumId;
        const navigationState = this.router.getCurrentNavigation()?.extras.state as { returnUrl?: string } | undefined;
        this.returnUrl = navigationState?.returnUrl ?? this.buildDefaultReturnUrl(initialAlbumId);

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
                this.loadComments(result.id);
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
                this.metadataExpanded.set(false);
                console.log('Loaded metadata for photo', photoId, metadata);
            });
    }

    handleCommentInput(event: Event): void {
        const target = event.target as HTMLTextAreaElement;
        this.commentDraft.set(target.value.slice(0, MAX_COMMENT_LENGTH));
    }

    saveComment(): void {
        const photo = this.photo();
        if (!photo || !this.authService.isAuthenticated()) {
            return;
        }

        const trimmed = this.commentDraft().trim();
        if (trimmed.length === 0 || trimmed.length > MAX_COMMENT_LENGTH) {
            this.commentError.set(`Comment must be between 1 and ${MAX_COMMENT_LENGTH} characters.`);
            return;
        }

        this.commentSaving.set(true);
        this.commentError.set(null);

        this.photoService.createPhotoComment(photo.id, trimmed)
            .pipe(first())
            .subscribe({
                next: (comment: PhotoComment) => {
                    this.commentSaving.set(false);
                    this.comments.update(current => [comment, ...current]);
                    this.commentDraft.set('');
                },
                error: () => {
                    this.commentSaving.set(false);
                    this.commentError.set('Unable to save your comment.');
                }
            });
    }

    toggleCommentEditor(): void {
        if (!this.authService.isAuthenticated()) {
            return;
        }
        this.commentEditorVisible.update(value => !value);
    }

    toggleMetadata(): void {
        this.metadataExpanded.update(value => !value);
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
        this.router.navigate(commands, { state: { returnUrl: this.returnUrl } });
    }

    close(): void {
        const target = this.returnUrl ?? this.buildDefaultReturnUrl(this.albumId);
        this.router.navigateByUrl(target);
    }

    private buildDefaultReturnUrl(albumId: string | null): string {
        return albumId ? `/album/${albumId}` : '/';
    }

    private loadComments(photoId: string): void {
        this.commentsLoading.set(true);
        this.commentsError.set(null);
        this.photoService.getPhotoComments(photoId)
            .pipe(first())
            .subscribe({
                next: (comments: PhotoComment[]) => {
                    console.log('Loaded comments for photo', photoId, comments);
                    this.comments.set(comments);
                    this.commentsLoading.set(false);
                },
                error: () => {
                    this.comments.set([]);
                    this.commentsLoading.set(false);
                    this.commentsError.set('Failed to load comments.');
                }
            });
    }

    formatCommentDate(value?: string): string {
        if (!value) {
            return '';
        }
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

    getPhotoPath(): string {
        return this.photoService.getThumbnailPath(this.photo()!);
    }

    metadataSections(p?: Photo | null): { title: string; fields: { label: string; value: string }[] }[] {
        if (!p || !p.metadata) {
            return [];
        }

        const record = { ...p.metadata } as Record<string, unknown>;
        if (p.width) record['width'] = p.width;
        if (p.height) record['height'] = p.height;

        const buildSection = (title: string, keys: string[]) => {
            const fields = keys
                .map(key => this.buildMetadataField(record, key))
                .filter((field): field is { label: string; value: string } => Boolean(field));
            return { title, fields };
        };

        const sections = [
            buildSection('Summary', [
                'make',
                'model',
                'lensModel',
                'apertureValue',
                'exposureTime',
                'iso',
                'focalLength',
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
                'bitsPerSample',
                'orientation',
                'compression',
                'digitalZoomRatio',
                'resolutionUnit',
            ]),
            buildSection('Exposure & Capture', [
                'exposureProgram',
                'exposureMode',
                'exposureBiasValue',
                'meteringMode',
                'lightSource',
                'flash',
                'gainControl',
            ]),
            buildSection('Tone & Color', [
                'whiteBalance',
                'contrast',
                'saturation',
                'sharpness',
                'gamma',
            ]),
            buildSection('Timing', [
                'datetimeOriginal',
                'datetimeDigitized',
                'datetime',
            ]),
            buildSection('Gps', [
                'gpsLatitude',
                'gpsLongitude',
                'gpsAltitude',
                'gpsSpeed',
            ]),
            buildSection('Misc', [
                'software',
                'artist',
                'copyright',
                'userComment',
            ]),
        ].filter(section => section.fields.length > 0);

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
            make: 'Make',
            model: 'Model',
            lensModel: 'Lens',
            lensMake: 'Lens make',
            lensSpecification: 'Lens specific',
            lensSerialNumber: 'Lens serial',
            bodySerialNumber: 'Body serial',
            iso: 'ISO',
            fNumber: 'Aperture',
            apertureValue: 'Aperture',
            shutterSpeedValue: 'Shutter speed',
            exposureTime: 'Exposure time',
            exposureProgram: 'Program',
            exposureMode: 'Mode',
            exposureBiasValue: 'Exposure bias',
            focalLength: 'Focal length',
            meteringMode: 'Metering',
            lightSource: 'Light source',
            flash: 'Flash',
            gainControl: 'Gain control',
            width: 'Width',
            height: 'Height',
            bitsPerSample: 'Bits per sample',
            orientation: 'Orientation',
            compression: 'Compression',
            digitalZoomRatio: 'Digital zoom',
            resolutionUnit: 'Resolution unit',
            colorSpace: 'Color space',
            whiteBalance: 'White balance',
            contrast: 'Contrast',
            saturation: 'Saturation',
            sharpness: 'Sharpness',
            gamma: 'Gamma',
            datetimeOriginal: 'Taken',
            datetimeDigitized: 'Digitized',
            datetime: 'Edited',
            gpsLatitude: 'Latitude',
            gpsLongitude: 'Longitude',
            gpsAltitude: 'Altitude',
            gpsSpeed: 'Speed',
            software: 'Software',
            artist: 'Artist',
            copyright: 'Copyright',
            userComment: 'User comments',
            imageLength: 'Height',
            imageWidth: 'Width',
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
