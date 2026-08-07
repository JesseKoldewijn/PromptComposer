type GettingStartedModalProps = {
  open: boolean;
  onClose: () => void;
};

export function GettingStartedModal(props: GettingStartedModalProps) @{
  @if (props.open) {
    <div
      class="modal-backdrop"
      data-testid="getting-started-modal"
      role="presentation"
      onClick={(event) => {
        if (event.target === event.currentTarget) {
          props.onClose();
        }
      }}
    >
      <div
        class="modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="getting-started-title"
      >
        <div class="modal-head">
          <h2 id="getting-started-title">Getting started</h2>
          <button
            class="btn ghost"
            type="button"
            data-testid="getting-started-close"
            aria-label="Close"
            onClick={() => props.onClose()}
          >
            Close
          </button>
        </div>

        <ol class="guide-steps">
          <li>
            <strong>Upload an archive</strong>
            <span>
              Choose an existing <code>.xlsx</code>, or create a blank template with the required
              sheets and <code>e.g.</code> hint rows.
            </span>
          </li>
          <li>
            <strong>Write a query</strong>
            <span>
              Subject Excel row, then Outfit → Pose → Action → optional Scene. Or click{' '}
              <strong>Random</strong> for a valid sample within the archive ranges.
            </span>
          </li>
          <li>
            <strong>Compose</strong>
            <span>
              The prompt appears below. Use <strong>Copy</strong> for the clipboard, and{' '}
              <strong>Show parts</strong> to inspect each piece.
            </span>
          </li>
          <li>
            <strong>Replace or Clear</strong>
            <span>Use the header chip when you want a different archive or to remove the stored one.</span>
          </li>
        </ol>

        <div class="guide-query">
          <div class="label">Query format</div>
          <code class="guide-code">{'<excel-row> <level>lvl<index> …'}</code>
          <p>
            Shorthand <code>N/M</code> equals <code>NlvlM</code>. Example:{' '}
            <code>2 1lvl1 2lvl1 1lvl2</code> or <code>7 1/5 2/3 1/4</code>.
          </p>
        </div>

        <div class="guide-query">
          <div class="label">Archive sheets</div>
          <p>
            Required: <strong>Subjects</strong>, <strong>Outfits</strong>, <strong>Poses</strong>,{' '}
            <strong>Actions</strong>, <strong>Scenes</strong>. Category names must end with{' '}
            <code>{'L{level}-{index}'}</code> (e.g. <code>Outfit L1-01</code> → <code>1lvl1</code>).
          </p>
        </div>

        <div class="actions modal-actions">
          <button
            class="btn primary"
            type="button"
            data-testid="getting-started-dismiss"
            onClick={() => props.onClose()}
          >
            Got it
          </button>
        </div>
      </div>
    </div>
  }
}
