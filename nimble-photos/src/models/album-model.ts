export interface AlbumModel {
    id: string;
    parent_id?: string | null;
    name: string;
    create_date?: string | null;
    description?: string | null;
    category?: string | null;
    kind?: string | null;
    rules_json?: string | null;
    thumbnail_hash?: string | null;
    sort_order?: number | null;
    image_count?: number | null;
}
