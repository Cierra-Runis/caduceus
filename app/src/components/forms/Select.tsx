'use client';

import { Select as HeroUISelect, SelectProps } from '@heroui/select';
import {
  FieldPath,
  FieldValues,
  useController,
  UseControllerProps,
} from 'react-hook-form';

export function Select<
  TFieldValues extends FieldValues,
  TName extends FieldPath<TFieldValues>,
  TTransformedValues,
  TItem extends object,
>({
  children,
  items,
  ...props
}: Pick<
  SelectProps<TItem>,
  'children' | 'defaultSelectedKeys' | 'items' | 'label'
> &
  UseControllerProps<TFieldValues, TName, TTransformedValues>) {
  const {
    field: { disabled, name, onBlur, onChange, ref, value },
    fieldState: { error, invalid },
  } = useController(props);

  return (
    <HeroUISelect
      {...props}
      errorMessage={error?.message}
      isDisabled={disabled}
      isInvalid={invalid}
      items={items}
      name={name}
      onBlur={onBlur}
      onChange={onChange}
      ref={ref}
      value={value}
    >
      {children}
    </HeroUISelect>
  );
}
