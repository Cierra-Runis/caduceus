'use client';

import { Form, FormProps } from '@heroui/form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useCallback } from 'react';
import {
  Control,
  DefaultValues,
  SubmitErrorHandler,
  SubmitHandler,
  useForm,
  UseFormSetValue,
} from 'react-hook-form';
import { format } from 'util';
import * as z from 'zod';

export function ZodForm<T extends z.ZodObject>({
  children,
  defaultValues,
  onInvalid,
  onValid,
  schema,
  ...props
}: {
  children: (
    control: Control<z.input<T>, unknown, z.output<T>>,
    setValue: UseFormSetValue<z.core.input<T>>,
  ) => React.ReactNode;
  defaultValues?: DefaultValues<z.core.input<T>>;
  onInvalid?: SubmitErrorHandler<z.input<T>>;
  onValid: SubmitHandler<z.output<T>>;
  schema: T;
} & Omit<FormProps, 'children' | 'onInvalid' | 'onValid'>) {
  const { control, handleSubmit, setValue } = useForm({
    defaultValues,
    resolver: zodResolver(schema),
  });
  const log: SubmitErrorHandler<z.input<T>> = useCallback(
    (errors) =>
      console.error(
        'Form submission failed. Please check the form for errors.',
        format(errors),
      ),
    [],
  );

  return (
    <Form
      onSubmit={async (e) => {
        e.preventDefault();
        try {
          await handleSubmit(onValid, onInvalid ?? log)(e);
        } catch {
          // noop
        }
      }}
      validationBehavior='aria'
      {...props}
    >
      {children(control, setValue)}
    </Form>
  );
}
