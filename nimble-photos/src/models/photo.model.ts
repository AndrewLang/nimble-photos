export interface PhotoModel {
    id: string;
    path: string;
    name: string;
    format?: string | null;
    hash?: string | null;
    size?: number | null;
    createdAt?: string | null;
    updatedAt?: string | null;
    dateImported?: string | null;
    dateTaken?: string | null;
    thumbnailPath?: string | null;
    thumbnailOptimized?: boolean | null;
    metadataExtracted?: boolean | null;
    isRaw?: boolean | null;
    width?: number | null;
    height?: number | null;
    thumbnailWidth?: number | null;
    thumbnailHeight?: number | null;
}
