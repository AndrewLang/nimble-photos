import { HttpClient } from '@angular/common/http';
import { Injectable, signal } from '@angular/core';
import { catchError, map, Observable, of, switchMap, tap } from 'rxjs';

import { Album, GroupedPhotos, PagedPhotos, Photo, PhotoLoc } from '../models/photo';
import { AlbumModel } from '../models/album.model';
import { PagedResponseModel } from '../models/paged.response.model';
import { PagedAlbumsModel } from '../models/paged.albums.model';


type DateFieldKey = 'createdAt' | 'updatedAt' | 'dateImported' | 'dateTaken';

type PhotoResponse = Omit<Photo, DateFieldKey> & {
  createdAt?: string | null;
  updatedAt?: string | null;
  dateImported?: string | null;
  dateTaken?: string | null;
};

type PhotoLocResponse = PhotoResponse & {
  lat: number;
  lon: number;
};

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
      .get<PagedResponseModel<PhotoResponse>>(`${this.apiBase}/photos/${page}/${pageSize}`)
      .pipe(map((response) => this.mapPhotoPage(response)));
  }

  getPhotoById(id: string): Observable<Photo | null> {
    return this.http.get<PhotoResponse>(`${this.apiBase}/photos/${id}`).pipe(
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
      .get<{ title: string; photos: PagedResponseModel<PhotoResponse> }[]>(`${this.apiBase}/photos/timeline/${page}/${pageSize}`)
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

  getAlbumById(id: string): Observable<Album | null> {
    return this.http.get<AlbumModel>(`${this.apiBase}/albums/${id}`).pipe(
      switchMap((dto) => {
        const rules = JSON.parse(dto.rulesJson || '{ "photoIds": [] }');

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
      .get<PagedResponseModel<PhotoResponse>>(`${this.apiBase}/albums/${albumId}/photos/${page}/${pageSize}`)
      .pipe(
        map((response) => this.mapPhotoPage(response)),
        catchError(() => of(null))
      );
  }

  private mapPhotoPage(response: PagedResponseModel<PhotoResponse>): PagedPhotos {
    return {
      page: response.page,
      pageSize: response.pageSize,
      total: response.total,
      items: response.items.map((dto) => this.mapPhoto(dto)),
    };
  }

  private mapPhoto(dto: PhotoResponse): Photo {
    return {
      id: dto.id,
      path: dto.path,
      name: dto.name,
      format: dto.format ?? undefined,
      hash: dto.hash ?? undefined,
      size: dto.size ?? undefined,
      createdAt: this.toDate(dto.createdAt),
      updatedAt: this.toDate(dto.updatedAt),
      dateImported: this.toDate(dto.dateImported),
      dateTaken: this.toDate(dto.dateTaken),
      thumbnailPath: dto.thumbnailPath ?? undefined,
      thumbnailOptimized: dto.thumbnailOptimized ?? undefined,
      metadataExtracted: dto.metadataExtracted ?? undefined,
      isRaw: dto.isRaw ?? undefined,
      width: dto.width ?? undefined,
      height: dto.height ?? undefined,
      thumbnailWidth: dto.thumbnailWidth ?? undefined,
      thumbnailHeight: dto.thumbnailHeight ?? undefined,
    };
  }

  private mapAlbum(dto: AlbumModel): Album {
    return {
      id: dto.id,
      parentId: dto.parentId ?? undefined,
      name: dto.name,
      createDate: this.toDate(dto.createDate),
      description: dto.description ?? undefined,
      category: dto.category ?? undefined,
      kind: (dto.kind === 'smart' ? 'smart' : 'manual') as Album['kind'],
      rulesJson: dto.rulesJson ?? undefined,
      thumbnailHash: dto.thumbnailHash ?? undefined,
      sortOrder: dto.sortOrder ?? 0,
      imageCount: dto.imageCount ?? undefined,
    };
  }


  getPhotosWithGps(page = 1, pageSize = 100): Observable<{ page: number, pageSize: number, total: number, items: PhotoLoc[] }> {
    return this.http
      .get<PagedResponseModel<PhotoLocResponse>>(`${this.apiBase}/photos/with-gps/${page}/${pageSize}`)
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
}
