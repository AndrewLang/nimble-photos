import { HttpClient, HttpParams } from '@angular/common/http';
import { Injectable, signal } from '@angular/core';
import { catchError, forkJoin, map, Observable, of, switchMap, tap } from 'rxjs';

import { AlbumModel } from '../models/album.model';
import { PagedAlbumsModel } from '../models/paged.albums.model';
import { PagedModel } from '../models/paged.response.model';
import { Album, AlbumComment, GroupedPhotos, PagedPhotos, Photo, PhotoComment, PhotoLoc, PhotoMetadata } from '../models/photo';
import { PhotoLocResponse, PhotoResponse } from '../models/photo.response.model';
import { API_BASE_URL } from './api.config';

@Injectable({
  providedIn: 'root',
})
export class PhotoService {
  readonly apiBase = API_BASE_URL;
  private timelinePhotoIds: string[] | null = null;
  public timelineCache: GroupedPhotos[] | null = null;
  readonly timelineRefreshTick = signal(0);

  lastGalleryScrollIndex = 0;
  readonly isScrolled = signal(false);

  constructor(private readonly http: HttpClient) { }

  getPhotos(page = 1, pageSize = 56): Observable<PagedPhotos> {
    return this.http
      .get<PagedModel<PhotoResponse>>(`${this.apiBase}/photos/${page}/${pageSize}`)
      .pipe(map((response) => this.mapPhotoPage(response)));
  }

  requestTimelineRefresh(): void {
    this.timelineCache = null;
    this.timelinePhotoIds = null;
    this.lastGalleryScrollIndex = 0;
    this.timelineRefreshTick.update(value => value + 1);
  }

  uploadPhotos(files: File[], storageId?: string): Observable<void> {
    const formData = new FormData();
    files.forEach(file => formData.append('files', file, file.name));
    const params = storageId ? new HttpParams().set('storageId', storageId) : undefined;
    return this.http.post<void>(`${this.apiBase}/photos`, formData, { params });
  }

  getPhotoById(id: string): Observable<Photo | null> {
    return this.http.get<PhotoResponse>(`${this.apiBase}/photos/${id}`).pipe(
      map((dto) => this.mapPhoto(dto)),
      catchError(() => of(null))
    );
  }

  getPhotoMetadata(id: string): Observable<PhotoMetadata | null> {
    return this.http
      .get<PhotoMetadata | null>(`${this.apiBase}/photos/metadata/${id}`)
      .pipe(
        map((metadata) => this.asRecord(metadata) as PhotoMetadata | null),
        catchError(() => of(null))
      );
  }

  getAlbumComments(albumId: string): Observable<PagedModel<AlbumComment>> {
    return this.http
      .get<PagedModel<AlbumComment>>(`${this.apiBase}/album/comments/${albumId}`)
      .pipe(
        catchError(() =>
          of({
            page: 1,
            pageSize: 0,
            total: 0,
            items: [],
          }),
        ),
      );
  }

  createAlbumComment(albumId: string, comment: string): Observable<AlbumComment> {
    return this.http.post<AlbumComment>(`${this.apiBase}/album/comments/${albumId}`, { comment });
  }

  updateAlbumCommentVisibility(albumId: string, commentId: string, hidden: boolean): Observable<AlbumComment> {
    return this.http.patch<AlbumComment>(
      `${this.apiBase}/album/comments/visibility/${albumId}/${commentId}`,
      { hidden },
    );
  }

  getPhotoComments(photoId: string): Observable<PhotoComment[]> {
    return this.http
      .get<PhotoComment[] | PagedModel<PhotoComment>>(`${this.apiBase}/photos/comments/${photoId}/1/100`)
      .pipe(
        map((response) => {
          if (Array.isArray(response)) {
            return response;
          }
          return this.asArray(response?.items);
        }),
        catchError(() => of([]))
      );
  }

  createPhotoComment(photoId: string, comment: string): Observable<PhotoComment> {
    return this.http.post<PhotoComment>(`${this.apiBase}/photos/comments/${photoId}`, { comment });
  }

  updatePhotoComment(photoId: string, comment: string | null): Observable<PhotoMetadata | null> {
    return this.http
      .patch<PhotoMetadata | null>(`${this.apiBase}/photos/${photoId}/metadata/comment`, { comment })
      .pipe(
        catchError(() => of(null))
      );
  }

  getAllPhotoTags(): Observable<string[]> {
    return this.http
      .get<string[]>(`${this.apiBase}/photos/tags`)
      .pipe(catchError(() => of([])));
  }

  updatePhotoTags(photoIds: string[], tags: string[]): Observable<{ updated: number }> {
    return this.http.put<{ updated: number }>(`${this.apiBase}/photos/tags`, {
      photoIds,
      tags
    });
  }

  deletePhotos(photoIds: string[]): Observable<{ deleted: number }> {
    if (!photoIds.length) {
      return of({ deleted: 0 });
    }

    return this.http.delete<{ deleted: number }>(`${this.apiBase}/photos`, {
      body: { photoIds }
    });
  }

  getThumbnailPath(photo: Photo): string {
    if (photo.hash) {
      return `${this.apiBase}/photos/thumbnail/${photo.hash}`;
    }

    return photo.path;
  }

  getPreviewPath(photo: Photo): string {
    if (photo.hash) {
      return `${this.apiBase}/photos/preview/${photo.storageId}/${photo.hash}`;
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
          const items = this.asArray(page.items);
          const index = items.findIndex((photo) => photo.id === id);
          return this.resolveAdjacent(items.map(p => p.id), index);
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
        const safeIds = this.asArray(ids);
        const index = safeIds.indexOf(id);
        return this.resolveAdjacent(safeIds, index);
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
      .get<{ title: string; photos: PagedModel<PhotoResponse> }[]>(`${this.apiBase}/timeline/${page}/${pageSize}`)
      .pipe(
        map((groups) =>
          this.asArray(groups).map((g) => ({
            title: g.title,
            photos: this.mapPhotoPage(g.photos),
          }))
        ),
        tap(groups => {
          const safeGroups = this.asArray(groups);
          if (page === 1) {
            this.timelineCache = safeGroups;
            this.timelinePhotoIds = safeGroups.flatMap(g => this.asArray(g?.photos?.items).map(p => p.id));
          } else {
            if (this.timelineCache) {
              this.timelineCache.push(...safeGroups);
            }
            if (this.timelinePhotoIds) {
              this.timelinePhotoIds.push(...safeGroups.flatMap(g => this.asArray(g?.photos?.items).map(p => p.id)));
            }
          }
        }),
        catchError(() => of([]))
      );
  }

  getTimelineRange(startPage: number, endPage: number, pageSize: number = 10): Observable<GroupedPhotos[]> {
    if (startPage > endPage) {
      return of([]);
    }

    const requests = [];
    for (let p = startPage; p <= endPage; p++) {
      requests.push(
        this.http.get<{ title: string; photos: PagedModel<PhotoResponse> }[]>(`${this.apiBase}/photos/timeline/${p}/${pageSize}`)
      );
    }

    return forkJoin(requests).pipe(
      map(results => {
        const allGroups: GroupedPhotos[] = [];
        for (const groups of results) {
          allGroups.push(...this.asArray(groups).map((g) => ({
            title: g.title,
            photos: this.mapPhotoPage(g.photos),
          })));
        }
        return allGroups;
      }),
      tap(allGroups => {
        if (this.timelineCache) {
          this.timelineCache.push(...allGroups);
        }
        if (this.timelinePhotoIds) {
          this.timelinePhotoIds.push(...allGroups.flatMap(g => this.asArray(g?.photos?.items).map(p => p.id)));
        }
      }),
      catchError(() => of([]))
    );
  }

  getTimelineYears(): Observable<string[]> {
    return this.http.get<string[]>(`${this.apiBase}/timeline/years`).pipe(
      catchError(() => of([]))
    );
  }

  getTimelineYearOffset(year: string): Observable<number> {
    return this.http.get<number>(`${this.apiBase}/timeline/year-offset/${year}`).pipe(
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
      .get<PagedModel<AlbumModel>>(`${this.apiBase}/albums/${page}/${pageSize}`)
      .pipe(
        map((response) => ({
          page: response.page,
          pageSize: response.pageSize,
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

  deleteAlbum(id: string): Observable<void> {
    return this.http.delete(`${this.apiBase}/albums/${id}`, { responseType: 'text' }).pipe(
      map(() => undefined)
    );
  }

  updateAlbum(album: Partial<AlbumModel>): Observable<Album> {
    return this.http.put<AlbumModel>(`${this.apiBase}/albums`, album).pipe(
      map(dto => this.mapAlbum(dto))
    );
  }

  addPhotosToAlbum(albumId: string, photoIds: string[]): Observable<{ updated: number }> {
    return this.http.post<{ updated: number }>(`${this.apiBase}/albums/${albumId}/photos`, { photoIds });
  }

  removePhotosFromAlbum(albumId: string, photoIds: string[]): Observable<{ updated: number }> {
    return this.http.delete<{ updated: number }>(`${this.apiBase}/albums/${albumId}/photos`, {
      body: { photoIds }
    });
  }

  getAlbumById(id: string): Observable<Album | null> {
    return this.http.get<AlbumModel>(`${this.apiBase}/albums/${id}`).pipe(
      switchMap((dto) => {
        return this.getAlbumPhotos(id, 1, 100).pipe(
          map((photos) => ({
            ...this.mapAlbum(dto),
            imageCount: photos?.total ?? 0,
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
      .get<PagedModel<PhotoResponse>>(`${this.apiBase}/albums/${albumId}/photos/${page}/${pageSize}`)
      .pipe(
        map((response) => this.mapPhotoPage(response)),
        catchError(() => of(null))
      );
  }

  private mapPhotoPage(response: PagedModel<PhotoResponse>): PagedPhotos {
    const items = this.asArray(response?.items);
    return {
      page: response?.page ?? 1,
      pageSize: response?.pageSize ?? items.length,
      total: response?.total ?? items.length,
      items: items.map((dto) => this.mapPhoto(dto)),
    };
  }

  private mapPhoto(dto: PhotoResponse): Photo {
    return {
      id: dto.id,
      storageId: dto.storageId,
      path: dto.path,
      name: dto.name,
      tags: Array.isArray(dto.tags) ? dto.tags : undefined,
      format: dto.format ?? undefined,
      hash: dto.hash ?? undefined,
      size: dto.size ?? undefined,
      createdAt: this.toDate(dto.createdAt),
      updatedAt: this.toDate(dto.updatedAt),
      dateImported: this.toDate(dto.dateImported),
      dateTaken: this.toDate(dto.dateTaken),
      dayDate: this.toDate(dto.dayDate),
      sortDate: this.toDate(dto.sortDate),
      metadataExtracted: dto.metadataExtracted ?? undefined,
      isRaw: dto.isRaw ?? undefined,
      width: dto.width ?? undefined,
      height: dto.height ?? undefined,
    };
  }

  private mapAlbum(dto: AlbumModel): Album {
    return {
      id: dto.id,
      parentId: dto.parentId ?? undefined,
      name: dto.name,
      createDate: this.toDate(dto.createDate) || new Date(),
      description: dto.description ?? undefined,
      category: dto.category ?? undefined,
      kind: (dto.kind === 'smart' ? 'smart' : 'manual') as Album['kind'],
      thumbnailHash: dto.thumbnailHash ?? undefined,
      sortOrder: dto.sortOrder ?? 0,
      imageCount: dto.imageCount ?? undefined,
    };
  }


  getPhotosWithGps(page = 1, pageSize = 100): Observable<{ page: number, pageSize: number, total: number, items: PhotoLoc[] }> {
    return this.http
      .get<PagedModel<PhotoLocResponse>>(`${this.apiBase}/photos/gps/${page}/${pageSize}`)
      .pipe(
        map((response) => ({
          page: response.page,
          pageSize: response.pageSize,
          total: response.total ?? 0,
          items: response.items.map((dto) => this.mapPhotoLoc(dto)),
        }))
      );
  }

  private mapPhotoLoc(dto: PhotoLocResponse): PhotoLoc {
    const photo = this.mapPhoto(dto);
    return {
      ...photo,
      lat: dto.lat,
      lon: dto.lon
    };
  }

  private toDate(value?: string | null): Date | undefined {
    if (!value) {
      return undefined;
    }
    const parsed = new Date(value);
    return Number.isNaN(parsed.getTime()) ? undefined : parsed;
  }

  private asArray<T>(value: T[] | null | undefined): T[] {
    return Array.isArray(value) ? value : [];
  }

  private asRecord<T>(value: T | null | undefined): T | null {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
      return null;
    }
    return value;
  }
}
