'use client';

import { Input as HeroUIInput, InputProps } from '@heroui/input';
import {
  FieldPath,
  FieldValues,
  useController,
  UseControllerProps,
} from 'react-hook-form';

export function Input<
  TFieldValues extends FieldValues,
  TName extends FieldPath<TFieldValues>,
  TTransformedValues,
>(
  props: InputProps &
    UseControllerProps<TFieldValues, TName, TTransformedValues>,
) {
  const {
    field: { disabled, name, onBlur, onChange, ref, value },
    fieldState: { error, invalid },
  } = useController(props);

  return (
    <HeroUIInput
      {...props}
      errorMessage={error?.message}
      isDisabled={disabled}
      isInvalid={invalid}
      name={name}
      onBlur={onBlur}
      onChange={onChange}
      ref={ref}
      value={value}
    />
  );
}
