"use client";

import { Button } from "@heroui/button";
import { Card, CardBody, CardFooter, CardHeader } from "@heroui/card";
import { Form } from "@heroui/form";
import { Input } from "@heroui/input";
import NextLink from 'next/link';
import { FormEvent } from "react";

export default function LoginPage() {

  const onSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    const data = Object.fromEntries(new FormData(e.currentTarget));

    console.log("Form Data Submitted:", data);
  };

  return (
    <section className="flex items-center flex-col justify-center h-screen p-4">
      <Card className="w-full max-w-3xl p-4">
        <Form onSubmit={onSubmit}>
          <CardHeader >
            <h1 className="text-2xl font-bold">Login</h1>
          </CardHeader>
          <CardBody>
            <div className="flex flex-col gap-4">
              <Input
                label="Username"
                labelPlacement="outside"
                variant="bordered"
                placeholder="Username"
                name="username"
                isRequired
              />
              <Input
                label="Password"
                labelPlacement="outside"
                variant="bordered"
                placeholder="Password"
                description={<NextLink href="/" className="text-primary">Forget Password?</NextLink>}
                name="password"
                type="password"
                isRequired
              />
            </div>
          </CardBody>
          <CardFooter className="flex justify-end gap-4">
            <Button href="/register" as={NextLink} variant="light">
              New to Caduceus?
            </Button>
            <Button type="submit" color="primary">
              Login
            </Button>
          </CardFooter>
        </Form>
      </Card>
    </section >
  );
}