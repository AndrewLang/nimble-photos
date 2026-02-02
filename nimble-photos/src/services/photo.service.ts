import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { catchError, map, Observable, of, switchMap } from 'rxjs';

import { Album, GroupedPhotos, PagedPhotos, Photo } from '../models/photo.model';

interface PhotoDto {
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

interface AlbumDto {
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

interface PagedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

@Injectable({
  providedIn: 'root',
})
export class PhotoService {
  private readonly apiBase = 'http://localhost:8080/api';

  constructor(private readonly http: HttpClient) { }

  getPhotos(page = 1, pageSize = 56): Observable<PagedPhotos> {
    return this.http
      .get<PagedResponse<PhotoDto>>(`${this.apiBase}/photos/${page}/${pageSize}`)
      .pipe(map((response) => this.mapPhotoPage(response)));
  }

  getPhotoById(id: string): Observable<Photo | null> {
    return this.http.get<PhotoDto>(`${this.apiBase}/photos/${id}`).pipe(
      map((dto) => this.mapPhoto(dto)),
      catchError(() => of(null))
    );
  }

  getThumbnailPath(photo: Photo): string {
    if (photo.hash) {
      return `${this.apiBase}/photos/thumbnail/${photo.hash}`;
    }

    return photo.path;
  }

  getAdjacentPhotos(id: string, _albumId?: string): Observable<{ prevId: string | null; nextId: string | null }> {
    return this.getPhotos(1, 400).pipe(
      map((page) => {
        const index = page.items.findIndex((photo) => photo.id === id);
        const prevId = index > 0 ? page.items[index - 1].id : null;
        const nextId = index >= 0 && index < page.items.length - 1 ? page.items[index + 1].id : null;
        return { prevId, nextId };
      })
    );
  }

  getTimeline(): Observable<GroupedPhotos[]> {
    return this.http
      .get<{ title: string; photos: PagedResponse<PhotoDto> }[]>(`${this.apiBase}/photos/timeline`)
      .pipe(
        map((groups) =>
          groups.map((g) => ({
            title: g.title,
            photos: this.mapPhotoPage(g.photos),
          }))
        ),
        catchError(() => of([]))
      );
  }

  getGroupedPhotos(
    groupIndex: number,
    pageInGroup: number = 1,
    pageSize: number = 100
  ): Observable<GroupedPhotos | null> {
    return this.getTimeline().pipe(
      map((groups) => {
        if (groupIndex >= groups.length) {
          return null;
        }
        const target = groups[groupIndex];
        const start = (pageInGroup - 1) * pageSize;
        const items = target.photos.items.slice(start, start + pageSize);
        return {
          title: target.title,
          photos: {
            ...target.photos,
            page: pageInGroup,
            pageSize,
            items,
          },
        };
      })
    );
  }

  getAlbums(page = 1, pageSize = 12): Observable<PagedAlbums> {
    return this.http
      .get<PagedResponse<AlbumDto>>(`${this.apiBase}/albums/${page}/${pageSize}`)
      .pipe(
        map((response) => ({
          page: response.page,
          pageSize: response.page_size,
          total: response.total,
          items: response.items.map((dto) => this.mapAlbum(dto)),
        }))
      );
  }

  getAlbumById(id: string): Observable<Album | null> {
    return this.http.get<AlbumDto>(`${this.apiBase}/albums/${id}`).pipe(
      switchMap((dto) =>
        this.getPhotos(1, 20).pipe(
          map((photos) => ({
            ...this.mapAlbum(dto),
            photos,
          }))
        )
      ),
      catchError(() => of(null))
    );
  }

  getAlbumPhotos(_albumId: string, page = 1, pageSize = 20): Observable<PagedPhotos | null> {
    return this.getPhotos(page, pageSize);
  }

  private mapPhotoPage(response: PagedResponse<PhotoDto>): PagedPhotos {
    console.log('Mapping photo page', response);
    return {
      page: response.page,
      pageSize: response.page_size,
      total: response.total,
      items: response.items.map((dto) => this.mapPhoto(dto)),
    };
  }

  private mapPhoto(dto: PhotoDto): Photo {
    return {
      id: dto.id,
      path: dto.path,
      name: dto.name,
      format: dto.format ?? undefined,
      hash: dto.hash ?? undefined,
      size: dto.size ?? undefined,
      createdAt: this.toDate(dto.created_at),
      updatedAt: this.toDate(dto.updated_at),
      dateImported: this.toDate(dto.date_imported),
      dateTaken: this.toDate(dto.date_taken),
      thumbnailPath: dto.thumbnail_path ?? undefined,
      thumbnailOptimized: dto.thumbnail_optimized ?? undefined,
      metadataExtracted: dto.metadata_extracted ?? undefined,
      isRaw: dto.is_raw ?? undefined,
      width: dto.width ?? undefined,
      height: dto.height ?? undefined,
      thumbnailWidth: dto.thumbnail_width ?? undefined,
      thumbnailHeight: dto.thumbnail_height ?? undefined,
    };
  }

  private mapAlbum(dto: AlbumDto): Album {
    return {
      id: dto.id,
      parentId: dto.parent_id ?? undefined,
      name: dto.name,
      createDate: this.toDate(dto.create_date),
      description: dto.description ?? undefined,
      category: dto.category ?? undefined,
      kind: (dto.kind === 'smart' ? 'smart' : 'manual') as Album['kind'],
      rulesJson: dto.rules_json ?? undefined,
      thumbnailHash: dto.thumbnail_hash ?? undefined,
      sortOrder: dto.sort_order ?? 0,
      imageCount: dto.image_count ?? undefined,
    };
  }


  private toDate(value?: string | null): Date | undefined {
    if (!value) {
      return undefined;
    }
    const parsed = new Date(value);
    return Number.isNaN(parsed.getTime()) ? undefined : parsed;
  }
}

export interface PagedAlbums {
  page: number;
  pageSize: number;
  total: number;
  items: Album[];
}
