"use client";

import { Button } from "@heroui/button";
import { Card, CardBody, CardFooter, CardHeader } from "@heroui/card";
import { Form } from "@heroui/form";
import { Input } from "@heroui/input";
import { Link } from "@heroui/link";
import NextLink from 'next/link';
import { FormEvent } from "react";

export default function RegisterPage() {

  const onSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    const data = Object.fromEntries(new FormData(e.currentTarget));

    console.log("Form Data Submitted:", data);
  };

  return (
    <section className="flex items-center flex-col justify-center h-screen p-4">
      <Card className="w-full max-w-3xl p-4">
        <Form onSubmit={onSubmit}>
          <CardHeader className="flex justify-between items-center">
            <h1 className="text-2xl font-bold">Register</h1>
            <Button href="/" as={NextLink} size="sm" variant="light">Back to homepage</Button>
          </CardHeader>
          <CardBody>
            <div className="flex flex-col gap-4">
              <Input
                label="Username"
                labelPlacement="outside"
                variant="bordered"
                description="Username may only contain alphanumeric characters or single hyphens, and cannot begin or end with a hyphen."
                placeholder="Username"
                name="username"
                isRequired
              />
              <Input
                label="Nickname"
                labelPlacement="outside"
                variant="bordered"
                description="Nickname can contain any characters you want and it will not used for identification."
                placeholder="Nickname"
                name="nickname"
              />
              <Input
                label="Password"
                labelPlacement="outside"
                variant="bordered"
                description="Password should be at least 15 characters OR at least 8 characters including a number and a lowercase letter."
                placeholder="Password"
                name="password"
                type="password"
                isRequired
              />
            </div>
            <p className="mt-4 text-sm">
              By signing up, you confirm that you have read and accepted our <Link href="/privacy" className="text-sm">Privacy Policy</Link>.
            </p>
          </CardBody>
          <CardFooter className="flex justify-end gap-4">
            <Button href="/login" as={NextLink} variant="light">
              Already have an account?
            </Button>
            <Button type="submit" color="primary">
              Register
            </Button>
          </CardFooter>
        </Form>
      </Card>
    </section>
  );
}