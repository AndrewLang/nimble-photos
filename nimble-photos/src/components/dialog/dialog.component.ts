import { Component, Type, ViewChild, ViewContainerRef, ViewEncapsulation, signal, inject, AfterViewInit, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DIALOG_DATA, DialogRef } from '@angular/cdk/dialog';
import { DialogAction, DialogConfig } from '../../models/dialog.model';

@Component({
  selector: 'mtx-dialog',
  imports: [CommonModule],
  templateUrl: './dialog.component.html',
  encapsulation: ViewEncapsulation.None
})
export class DialogComponent implements AfterViewInit, OnInit {
  @ViewChild('contentContainer', { read: ViewContainerRef })
  contentContainer!: ViewContainerRef;

  private readonly dialogRef = inject(DialogRef);
  private readonly data = inject<{ component: Type<any>, config: DialogConfig }>(DIALOG_DATA);

  readonly config = signal<DialogConfig>({});
  readonly componentInstance = signal<any>(null);

  ngOnInit() {
    this.config.set(this.data.config || {});
  }

  ngAfterViewInit() {
    setTimeout(() => {
      if (this.data.component) {
        const componentRef = this.contentContainer.createComponent(this.data.component);
        this.componentInstance.set(componentRef.instance);

        if (this.config().data) {
          Object.assign(componentRef.instance, this.config().data);
        }
      }
    });
  }

  close(result?: any) {
    this.dialogRef.close(result);
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
