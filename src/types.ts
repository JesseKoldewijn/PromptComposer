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

export interface ArchiveStatus {
  loaded: boolean;
  originalName: string | null;
  importedAt: number | null;
  counts: CatalogCounts | null;
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
