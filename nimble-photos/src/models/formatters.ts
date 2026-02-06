export class Formatter {
    static formatBytes(bytes?: number, options?: { zeroLabel?: string }): string {
        if (!Number.isFinite(bytes) || (bytes ?? 0) <= 0) {
            return options?.zeroLabel ?? '0 B';
        }
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const index = Math.min(Math.floor(Math.log(bytes!) / Math.log(1024)), units.length - 1);
        const value = bytes! / Math.pow(1024, index);
        return `${value.toFixed(value >= 10 || index === 0 ? 0 : 1)} ${units[index]}`;
    }

    static formatAvailablePercent(availableBytes: number, totalBytes: number): string {
        if (!Number.isFinite(availableBytes) || !Number.isFinite(totalBytes) || totalBytes <= 0) {
            return '0%';
        }
        const percent = Math.max(0, Math.min(1, availableBytes / totalBytes)) * 100;
        return `${Math.round(percent)}%`;
    }
}
