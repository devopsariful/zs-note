import type { $NodeSchema } from '@milkdown/utils';

declare module '@milkdown/preset-commonmark' {
  export const codeBlockSchema: $NodeSchema;
}

declare module '@milkdown/preset-gfm' {
  export const tableSchema: $NodeSchema;
}
