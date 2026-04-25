'use client';
import {
  FieldPath,
  FieldValues,
  useController,
  UseControllerProps,
} from 'react-hook-form';

import {
  Field,
  FieldDescription,
  FieldError,
  FieldLabel,
} from '@/components/ui/field';
import { Input as ShadcnInput } from '@/components/ui/input';

export function Input<
  TFieldValues extends FieldValues,
  TName extends FieldPath<TFieldValues>,
  TTransformedValues,
>({
  description,
  helper,
  inputProps,
  label,
  ...props
}: {
  description?: React.ReactNode;
  helper?: React.ReactNode;
  inputProps?: React.ComponentProps<'input'>;
  label?: React.ReactNode;
} & UseControllerProps<TFieldValues, TName, TTransformedValues>) {
  const {
    field: { disabled, name, onBlur, onChange, ref, value },
    fieldState: { error, invalid },
  } = useController(props);

  return (
    <Field data-invalid={invalid}>
      <div className='flex items-center'>
        <FieldLabel>{label}</FieldLabel>
        {helper}
      </div>
      <ShadcnInput
        {...inputProps}
        disabled={disabled}
        name={name}
        onBlur={onBlur}
        onChange={onChange}
        ref={ref}
        value={value}
      />
      {error != undefined ? (
        <FieldError errors={[error]} />
      ) : (
        <FieldDescription>{description}</FieldDescription>
      )}
    </Field>
  );
}
