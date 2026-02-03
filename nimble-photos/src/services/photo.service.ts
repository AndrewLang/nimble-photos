import { HttpClient } from '@angular/common/http';
import { Injectable, signal } from '@angular/core';
import { catchError, map, Observable, of, switchMap, tap } from 'rxjs';

import { Album, GroupedPhotos, PagedPhotos, Photo } from '../models/photo.model';
import { PhotoModel } from '../models/photo-model';
import { AlbumModel } from '../models/album-model';
import { PagedResponseModel } from '../models/paged-response-model';
import { PagedAlbumsModel } from '../models/paged-albums-model';


@Injectable({
  providedIn: 'root',
})
export class PhotoService {
  private readonly apiBase = 'http://localhost:8080/api';
  private timelinePhotoIds: string[] | null = null;
  public timelineCache: GroupedPhotos[] | null = null;

  lastGalleryScrollIndex = 0;
  readonly isScrolled = signal(false);

  constructor(private readonly http: HttpClient) { }

  getPhotos(page = 1, pageSize = 56): Observable<PagedPhotos> {
    return this.http
      .get<PagedResponseModel<PhotoModel>>(`${this.apiBase}/photos/${page}/${pageSize}`)
      .pipe(map((response) => this.mapPhotoPage(response)));
  }

  getPhotoById(id: string): Observable<Photo | null> {
    return this.http.get<PhotoModel>(`${this.apiBase}/photos/${id}`).pipe(
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

  getDownloadPath(photo: Photo): string {
    return `${this.apiBase}/photos/file/${photo.id}`;
  }

  getAdjacentPhotos(id: string, albumId?: string): Observable<{ prevId: string | null; nextId: string | null }> {
    if (albumId) {
      return this.getAlbumPhotos(albumId, 1, 400).pipe(
        map((page) => {
          if (!page) return { prevId: null, nextId: null };
          const index = page.items.findIndex((photo) => photo.id === id);
          return this.resolveAdjacent(page.items.map(p => p.id), index);
        })
      );
    }

    let source$: Observable<string[]>;
    if (this.timelinePhotoIds) {
      source$ = of(this.timelinePhotoIds);
    } else {
      source$ = this.getTimeline().pipe(
        map(() => this.timelinePhotoIds || [])
      );
    }

    return source$.pipe(
      map(ids => {
        const index = ids.indexOf(id);
        return this.resolveAdjacent(ids, index);
      })
    );
  }

  private resolveAdjacent(ids: string[], index: number) {
    if (index === -1) return { prevId: null, nextId: null };
    const prevId = index > 0 ? ids[index - 1] : null;
    const nextId = index >= 0 && index < ids.length - 1 ? ids[index + 1] : null;
    return { prevId, nextId };
  }

  getTimeline(page: number = 1, pageSize: number = 10): Observable<GroupedPhotos[]> {
    if (page === 1 && this.timelineCache) {
      return of(this.timelineCache);
    }

    return this.http
      .get<{ title: string; photos: PagedResponseModel<PhotoModel> }[]>(`${this.apiBase}/photos/timeline/${page}/${pageSize}`)
      .pipe(
        map((groups) =>
          groups.map((g) => ({
            title: g.title,
            photos: this.mapPhotoPage(g.photos),
          }))
        ),
        tap(groups => {
          if (page === 1) {
            this.timelineCache = groups;
            this.timelinePhotoIds = groups.flatMap(g => g.photos.items.map(p => p.id));
          } else {
            if (this.timelineCache) {
              this.timelineCache.push(...groups);
            }
            if (this.timelinePhotoIds) {
              this.timelinePhotoIds.push(...groups.flatMap(g => g.photos.items.map(p => p.id)));
            }
          }
        }),
        catchError(() => of([]))
      );
  }

  getTimelineYears(): Observable<string[]> {
    return this.http.get<string[]>(`${this.apiBase}/photos/timeline/years`).pipe(
      catchError(() => of([]))
    );
  }

  getTimelineYearOffset(year: string): Observable<number> {
    return this.http.get<number>(`${this.apiBase}/photos/timeline/year-offset/${year}`).pipe(
      catchError(() => of(0))
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

  getAlbums(page = 1, pageSize = 12): Observable<PagedAlbumsModel> {
    return this.http
      .get<PagedResponseModel<AlbumModel>>(`${this.apiBase}/albums/${page}/${pageSize}`)
      .pipe(
        map((response) => ({
          page: response.page,
          pageSize: response.page_size,
          total: response.total,
          items: response.items.map((dto) => this.mapAlbum(dto)),
        }))
      );
  }

  createAlbum(album: Partial<AlbumModel>): Observable<Album> {
    return this.http.post<AlbumModel>(`${this.apiBase}/albums`, album).pipe(
      map(dto => this.mapAlbum(dto))
    );
  }

  getAlbumById(id: string): Observable<Album | null> {
    return this.http.get<AlbumModel>(`${this.apiBase}/albums/${id}`).pipe(
      switchMap((dto) => {
        console.log('Album: ', dto);
        const rules = JSON.parse(dto.rules_json || '{ "photoIds": [] }');
        console.log('Rules: ', rules);

        return this.getAlbumPhotos(id, 1, 100).pipe(
          map((photos) => ({
            ...this.mapAlbum(dto),
            photos: photos || { page: 1, pageSize: 100, total: 0, items: [] },
          }))
        );
      }
      ),
      catchError(() => of(null))
    );
  }

  getAlbumPhotos(albumId: string, page = 1, pageSize = 20): Observable<PagedPhotos | null> {
    return this.http
      .get<PagedResponseModel<PhotoModel>>(`${this.apiBase}/albums/${albumId}/photos/${page}/${pageSize}`)
      .pipe(
        map((response) => this.mapPhotoPage(response)),
        catchError(() => of(null))
      );
  }

  private mapPhotoPage(response: PagedResponseModel<PhotoModel>): PagedPhotos {
    console.log('Mapping photo page', response);
    return {
      page: response.page,
      pageSize: response.page_size,
      total: response.total,
      items: response.items.map((dto) => this.mapPhoto(dto)),
    };
  }

  private mapPhoto(dto: PhotoModel): Photo {
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

  private mapAlbum(dto: AlbumModel): Album {
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