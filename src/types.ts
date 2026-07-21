export interface ComposeErrorPayload {
  code: string;
  message: string;
  suggestion: string | null;
}

export interface PromptPart {
  kind: string;
  label: string;
  text: string;
}

export interface ComposeResult {
  prompt: string;
  parts: PromptPart[];
  query: string;
}

export interface CatalogCounts {
  subjects: number;
  outfits: number;
  poses: number;
  actions: number;
  scenes: number;
}

export interface SubjectRange {
  minRow: number;
  maxRow: number;
}

export interface CategoryRange {
  minLevel: number;
  maxLevel: number;
  minIndex: number;
  maxIndex: number;
}

export interface CatalogRanges {
  subjects: SubjectRange;
  outfits: CategoryRange | null;
  poses: CategoryRange | null;
  actions: CategoryRange | null;
  scenes: CategoryRange | null;
}

export interface ArchiveStatus {
  loaded: boolean;
  originalName: string | null;
  importedAt: number | null;
  counts: CatalogCounts | null;
  ranges: CatalogRanges | null;
}

export function isComposeError(err: unknown): err is ComposeErrorPayload {
  return (
    typeof err === 'object' &&
    err !== null &&
    'code' in err &&
    'message' in err &&
    typeof (err as ComposeErrorPayload).code === 'string' &&
    typeof (err as ComposeErrorPayload).message === 'string'
  );
}

export function errorMessage(err: unknown): ComposeErrorPayload {
  if (isComposeError(err)) {
    return err;
  }
  if (err instanceof Error) {
    return { code: 'unknown', message: err.message, suggestion: null };
  }
  return { code: 'unknown', message: String(err), suggestion: null };
}

export function formatCategoryRange(range: CategoryRange | null | undefined): string {
  if (!range) {
    return '(none)';
  }
  return `levels ${range.minLevel}–${range.maxLevel} · indexes ${range.minIndex}–${range.maxIndex}`;
}

export interface QueryRangeItem {
  label: string;
  value: string;
}

export function formatQueryRangeItems(ranges: CatalogRanges): QueryRangeItem[] {
  const items: QueryRangeItem[] = [
    {
      label: 'Subjects',
      value: `rows ${ranges.subjects.minRow}–${ranges.subjects.maxRow}`,
    },
    { label: 'Outfit', value: formatCategoryRange(ranges.outfits) },
    { label: 'Pose', value: formatCategoryRange(ranges.poses) },
    { label: 'Action', value: formatCategoryRange(ranges.actions) },
  ];
  if (ranges.scenes) {
    items.push({ label: 'Scene', value: formatCategoryRange(ranges.scenes) });
  }
  return items;
}
