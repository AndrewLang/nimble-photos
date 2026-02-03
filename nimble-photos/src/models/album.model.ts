export interface AlbumModel {
    id: string;
    parentId?: string | null;
    name: string;
    createDate?: string | null;
    description?: string | null;
    category?: string | null;
    kind?: string | null;
    rulesJson?: string | null;
    thumbnailHash?: string | null;
    sortOrder?: number | null;
    imageCount?: number | null;
}
