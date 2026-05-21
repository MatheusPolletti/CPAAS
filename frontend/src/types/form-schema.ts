import z from "zod";

export const LoginFormSchema = z.object({
  email: z.email({
    message: "Coloque um endereço de e-mail válido.",
  }),
  password: z
    .string()
    .min(1, {
      message: "A senha precisa de no mínimo 1 caracter.",
    })
    .max(1024, {
      message: "A senha não pode ter mais que 1024 caracteres.",
    }),
});

export const RegisterFormSchema = z.object({
  username: z
    .string()
    .min(3, {
      message: "O nome precisa de no mínimo 3 caracteres.",
    })
    .max(255, {
      message: "O nome não pode ter mais que 255 caracteres.",
    }),
  email: z.email({
    message: "Coloque um endereço de e-mail válido.",
  }),
  password: z
    .string()
    .min(8, {
      message: "A senha precisa de no mínimo 8 caracteres.",
    })
    .max(1024, {
      message: "A senha não pode ter mais que 1024 caracteres.",
    }),
});