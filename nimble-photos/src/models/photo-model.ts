export interface PhotoModel {
    id: string;
    path: string;
    name: string;
    format?: string | null;
    hash?: string | null;
    size?: number | null;
    created_at?: string | null;
    updated_at?: string | null;
    date_imported?: string | null;
    date_taken?: string | null;
    thumbnail_path?: string | null;
    thumbnail_optimized?: boolean | null;
    metadata_extracted?: boolean | null;
    is_raw?: boolean | null;
    width?: number | null;
    height?: number | null;
    thumbnail_width?: number | null;
    thumbnail_height?: number | null;
}
