import { CommonModule } from '@angular/common';
import { Component, computed, effect, inject, input, OnInit, signal } from '@angular/core';
import { catchError, first, of } from 'rxjs';
import { PhotoComment } from '../../models/photo';
import { ResourceLoader } from '../../models/resource.loader';
import { SettingNames } from '../../models/setting.names';
import { AuthService } from '../../services/auth.service';
import { PhotoService } from '../../services/photo.service';
import { SettingsService } from '../../services/settings.service';

const MAX_COMMENT_LENGTH = 1024;

@Component({
    selector: 'mtx-photo-comment',
    templateUrl: 'photo.comment.component.html',
    host: {
        class: 'block',
    },
    imports: [CommonModule]
})
export class PhotoCommentComponent implements OnInit {
    private readonly authService = inject(AuthService);
    private readonly photoService = inject(PhotoService);
    private readonly settingsService = inject(SettingsService);

    readonly photoId = input<string | null>(null);
    readonly commentDraft = signal('');
    readonly isSaving = signal(false);
    readonly maxCommentLength = MAX_COMMENT_LENGTH;
    readonly isLoading = signal(false);
    readonly isEditorVisible = signal(false);
    readonly allowComments = signal(false);
    readonly errorMessage = signal<string | null>(null);
    readonly isAuthenticated = computed(() => this.authService.isAuthenticated());
    readonly comments = new ResourceLoader<PhotoComment[]>();

    private readonly photoIdEffect = effect(() => {
        const id = this.photoId();
        this.commentDraft.set('');
        this.errorMessage.set(null);
        this.isEditorVisible.set(false);

        if (!id) {
            this.comments.set([]);
            this.isLoading.set(false);
            this.errorMessage.set(null);
            return;
        }
        this.loadComments(id);
    });

    constructor() {

    }

    async ngOnInit(): Promise<void> {
        this.settingsService
            .getSettingByName(SettingNames.SiteAllowComments)
            .pipe(first(), catchError(() => of(null)))
            .subscribe(setting => {
                const enabled = typeof setting?.value === 'boolean' ? setting.value : true;
                this.allowComments.set(enabled);
                this.isEditorVisible.set(enabled);
            });
    }

    handleCommentInput(event: Event): void {
        const target = event.target as HTMLTextAreaElement;
        this.commentDraft.set(target.value.slice(0, MAX_COMMENT_LENGTH));
    }

    saveComment(): void {
        const photoId = this.photoId();
        if (!photoId || !this.authService.isAuthenticated() || !this.allowComments()) {
            return;
        }

        const trimmed = this.commentDraft().trim();
        if (trimmed.length === 0 || trimmed.length > MAX_COMMENT_LENGTH) {
            this.errorMessage.set(`Comment must be between 1 and ${MAX_COMMENT_LENGTH} characters.`);
            return;
        }

        this.isSaving.set(true);
        this.errorMessage.set(null);

        this.photoService
            .createPhotoComment(photoId, trimmed)
            .pipe(first())
            .subscribe({
                next: (comment: PhotoComment) => {
                    this.isSaving.set(false);
                    this.comments.value.update(current => [comment, ...current || []]);
                    this.commentDraft.set('');
                    this.isEditorVisible.set(false);
                },
                error: () => {
                    this.isSaving.set(false);
                    this.errorMessage.set('Unable to save your comment.');
                }
            });
    }

    toggleCommentEditor(): void {
        if (!this.allowComments() || !this.authService.isAuthenticated()) {
            return;
        }
        this.isEditorVisible.update(value => !value);
    }

    private loadComments(photoId: string): void {
        this.comments.load(() =>
            this.photoService.getPhotoComments(photoId),
            'Failed to load comments.'
        );
    }
}
