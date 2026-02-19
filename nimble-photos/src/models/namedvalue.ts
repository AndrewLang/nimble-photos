export interface NamedValue<T = unknown> extends Record<string, unknown> {
    name: string;
    value: T;
}
