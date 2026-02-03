import { Component, Type, ViewChild, ViewContainerRef, ViewEncapsulation, signal, inject, AfterViewInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DialogAction, DialogConfig } from '../../models/dialog.model';

@Component({
  selector: 'mtx-dialog',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="fixed inset-0 z-[100] flex items-center justify-center p-4">
      <div 
        class="absolute inset-0 bg-slate-950/60 backdrop-blur-sm animate-in fade-in duration-300"
        (click)="onBackdropClick()">
      </div>

      <div 
        [style.width]="config().width || '450px'"
        [style.max-width]="config().maxWidth || '95vw'"
        class="relative bg-slate-900 border border-white/5 shadow-2xl rounded-3xl overflow-hidden flex flex-col animate-in zoom-in-95 fade-in duration-300">
        
        @if (config().title) {
        <div class="px-6 py-4 flex items-center justify-between border-b border-white/5">
          <h3 class="text-sm font-bold uppercase tracking-widest text-slate-100">{{ config().title }}</h3>
          
          @if (config().showCloseButton !== false) {
          <button 
            (click)="close()"
            class="p-1.5 hover:bg-white/5 rounded-full text-slate-400 hover:text-white transition-all cursor-pointer border-none bg-transparent">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
          }
        </div>
        }

        <div class="px-6 py-6 flex-1 max-h-[70vh] overflow-y-auto custom-scrollbar">
          <ng-container #contentContainer></ng-container>
        </div>

        @if (config().actions && config().actions!.length > 0) {
        <div class="px-6 py-4 bg-slate-950/30 flex items-center justify-end gap-3 border-t border-white/5">
          @for (action of config().actions; track $index) {
          <button 
            (click)="onAction(action)"
            [class]="getActionClasses(action)">
            {{ action.label }}
          </button>
          }
        </div>
        }
      </div>
    </div>
  `,
  encapsulation: ViewEncapsulation.None
})
export class DialogComponent implements AfterViewInit {
  @ViewChild('contentContainer', { read: ViewContainerRef })
  contentContainer!: ViewContainerRef;

  readonly config = signal<DialogConfig>({});

  contentComponent!: Type<any>;
  onClose!: (result?: any) => void;
  readonly componentInstance = signal<any>(null);

  ngAfterViewInit() {
    setTimeout(() => {
      const componentRef = this.contentContainer.createComponent(this.contentComponent);
      this.componentInstance.set(componentRef.instance);
      if (this.config().data) {
        Object.assign(componentRef.instance, this.config().data);
      }
    });
  }

  close(result?: any) {
    this.onClose(result);
  }

  onBackdropClick() {
    if (this.config().closeOnBackdropClick !== false) {
      this.close();
    }
  }

  onAction(action: DialogAction) {
    if (action.closeDialog !== false) {
      this.close(action.value);
    }
  }

  getActionClasses(action: DialogAction): string {
    const base = 'px-5 py-2 rounded-xl text-xs font-bold uppercase tracking-widest transition-all cursor-pointer flex items-center justify-center min-w-[80px] border ';

    switch (action.style) {
      case 'danger':
        return base + 'bg-red-500/10 border-red-500/20 text-red-400 hover:bg-red-500/20';
      case 'ghost':
        return base + 'bg-transparent border-white/5 text-slate-400 hover:text-white hover:bg-white/5';
      case 'secondary':
        return base + 'bg-white/5 border-white/10 text-slate-300 hover:bg-white/10 hover:text-white';
      case 'primary':
      default:
        return base + 'bg-indigo-600 border-indigo-500 text-white hover:bg-indigo-500 shadow-lg shadow-indigo-600/20';
    }
  }
}
