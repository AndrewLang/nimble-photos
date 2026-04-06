import { logger } from "./logger";
import { Photo } from "./photo";

export class PhotoMetadataProcessor {
    buildMetadataSections(p?: Photo | null): { title: string; fields: { label: string; value: string }[] }[] {
        logger.debug('Building metadata sections for photo', p);
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

    buildMetadataField(record: Record<string, unknown>, key: string): { label: string; value: string } | null {
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

    formatMetadataValue(key: string, value: unknown): string {
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

    formatNumber(value: unknown, decimals = 0): string {
        const num =
            typeof value === 'number' ? value :
                typeof value === 'string' ? Number(value) : NaN;
        if (Number.isNaN(num)) {
            return '';
        }
        return num.toFixed(decimals);
    }

    formatDateString(value: string): string {
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

    private humanizeKey(key: string): string {
        return key
            .replace(/([A-Z])/g, ' $1')
            .replace(/_/g, ' ')
            .replace(/\s+/g, ' ')
            .trim()
            .replace(/\b\w/g, (char) => char.toUpperCase());
    }

}