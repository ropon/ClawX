import * as React from 'react';
import { cn } from '@/lib/utils';

export interface CheckboxProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type'> {
  label?: string;
}

const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  ({ className, label, id, ...props }, ref) => {
    const inputId = id || React.useId();
    return (
      <label
        htmlFor={inputId}
        className={cn('flex items-center gap-2 cursor-pointer', className)}
      >
        <input
          ref={ref}
          id={inputId}
          type="checkbox"
          className="h-4 w-4 rounded border-border text-primary accent-primary cursor-pointer"
          {...props}
        />
        {label && <span className="text-sm">{label}</span>}
      </label>
    );
  }
);
Checkbox.displayName = 'Checkbox';

export { Checkbox };
