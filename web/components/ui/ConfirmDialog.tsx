'use client';

import { useEffect, useRef } from 'react';
import { Button } from '@/components/ui/Primitives';

interface ConfirmDialogProps {
  open: boolean;
  title: string;
  description: string;
  confirmLabel: string;
  loading?: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}

export function ConfirmDialog({
  open,
  title,
  description,
  confirmLabel,
  loading = false,
  onCancel,
  onConfirm,
}: ConfirmDialogProps) {
  const ref = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = ref.current;
    if (!dialog) return;
    if (open && !dialog.open) dialog.showModal();
    if (!open && dialog.open) dialog.close();
  }, [open]);

  return (
    <dialog
      ref={ref}
      className="dialog"
      aria-labelledby="confirm-dialog-title"
      aria-describedby="confirm-dialog-description"
      onCancel={(event) => {
        event.preventDefault();
        onCancel();
      }}
      onClose={onCancel}
    >
      <div className="dialog__signal" aria-hidden="true">!</div>
      <div className="dialog__copy">
        <p className="eyebrow">Confirm Action</p>
        <h2 id="confirm-dialog-title">{title}</h2>
        <p id="confirm-dialog-description">{description}</p>
      </div>
      <div className="dialog__actions">
        <Button variant="secondary" onClick={onCancel}>Keep Running</Button>
        <Button variant="destructive" loading={loading} onClick={onConfirm}>{confirmLabel}</Button>
      </div>
    </dialog>
  );
}
