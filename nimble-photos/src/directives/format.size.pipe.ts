import { Pipe, PipeTransform } from '@angular/core';

import { Formatter } from '../models/formatters';

@Pipe({
    name: 'formatSize'
})
export class FormatSizePipe implements PipeTransform {
    transform(value?: number, options?: { zeroLabel?: string }): string {
        return Formatter.formatBytes(value, options);
    }
}
